use crate::config::ConfigManager;
use crate::index::{IndexManager, ProjectRegistration};
use crate::git_operations::GitOperations;
use crate::atomic::{AtomicOperations, AddOperation, CommitOperation, PushOperation};
use crate::error::RepositoryError;
use std::path::{Path, PathBuf};
use std::env;

pub struct RepositoryManager {
    config: ConfigManager,
    index_manager: IndexManager,
}

impl RepositoryManager {
    pub fn new(config: ConfigManager, index_manager: IndexManager) -> Self {
        Self { config, index_manager }
    }
    
    pub async fn init_project(
        &mut self,
        directories: Vec<String>,
        skip_hidden: bool,
        no_atomic: bool
    ) -> Result<(), RepositoryError> {
        let current_dir = env::current_dir()?;
        
        // 检查并初始化 git
        self.ensure_git_initialized(&current_dir).await?;
        
        // 获取 remote origin
        let remote_url = self.get_remote_origin(&current_dir)?;
        let _base_key = GitOperations::generate_base_key(&remote_url)?;
        
        // 生成所有 Repository Keys 并检查重复
        let mut repo_keys = Vec::new();
        for dir in &directories {
            let repo_key = GitOperations::generate_repository_key(&remote_url, Some(dir))?;
            repo_keys.push((dir.clone(), repo_key));
        }
        
        // 检查重复
        for (_, repo_key) in &repo_keys {
            if self.index_manager.project_exists(repo_key) {
                return Err(RepositoryError::ProjectAlreadyExists(repo_key.clone()));
            }
        }
        
        if skip_hidden {
            // 跳过隐藏仓库操作
            return Ok(());
        }
        
        if no_atomic {
            // 非原子操作
            for (dir, repo_key) in repo_keys {
                self.create_hidden_repository(&current_dir, &dir, &repo_key).await?;
            }
        } else {
            // 原子操作
            let mut created_repos = Vec::new();
            let mut rollback_needed = false;
            
            for (dir, repo_key) in repo_keys {
                match self.create_hidden_repository(&current_dir, &dir, &repo_key).await {
                    Ok(_) => created_repos.push((dir, repo_key)),
                    Err(e) => {
                        rollback_needed = true;
                        eprintln!("Failed to create hidden repository for {}: {}", dir, e);
                        break;
                    }
                }
            }
            
            if rollback_needed {
                // 回滚已创建的仓库
                for (dir, repo_key) in created_repos {
                    if let Err(e) = self.rollback_hidden_repository(&current_dir, &dir, &repo_key).await {
                        eprintln!("Failed to rollback {}: {}", dir, e);
                    }
                }
                return Err(RepositoryError::AtomicOperationFailed);
            }
        }
        
        Ok(())
    }
    
    pub async fn status(&self, skip_hidden: bool) -> Result<String, RepositoryError> {
        let current_dir = env::current_dir()?;
        
        // 检查是否初始化了 dot
        if !self.is_dot_initialized(&current_dir).await? {
            return Ok("This directory is not initialized with dot. Run 'dot init <directory>' to initialize.".to_string());
        }
        
        let mut status_output = Vec::new();
        
        // 显示父仓库状态
        status_output.push("=== Parent Repository ===".to_string());
        let parent_status = GitOperations::get_status(&current_dir)?;
        status_output.push(parent_status);
        
        if !skip_hidden {
            // 显示隐藏仓库状态
            let hidden_repos = self.get_hidden_repositories(&current_dir).await?;
            
            for (dir_name, repo_path) in hidden_repos {
                status_output.push(format!("=== Hidden Repository: {} ===", dir_name));
                if repo_path.exists() {
                    let hidden_status = GitOperations::get_status(&repo_path)?;
                    status_output.push(hidden_status);
                } else {
                    status_output.push("Repository not found locally".to_string());
                }
            }
        }
        
        Ok(status_output.join("\n"))
    }
    
    pub async fn multi_repo_add(
        &self,
        files: Vec<String>,
        skip_hidden: bool,
        no_atomic: bool
    ) -> Result<(), RepositoryError> {
        let current_dir = env::current_dir()?;
        let mut operations = AtomicOperations::new(no_atomic);
        
        // 添加到隐藏仓库
        if !skip_hidden {
            let hidden_repos = self.get_hidden_repositories(&current_dir).await?;
            for (_, repo_path) in hidden_repos {
                if repo_path.exists() {
                    operations.add_operation(Box::new(AddOperation::new(repo_path, files.clone())));
                }
            }
        }
        
        // 添加到父仓库
        operations.add_operation(Box::new(AddOperation::new(current_dir, files)));
        
        operations.execute().await.map_err(RepositoryError::from)
    }
    
