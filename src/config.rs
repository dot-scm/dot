use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct DotConfig {
    pub authorized_organizations: Vec<String>,
    pub default_organization: Option<String>,
}

impl Default for DotConfig {
    fn default() -> Self {
        Self {
            authorized_organizations: vec![],
            default_organization: None,
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: DotConfig,
}

impl ConfigManager {
    pub async fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_file_path()?;
        
        let config = if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            serde_json::from_str(&content)?
        } else {
            // 创建默认配置文件
            let default_config = DotConfig::default();
            Self::ensure_config_dir(&config_path).await?;
            let content = serde_json::to_string_pretty(&default_config)?;
            tokio::fs::write(&config_path, content).await?;
            default_config
        };
        
        Ok(Self { config_path, config })
    }
    
    pub fn is_organization_authorized(&self, org: &str) -> bool {
        self.config.authorized_organizations.contains(&org.to_string())
    }
    
    pub fn get_default_organization(&self) -> Option<&String> {
        self.config.default_organization.as_ref()
    }
    
    pub async fn add_organization(&mut self, org: String) -> Result<(), ConfigError> {
        if !self.config.authorized_organizations.contains(&org) {
            self.config.authorized_organizations.push(org);
            self.save().await?;
        }
        Ok(())
    }
    
    pub async fn remove_organization(&mut self, org: &str) -> Result<(), ConfigError> {
        self.config.authorized_organizations.retain(|o| o != org);
        self.save().await
    }
    
    pub async fn set_default_organization(&mut self, org: String) -> Result<(), ConfigError> {
        if !self.is_organization_authorized(&org) {
            return Err(ConfigError::OrganizationNotAuthorized);
        }
        self.config.default_organization = Some(org);
        self.save().await
    }
    
    async fn save(&self) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(&self.config)?;
        tokio::fs::write(&self.config_path, content).await?;
        Ok(())
    }
    
    fn config_file_path() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::HomeDirectoryNotFound)?;
        Ok(home.join(".dot").join("dot.conf"))
    }
    
    async fn ensure_config_dir(config_path: &PathBuf) -> Result<(), ConfigError> {
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;
    
    #[tokio::test]
    async fn test_default_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_path_buf();
        
        // 临时设置 HOME 环境变量
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", &temp_home);
        
        let config = ConfigManager::load().await.unwrap();
        assert!(config.config.authorized_organizations.is_empty());
        assert!(config.config.default_organization.is_none());
        
        // 恢复原始 HOME 环境变量
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }
    
    #[tokio::test]
    async fn test_organization_management() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_path_buf();
        
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", &temp_home);
        
        let mut config = ConfigManager::load().await.unwrap();
        
        // 测试添加组织
        config.add_organization("test-org".to_string()).await.unwrap();
        assert!(config.is_organization_authorized("test-org"));
        
        // 测试重复添加
        config.add_organization("test-org".to_string()).await.unwrap();
        assert_eq!(config.config.authorized_organizations.len(), 1);
        
        // 测试移除组织
        config.remove_organization("test-org").await.unwrap();
        assert!(!config.is_organization_authorized("test-org"));
        
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }
}