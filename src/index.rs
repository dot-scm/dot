use crate::config::ConfigManager;
use crate::error::IndexError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

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
    index_data: IndexData,
}

impl IndexManager {
    pub async fn new(config: &ConfigManager) -> Result<Self, IndexError> {
        let org = config.get_default_organization()
            .ok_or(IndexError::NoDefaultOrganization)?
            .clone();
            
        let local_index_path = Self::local_index_path()?;
        
        // 检查并设置索引仓库
        let mut manager = Self {
            local_index_path,
            remote_organization: org,
            index_data: IndexData::default(),
        };
        
        manager.ensure_index_repository().await?;
        manager.load_index_data().await?;
        
        Ok(manager)
    }
    
    async fn ensure_index_repository(&self) -> Result<(), IndexError> {
        // 检查本地索引目录是否存在
        if self.local_index_path.exists() {
            // 本地已存在，尝试更新
            self.update_local_index().await?;
            return Ok(());
        }
        
        // 尝试克隆远程 .index 仓库
        let clone_result = self.clone_index_repository().await;
        
        match clone_result {
            Ok(_) => Ok(()),
            Err(_) => {
                // 克隆失败，可能是仓库不存在
                // 创建本地索引目录和初始文件
                println!("⚠️  无法克隆索引仓库，将创建本地索引");
                println!("   请确保在 GitHub 上创建了 .index 仓库");
                self.create_local_index().await?;
                Ok(())
            }
        }
    }
    
    async fn clone_index_repository(&self) -> Result<(), IndexError> {
        // 使用 SSH URL（利用用户的 Git 凭证）
        let clone_url = format!("git@github.com:{}/{}.git", self.remote_organization, ".index");
        
        // 确保父目录存在
        if let Some(parent) = self.local_index_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // 使用 git 命令克隆（利用系统的 Git 凭证）
        let output = Command::new("git")
            .args(["clone", &clone_url, self.local_index_path.to_str().unwrap()])
            .output()
            .map_err(|e| IndexError::IoError(e))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IndexError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to clone index repository: {}", stderr)
            )));
        }
        
        // 如果仓库是空的，创建初始的 index.json 文件
        let index_file = self.local_index_path.join("index.json");
        if !index_file.exists() {
            self.initialize_index_file().await?;
        }
        
        Ok(())
    }
    
    async fn create_local_index(&self) -> Result<(), IndexError> {
        // 创建本地索引目录
        tokio::fs::create_dir_all(&self.local_index_path).await?;
        
        // 初始化 Git 仓库
        let output = Command::new("git")
            .args(["init"])
            .current_dir(&self.local_index_path)
            .output()
            .map_err(|e| IndexError::IoError(e))?;
            
        if !output.status.success() {
            return Err(IndexError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to initialize local index repository"
            )));
        }
        
        // 设置远程 origin
        let remote_url = format!("git@github.com:{}/{}.git", self.remote_organization, ".index");
        let _ = Command::new("git")
            .args(["remote", "add", "origin", &remote_url])
            .current_dir(&self.local_index_path)
            .output();
        
        // 创建初始 index.json
        self.initialize_index_file().await?;
        
        Ok(())
    }
    
    async fn initialize_index_file(&self) -> Result<(), IndexError> {
        let index_file = self.local_index_path.join("index.json");
        let initial_data = IndexData::default();
        let content = serde_json::to_string_pretty(&initial_data)?;
        tokio::fs::write(&index_file, &content).await?;
        
        // Git add and commit
        let _ = Command::new("git")
            .args(["add", "index.json"])
            .current_dir(&self.local_index_path)
            .output();
            
        let _ = Command::new("git")
            .args(["commit", "-m", "Initialize index repository"])
            .current_dir(&self.local_index_path)
            .output();
            
        Ok(())
    }
    
    async fn update_local_index(&self) -> Result<(), IndexError> {
        // 使用 git pull 更新本地索引
        let output = Command::new("git")
            .args(["pull", "--rebase"])
            .current_dir(&self.local_index_path)
            .output();
            
        // 忽略 pull 失败（可能是远程仓库不存在或网络问题）
        if let Ok(out) = output {
            if !out.status.success() {
                // 静默忽略，使用本地数据
            }
        }
        
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
        
        // Git add
        let _ = Command::new("git")
            .args(["add", "index.json"])
            .current_dir(&self.local_index_path)
            .output();
        
        // Git commit
        let _ = Command::new("git")
            .args(["commit", "-m", "Update index"])
            .current_dir(&self.local_index_path)
            .output();
        
        // Git push（使用系统的 Git 凭证）
        let push_output = Command::new("git")
            .args(["push", "-u", "origin", "main"])
            .current_dir(&self.local_index_path)
            .output();
            
        // 如果 main 分支不存在，尝试 master
        if let Ok(out) = push_output {
            if !out.status.success() {
                let _ = Command::new("git")
                    .args(["push", "-u", "origin", "master"])
                    .current_dir(&self.local_index_path)
                    .output();
            }
        }
        
        Ok(())
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
}