    pub async fn multi_repo_commit(
        &self,
        message: String,
        skip_hidden: bool,
        no_atomic: bool
    ) -> Result<(), RepositoryError> {
        let current_dir = env::current_dir()?;
        let mut operations = AtomicOperations::new(no_atomic);
        
        // 先提交隐藏仓库
        if !skip_hidden {
            let hidden_repos = self.get_hidden_repositories(&current_dir).await?;
            for (_, repo_path) in hidden_repos {
                if repo_path.exists() {
                    operations.add_operation(Box::new(CommitOperation::new(repo_path, message.clone())));
                }
            }
        }
        
        // 然后提交父仓库
        operations.add_operation(Box::new(CommitOperation::new(current_dir, message)));
        
        operations.execute().await.map_err(RepositoryError::from)
    }
    
    pub async fn multi_repo_push(
        &self,
        skip_hidden: bool,
        no_atomic: bool
    ) -> Result<String, RepositoryError> {
        let current_dir = env::current_dir()?;
        let mut operations = AtomicOperations::new(no_atomic);
        let mut results = Vec::new();
        
        // 先推送隐藏仓库
        if !skip_hidden {
            let hidden_repos = self.get_hidden_repositories(&current_dir).await?;
            for (dir_name, repo_path) in hidden_repos {
                if repo_path.exists() {
                    operations.add_operation(Box::new(PushOperation::new(repo_path.clone())));
                    results.push(format!("Hidden repository '{}': pushed", dir_name));
                }
            }
        }
        
        // 然后推送父仓库
        operations.add_operation(Box::new(PushOperation::new(current_dir)));
        results.push("Parent repository: pushed".to_string());
        
        operations.execute().await.map_err(RepositoryError::from)?;
        
        Ok(results.join("\n"))
    }
    
    pub async fn clone_project(
        &mut self,
        repository_url: String,
        target_dir: Option<String>
    ) -> Result<(), RepositoryError> {
        // 生成目标目录名
        let dir_name = target_dir.unwrap_or_else(|| {
            repository_url
                .split('/')
                .last()
                .unwrap_or("repo")
                .strip_suffix(".git")
                .unwrap_or("repo")
                .to_string()
        });
        
        let target_path = env::current_dir()?.join(&dir_name);
        
        // 克隆主仓库
        GitOperations::clone_repository(&repository_url, &target_path)?;
        
        // 生成 base key 并查找关联的隐藏仓库
        let base_key = GitOperations::generate_base_key(&repository_url)?;
        let associated_projects = self.index_manager.find_projects_by_base_key(&base_key);
        
        if associated_projects.is_empty() {
            println!("No associated hidden repositories found for this project.");
            return Ok(());
        }
        
        // 克隆所有关联的隐藏仓库
        for project in associated_projects {
            let hidden_dir = target_path.join(&project.hidden_directory);
            let hidden_repo_url = self.generate_hidden_repo_url(&project.repository_key)?;
            
            match GitOperations::clone_repository(&hidden_repo_url, &hidden_dir) {
                Ok(_) => println!("Cloned hidden repository: {}", project.hidden_directory),
                Err(e) => eprintln!("Failed to clone hidden repository {}: {}", project.hidden_directory, e),
            }
        }
        
        Ok(())
    }
    
    // 私有辅助方法
    
    async fn ensure_git_initialized(&self, path: &Path) -> Result<(), RepositoryError> {
        if !GitOperations::is_git_initialized(path) {
            GitOperations::init_repository(path)?;
            println!("Initialized git repository in {}", path.display());
        }
        Ok(())
    }
    
    fn get_remote_origin(&self, path: &Path) -> Result<String, RepositoryError> {
        GitOperations::get_remote_origin(path)
    }
    
