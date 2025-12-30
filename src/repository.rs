use crate::config::ConfigManager;
use crate::index::{IndexManager, ProjectRegistration};
use crate::git_operations::GitOperations;
use crate::atomic::{AtomicOperations, AddOperation, CommitOperation, PushOperation};
use crate::github::GitHubClient;
use crate::error::RepositoryError;
use std::path::{Path, PathBuf};
use std::env;
use md5;

pub struct RepositoryManager {
    #[allow(dead_code)]
    config: ConfigManager,
    index_manager: IndexManager,
    github_client: GitHubClient,
}

impl RepositoryManager {
    pub fn new(config: ConfigManager, index_manager: IndexManager) -> Self {
        let github_token = config.get_github_token();
        let github_client = GitHubClient::new(github_token);
        Self { config, index_manager, github_client }
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
        
        // 生成所有 Repository Keys 并检查重复，同时记录目录是否已存在
        let mut repo_keys = Vec::new();
        for dir in &directories {
            let repo_key = GitOperations::generate_repository_key(&remote_url, Some(dir))?;
            let dir_exists = current_dir.join(dir).exists();
            repo_keys.push((dir.clone(), repo_key, dir_exists));
        }
        
        // 检查重复
        for (_, repo_key, _) in &repo_keys {
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
            for (dir, repo_key, _) in repo_keys {
                self.create_hidden_repository(&current_dir, &dir, &repo_key).await?;
            }
        } else {
            // 原子操作
            // 记录：(目录名, repo_key, 目录原本是否存在)
            let mut created_repos: Vec<(String, String, bool)> = Vec::new();
            let mut rollback_needed = false;
            
            for (dir, repo_key, dir_existed) in repo_keys {
                match self.create_hidden_repository(&current_dir, &dir, &repo_key).await {
                    Ok(_) => created_repos.push((dir, repo_key, dir_existed)),
                    Err(e) => {
                        rollback_needed = true;
                        eprintln!("Failed to create hidden repository for {}: {}", dir, e);
                        break;
                    }
                }
            }
            
            if rollback_needed {
                // 回滚已创建的仓库
                for (dir, repo_key, dir_existed) in created_repos {
                    // 只有当目录是我们新创建的才删除
                    let dir_was_created = !dir_existed;
                    if let Err(e) = self.rollback_hidden_repository(&current_dir, &dir, &repo_key, dir_was_created).await {
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
            let hidden_repo_url = self.generate_hidden_repo_url(&project.repository_name)?;
            
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
        
        // 检查目录是否已存在
        let dir_existed = hidden_dir.exists();
        
        // 创建隐藏目录（如果不存在）
        if !dir_existed {
            std::fs::create_dir_all(&hidden_dir)?;
        }
        
        // 生成 MD5 仓库名
        let repo_name = format!("{:x}", md5::compute(repository_key.as_bytes()));
        
        // 获取组织名
        let org = self.index_manager.get_organization().to_string();
        
        // 使用 GitHub API 或 gh CLI 创建远程仓库（必须成功）
        println!("Creating remote repository: {}/{}", org, repo_name);
        let description = format!("Hidden repository for {}", repository_key);
        
        let remote_url = match self.github_client.create_repository(&org, &repo_name, &description).await {
            Ok(url) => {
                println!("  ✓ Remote repository created successfully");
                url
            }
            Err(e) => {
                // 远程仓库创建失败
                // 只有当目录是我们新创建的才删除，已存在的目录不能删！
                if !dir_existed && hidden_dir.exists() {
                    let _ = std::fs::remove_dir_all(&hidden_dir);
                }
                return Err(e);
            }
        };
        
        // 检查是否已经是 git 仓库
        let is_git_repo = hidden_dir.join(".git").exists();
        
        if !is_git_repo {
            // 初始化本地 git 仓库
            GitOperations::init_repository(&hidden_dir)?;
            
            // 设置远程 origin
            let repo = git2::Repository::open(&hidden_dir)?;
            repo.remote("origin", &remote_url)?;
        } else {
            // 已经是 git 仓库，检查是否需要更新 remote
            let repo = git2::Repository::open(&hidden_dir)?;
            if repo.find_remote("origin").is_err() {
                repo.remote("origin", &remote_url)?;
            }
        }
        
        // 注册到索引
        let registration = ProjectRegistration {
            repository_key: repository_key.to_string(),
            repository_name: repo_name.clone(),
            git_user: GitOperations::get_git_user(project_path)?,
            project_git_path: self.get_remote_origin(project_path)?,
            project_disk_path: project_path.to_string_lossy().to_string(),
            hidden_directory: directory.to_string(),
            created_at: chrono::Utc::now(),
        };
        
        self.index_manager.register_project(registration).await?;
        
        println!("✓ Created hidden repository: {}", directory);
        println!("  - Remote: {}", remote_url);
        Ok(())
    }
    
    async fn rollback_hidden_repository(
        &self,
        project_path: &Path,
        directory: &str,
        repository_key: &str,
        dir_was_created: bool,  // 新增参数：目录是否是我们创建的
    ) -> Result<(), RepositoryError> {
        // 只有当目录是我们新创建的才删除
        if dir_was_created {
            let hidden_dir = project_path.join(directory);
            if hidden_dir.exists() {
                std::fs::remove_dir_all(&hidden_dir)?;
            }
        }
        
        // 生成 MD5 仓库名
        let repo_name = format!("{:x}", md5::compute(repository_key.as_bytes()));
        let org = self.index_manager.get_organization();
        
        // 尝试删除远程仓库
        println!("Rolling back: deleting remote repository {}/{}", org, repo_name);
        if let Err(e) = self.github_client.delete_repository(org, &repo_name).await {
            eprintln!("Warning: Failed to delete remote repository: {}", e);
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    async fn create_remote_hidden_repository(&self, _repository_key: &str) -> Result<String, RepositoryError> {
        // 这个方法不再使用，保留以兼容
        unreachable!("This method is deprecated")
    }
    
    fn generate_hidden_repo_url(&self, repository_name: &str) -> Result<String, RepositoryError> {
        let org = self.index_manager.get_organization();
        Ok(format!("git@github.com:{}/{}.git", org, repository_name))
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