use crate::config::ConfigManager;
use crate::error::IndexError;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectRegistration {
    pub repository_key: String,
    pub git_user: String,
    pub project_git_path: String,
    pub project_disk_path: String,
    pub hidden_directory: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexData {
    pub projects: HashMap<String, ProjectRegistration>,
}

impl Default for IndexData {
    fn default() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }
}

pub struct IndexManager {
    local_index_path: PathBuf,
    remote_organization: String,
    github_client: Octocrab,
    index_data: IndexData,
}

impl IndexManager {
    pub async fn new(config: &ConfigManager) -> Result<Self, IndexError> {
        let org = config.get_default_organization()
            .ok_or(IndexError::NoDefaultOrganization)?
            .clone();
            
        let github_client = Octocrab::builder()
            .personal_token(Self::get_github_token()?)
            .build()
            .map_err(IndexError::GitHubError)?;
            
        let local_index_path = Self::local_index_path()?;
        
        // 检查并设置索引仓库
        let mut manager = Self {
            local_index_path,
            remote_organization: org,
            github_client,
            index_data: IndexData::default(),
        };
        
        manager.ensure_index_repository().await?;
        manager.load_index_data().await?;
        
        Ok(manager)
    }
    
    async fn ensure_index_repository(&self) -> Result<(), IndexError> {
        // 检查远程 .index 仓库是否存在
        let repo_exists = self.github_client
            .repos(&self.remote_organization, ".index")
            .get()
            .await
            .is_ok();
            
        if !repo_exists {
            // 创建 .index 仓库
            self.github_client
                .repos(&self.remote_organization)
                .create()
                .name(".index")
                .description("Dot CLI index repository")
                .private(true)
                .send()
                .await?;
        }
        
        // 克隆或更新本地索引
        if !self.local_index_path.exists() {
            self.clone_index_repository().await?;
        } else {
            self.update_local_index().await?;
        }
        
        Ok(())
    }
    
    async fn clone_index_repository(&self) -> Result<(), IndexError> {
        let clone_url = format!("https://github.com/{}/{}.git", self.remote_organization, ".index");
        
        // 确保父目录存在
        if let Some(parent) = self.local_index_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // 使用 git2 克隆仓库
        let repo = git2::Repository::clone(&clone_url, &self.local_index_path)
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        // 如果仓库是空的，创建初始的 index.json 文件
        let index_file = self.local_index_path.join("index.json");
        if !index_file.exists() {
            let initial_data = IndexData::default();
            let content = serde_json::to_string_pretty(&initial_data)?;
            tokio::fs::write(&index_file, content).await?;
            
            // 提交初始文件
            let mut index = repo.index().map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            index.add_path(std::path::Path::new("index.json")).map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            index.write().map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
            let tree_id = index.write_tree().map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            let tree = repo.find_tree(tree_id).map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
            let signature = git2::Signature::now("dot-cli", "dot-cli@example.com")
                .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initialize index repository",
                &tree,
                &[],
            ).map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        }
        
        Ok(())
    }
    
    async fn update_local_index(&self) -> Result<(), IndexError> {
        let repo = git2::Repository::open(&self.local_index_path)
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        // 简单的 git pull 操作
        let mut remote = repo.find_remote("origin")
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], None, None)
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        Ok(())
    }
    
    async fn load_index_data(&mut self) -> Result<(), IndexError> {
        let index_file = self.local_index_path.join("index.json");
        
        if index_file.exists() {
            let content = tokio::fs::read_to_string(&index_file).await?;
            self.index_data = serde_json::from_str(&content)?;
        } else {
            self.index_data = IndexData::default();
        }
        
        Ok(())
    }
    
    pub async fn register_project(&mut self, registration: ProjectRegistration) -> Result<(), IndexError> {
        // 检查是否已存在
        if self.index_data.projects.contains_key(&registration.repository_key) {
            return Err(IndexError::ProjectAlreadyExists(registration.repository_key));
        }
        
        // 添加到索引
        self.index_data.projects.insert(
            registration.repository_key.clone(),
            registration
        );
        
        // 保存并推送更改
        self.save_and_push_index().await?;
        
        Ok(())
    }
    
    pub fn project_exists(&self, repository_key: &str) -> bool {
        self.index_data.projects.contains_key(repository_key)
    }
    
    pub fn find_projects_by_base_key(&self, base_key: &str) -> Vec<&ProjectRegistration> {
        self.index_data.projects
            .values()
            .filter(|p| p.repository_key.starts_with(base_key))
            .collect()
    }
    
    async fn save_and_push_index(&self) -> Result<(), IndexError> {
        let index_file = self.local_index_path.join("index.json");
        let content = serde_json::to_string_pretty(&self.index_data)?;
        tokio::fs::write(&index_file, content).await?;
        
        // Git add, commit, push
        let repo = git2::Repository::open(&self.local_index_path)
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let mut index = repo.index()
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        index.add_path(std::path::Path::new("index.json"))
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        index.write()
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let tree_id = index.write_tree()
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let tree = repo.find_tree(tree_id)
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let signature = git2::Signature::now("dot-cli", "dot-cli@example.com")
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let parent_commit = repo.head()
            .and_then(|h| h.target().ok_or(git2::Error::from_str("No target")))
            .and_then(|oid| repo.find_commit(oid))
            .map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Update index",
            &tree,
            &[&parent_commit],
        ).map_err(|e| IndexError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        Ok(())
    }
    
    fn get_github_token() -> Result<String, IndexError> {
        env::var("GITHUB_TOKEN")
            .or_else(|_| env::var("GH_TOKEN"))
            .map_err(|_| IndexError::GitHubTokenNotFound)
    }
    
    fn local_index_path() -> Result<PathBuf, IndexError> {
        let home = dirs::home_dir().ok_or(IndexError::IoError(
            std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
        ))?;
        Ok(home.join(".dot").join(".index"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_index_data_serialization() {
        let mut index_data = IndexData::default();
        
        let registration = ProjectRegistration {
            repository_key: "github.com/user/repo/.kiro".to_string(),
            git_user: "testuser".to_string(),
            project_git_path: "git@github.com:user/repo.git".to_string(),
            project_disk_path: "/home/user/repo".to_string(),
            hidden_directory: ".kiro".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        index_data.projects.insert(registration.repository_key.clone(), registration);
        
        let json = serde_json::to_string_pretty(&index_data).unwrap();
        let deserialized: IndexData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(index_data.projects.len(), deserialized.projects.len());
    }
    
    #[test]
    fn test_find_projects_by_base_key() {
        let mut index_data = IndexData::default();
        
        let reg1 = ProjectRegistration {
            repository_key: "github.com/user/repo/.kiro".to_string(),
            git_user: "testuser".to_string(),
            project_git_path: "git@github.com:user/repo.git".to_string(),
            project_disk_path: "/home/user/repo".to_string(),
            hidden_directory: ".kiro".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        let reg2 = ProjectRegistration {
            repository_key: "github.com/user/repo/.config".to_string(),
            git_user: "testuser".to_string(),
            project_git_path: "git@github.com:user/repo.git".to_string(),
            project_disk_path: "/home/user/repo".to_string(),
            hidden_directory: ".config".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        index_data.projects.insert(reg1.repository_key.clone(), reg1);
        index_data.projects.insert(reg2.repository_key.clone(), reg2);
        
        let manager = IndexManager {
            local_index_path: PathBuf::new(),
            remote_organization: "test".to_string(),
            github_client: Octocrab::default(),
            index_data,
        };
        
        let results = manager.find_projects_by_base_key("github.com/user/repo");
        assert_eq!(results.len(), 2);
    }
}