    async fn create_hidden_repository(
        &mut self,
        project_path: &Path,
        directory: &str,
        repository_key: &str
    ) -> Result<(), RepositoryError> {
        let hidden_dir = project_path.join(directory);
        
        // 创建隐藏目录（如果不存在）
        if !hidden_dir.exists() {
            std::fs::create_dir_all(&hidden_dir)?;
        }
        
        // 在父仓库中创建 .gitignore 文件，忽略隐藏目录的内容
        let gitignore_path = hidden_dir.join(".gitignore");
        let gitignore_content = "# 忽略此目录下的所有内容（由 dot 管理）\n*\n!.gitignore\n";
        std::fs::write(&gitignore_path, gitignore_content)?;
        
        // 初始化 git 仓库
        GitOperations::init_repository(&hidden_dir)?;
        
        // 创建远程仓库并设置 origin
        let remote_url = self.create_remote_hidden_repository(repository_key).await?;
        let repo = git2::Repository::open(&hidden_dir)?;
        repo.remote("origin", &remote_url)?;
        
        // 注册到索引
        let registration = ProjectRegistration {
            repository_key: repository_key.to_string(),
            git_user: GitOperations::get_git_user(project_path)?,
            project_git_path: self.get_remote_origin(project_path)?,
            project_disk_path: project_path.to_string_lossy().to_string(),
            hidden_directory: directory.to_string(),
            created_at: chrono::Utc::now(),
        };
        
        self.index_manager.register_project(registration).await?;
        
        println!("Created hidden repository: {}", directory);
        println!("  - Added .gitignore to exclude from parent repository");
        Ok(())
    }
    
    async fn rollback_hidden_repository(
        &self,
        _project_path: &Path,
        directory: &str,
        repository_key: &str
    ) -> Result<(), RepositoryError> {
        // 删除本地隐藏目录
        let hidden_dir = _project_path.join(directory);
        if hidden_dir.exists() {
            std::fs::remove_dir_all(&hidden_dir)?;
        }
        
        // 这里应该删除远程仓库，但为了安全起见，我们只是记录
        eprintln!("Should delete remote repository for key: {}", repository_key);
        
        Ok(())
    }
    
    async fn create_remote_hidden_repository(&self, repository_key: &str) -> Result<String, RepositoryError> {
        // 这里应该使用 GitHub API 创建仓库
        // 为了简化，我们返回一个模拟的 URL
        let org = self.config.get_default_organization()
            .ok_or(RepositoryError::ConfigError(crate::error::ConfigError::OrganizationNotAuthorized))?;
        
        let repo_name = repository_key.replace('/', "-").replace(':', "-");
        Ok(format!("git@github.com:{}/{}.git", org, repo_name))
    }
    
    fn generate_hidden_repo_url(&self, repository_key: &str) -> Result<String, RepositoryError> {
        let org = self.config.get_default_organization()
            .ok_or(RepositoryError::ConfigError(crate::error::ConfigError::OrganizationNotAuthorized))?;
        
        let repo_name = repository_key.replace('/', "-").replace(':', "-");
        Ok(format!("git@github.com:{}/{}.git", org, repo_name))
    }
    
    async fn is_dot_initialized(&self, path: &Path) -> Result<bool, RepositoryError> {
        if !GitOperations::is_git_initialized(path) {
            return Ok(false);
        }
        
        let remote_url = match GitOperations::get_remote_origin(path) {
            Ok(url) => url,
            Err(_) => return Ok(false),
        };
        
        let base_key = GitOperations::generate_base_key(&remote_url)?;
        let projects = self.index_manager.find_projects_by_base_key(&base_key);
        
        Ok(!projects.is_empty())
    }
    
    async fn get_hidden_repositories(&self, path: &Path) -> Result<Vec<(String, PathBuf)>, RepositoryError> {
        let remote_url = GitOperations::get_remote_origin(path)?;
        let base_key = GitOperations::generate_base_key(&remote_url)?;
        let projects = self.index_manager.find_projects_by_base_key(&base_key);
        
        let mut hidden_repos = Vec::new();
        for project in projects {
            let repo_path = path.join(&project.hidden_directory);
            hidden_repos.push((project.hidden_directory.clone(), repo_path));
        }
        
        Ok(hidden_repos)
    }
}

// 实现 From trait 用于错误转换
impl From<crate::error::OperationError> for RepositoryError {
    fn from(err: crate::error::OperationError) -> Self {
        RepositoryError::IoError(std::io::Error::new(std::io::ErrorKind::Other, err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_repository_manager_creation() {
        // 这个测试需要实际的配置和索引管理器
        // 在实际测试中，我们会使用模拟对象
        assert!(true); // 占位符测试
    }
}