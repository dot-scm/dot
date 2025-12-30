use crate::error::RepositoryError;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// GitHub API 客户端
pub struct GitHubClient {
    token: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateRepoRequest {
    name: String,
    description: String,
    private: bool,
    auto_init: bool,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
    
    /// 创建远程仓库
    /// 优先使用 GitHub API（如果有 token），否则使用 gh CLI
    pub async fn create_repository(
        &self,
        org: &str,
        repo_name: &str,
        description: &str,
    ) -> Result<String, RepositoryError> {
        // 优先使用 GitHub API
        if let Some(token) = &self.token {
            return self.create_repo_via_api(org, repo_name, description, token).await;
        }
        
        // 回退到 gh CLI
        self.create_repo_via_gh_cli(org, repo_name, description).await
    }
    
    /// 使用 GitHub API 创建仓库
    async fn create_repo_via_api(
        &self,
        org: &str,
        repo_name: &str,
        description: &str,
        token: &str,
    ) -> Result<String, RepositoryError> {
        let client = reqwest::Client::new();
        
        let request_body = CreateRepoRequest {
            name: repo_name.to_string(),
            description: description.to_string(),
            private: true,
            auto_init: true,
        };
        
        // 判断是用户还是组织
        // 如果组织名和用户名相同，使用 /user/repos
        // 否则使用 /orgs/{org}/repos
        let url = format!("https://api.github.com/orgs/{}/repos", org);
        
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "dot-cli")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| RepositoryError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to send request: {}", e)
            )))?;
        
        let status = response.status();
        
        if status.is_success() {
            let remote_url = format!("git@github.com:{}/{}.git", org, repo_name);
            return Ok(remote_url);
        }
        
        // 如果组织 API 失败，尝试用户 API
        if status.as_u16() == 404 {
            return self.create_repo_for_user(repo_name, description, token).await;
        }
        
        // 处理错误
        let error_text = response.text().await.unwrap_or_default();
        
        // 检查是否是仓库已存在
        if error_text.contains("already exists") || status.as_u16() == 422 {
            let remote_url = format!("git@github.com:{}/{}.git", org, repo_name);
            return Ok(remote_url);
        }
        
        Err(RepositoryError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("GitHub API error ({}): {}", status, error_text)
        )))
    }
    
    /// 为用户创建仓库（当组织不存在时）
    async fn create_repo_for_user(
        &self,
        repo_name: &str,
        description: &str,
        token: &str,
    ) -> Result<String, RepositoryError> {
        let client = reqwest::Client::new();
        
        let request_body = CreateRepoRequest {
            name: repo_name.to_string(),
            description: description.to_string(),
            private: true,
            auto_init: true,
        };
        
        let url = "https://api.github.com/user/repos";
        
        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "dot-cli")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| RepositoryError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to send request: {}", e)
            )))?;
        
        let status = response.status();
        
        if status.is_success() {
            // 获取用户名
            let user = self.get_authenticated_user(token).await?;
            let remote_url = format!("git@github.com:{}/{}.git", user, repo_name);
            return Ok(remote_url);
        }
        
        let error_text = response.text().await.unwrap_or_default();
        
        // 检查是否是仓库已存在
        if error_text.contains("already exists") || status.as_u16() == 422 {
            let user = self.get_authenticated_user(token).await?;
            let remote_url = format!("git@github.com:{}/{}.git", user, repo_name);
            return Ok(remote_url);
        }
        
        Err(RepositoryError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("GitHub API error ({}): {}", status, error_text)
        )))
    }
    
    /// 获取认证用户名
    async fn get_authenticated_user(&self, token: &str) -> Result<String, RepositoryError> {
        let client = reqwest::Client::new();
        
        let response = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "dot-cli")
            .send()
            .await
            .map_err(|e| RepositoryError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get user: {}", e)
            )))?;
        
        if !response.status().is_success() {
            return Err(RepositoryError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to get authenticated user"
            )));
        }
        
        #[derive(Deserialize)]
        struct User {
            login: String,
        }
        
        let user: User = response.json().await.map_err(|e| {
            RepositoryError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to parse user response: {}", e)
            ))
        })?;
        
        Ok(user.login)
    }
    
    /// 使用 gh CLI 创建仓库（回退方案）
    async fn create_repo_via_gh_cli(
        &self,
        org: &str,
        repo_name: &str,
        description: &str,
    ) -> Result<String, RepositoryError> {
        let output = Command::new("gh")
            .args([
                "repo", "create",
                &format!("{}/{}", org, repo_name),
                "--private",
                "--description", description,
            ])
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    let remote_url = format!("git@github.com:{}/{}.git", org, repo_name);
                    return Ok(remote_url);
                }
                
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                // 如果仓库已存在，返回成功
                if stderr.contains("already exists") {
                    let remote_url = format!("git@github.com:{}/{}.git", org, repo_name);
                    return Ok(remote_url);
                }
                
                Err(RepositoryError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("gh CLI error: {}. Please run 'gh auth login' or set github_token in ~/.dot/dot.conf", stderr.trim())
                )))
            }
            Err(e) => {
                Err(RepositoryError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("GitHub CLI (gh) not available: {}. Please set github_token in ~/.dot/dot.conf or install gh CLI", e)
                )))
            }
        }
    }
    
    /// 删除远程仓库
    pub async fn delete_repository(
        &self,
        org: &str,
        repo_name: &str,
    ) -> Result<(), RepositoryError> {
        // 优先使用 GitHub API
        if let Some(token) = &self.token {
            return self.delete_repo_via_api(org, repo_name, token).await;
        }
        
        // 回退到 gh CLI
        self.delete_repo_via_gh_cli(org, repo_name).await
    }
    
    async fn delete_repo_via_api(
        &self,
        org: &str,
        repo_name: &str,
        token: &str,
    ) -> Result<(), RepositoryError> {
        let client = reqwest::Client::new();
        let url = format!("https://api.github.com/repos/{}/{}", org, repo_name);
        
        let response = client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "dot-cli")
            .send()
            .await
            .map_err(|e| RepositoryError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to delete repository: {}", e)
            )))?;
        
        if response.status().is_success() || response.status().as_u16() == 404 {
            return Ok(());
        }
        
        let error_text = response.text().await.unwrap_or_default();
        Err(RepositoryError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to delete repository: {}", error_text)
        )))
    }
    
    async fn delete_repo_via_gh_cli(
        &self,
        org: &str,
        repo_name: &str,
    ) -> Result<(), RepositoryError> {
        let output = Command::new("gh")
            .args([
                "repo", "delete",
                &format!("{}/{}", org, repo_name),
                "--yes",
            ])
            .output();
            
        if let Ok(result) = output {
            if result.status.success() {
                return Ok(());
            }
        }
        
        // 删除失败不是致命错误
        Ok(())
    }
}
