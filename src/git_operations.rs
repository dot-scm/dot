use crate::error::RepositoryError;
use git2::{Repository, Signature};
use std::path::Path;
use std::process::Command;

pub struct GitOperations;

impl GitOperations {
    /// 检查 git 是否可用
    pub fn check_git_available() -> Result<(), RepositoryError> {
        Command::new("git")
            .arg("--version")
            .output()
            .map_err(|_| RepositoryError::GitNotFound)?;
        Ok(())
    }
    
    /// 检查目录是否已初始化 git
    pub fn is_git_initialized<P: AsRef<Path>>(path: P) -> bool {
        Repository::open(path).is_ok()
    }
    
    /// 初始化 git 仓库
    pub fn init_repository<P: AsRef<Path>>(path: P) -> Result<Repository, RepositoryError> {
        Repository::init(path).map_err(RepositoryError::GitError)
    }
    
    /// 获取远程 origin URL
    pub fn get_remote_origin<P: AsRef<Path>>(path: P) -> Result<String, RepositoryError> {
        let repo = Repository::open(path)?;
        let remote = repo.find_remote("origin")?;
        
        remote.url()
            .ok_or(RepositoryError::InvalidRemoteUrl)
            .map(|s| s.to_string())
    }
    
    /// 生成 Repository Key
    pub fn generate_repository_key(remote_url: &str, directory: Option<&str>) -> Result<String, RepositoryError> {
        let base_key = Self::generate_base_key(remote_url)?;
        
        match directory {
            Some(dir) => Ok(format!("{}/{}", base_key, dir)),
            None => Ok(base_key),
        }
    }
    
    /// 生成基础 Repository Key
    pub fn generate_base_key(remote_url: &str) -> Result<String, RepositoryError> {
        // 移除协议部分 (everything before and including @)
        let after_at = if let Some(at_pos) = remote_url.rfind('@') {
            &remote_url[at_pos + 1..]
        } else {
            // 处理 HTTPS URL
            remote_url
                .strip_prefix("https://")
                .or_else(|| remote_url.strip_prefix("http://"))
                .unwrap_or(remote_url)
        };
        
        // 移除 .git 后缀
        let without_git = after_at.strip_suffix(".git")
            .unwrap_or(after_at);
            
        if without_git.is_empty() {
            return Err(RepositoryError::InvalidRemoteUrl);
        }
        
        Ok(without_git.to_string())
    }
    
    /// 获取当前 git 用户
    pub fn get_git_user<P: AsRef<Path>>(path: P) -> Result<String, RepositoryError> {
        let repo = Repository::open(path)?;
        let config = repo.config()?;
        
        config.get_string("user.name")
            .or_else(|_| config.get_string("user.email"))
            .map_err(|_| RepositoryError::GitError(git2::Error::from_str("No git user configured")))
    }
    
    /// 添加文件到 git index
    pub fn add_files<P: AsRef<Path>>(repo_path: P, files: &[String]) -> Result<(), RepositoryError> {
        let repo = Repository::open(repo_path)?;
        let mut index = repo.index()?;
        
        for file in files {
            let file_path = Path::new(file);
            if file_path.exists() {
                index.add_path(file_path)?;
            }
        }
        
        index.write()?;
        Ok(())
    }
    
    /// 添加所有更改到 git index
    pub fn add_all<P: AsRef<Path>>(repo_path: P) -> Result<(), RepositoryError> {
        let repo = Repository::open(repo_path)?;
        let mut index = repo.index()?;
        
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }
    
    /// 提交更改
    pub fn commit<P: AsRef<Path>>(repo_path: P, message: &str) -> Result<git2::Oid, RepositoryError> {
        let repo = Repository::open(repo_path)?;
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        
        let signature = Self::get_signature(&repo)?;
        
        // 获取 HEAD commit 作为 parent（如果存在）
        let parent_commit = match repo.head() {
            Ok(head) => {
                let oid = head.target().ok_or(RepositoryError::GitError(
                    git2::Error::from_str("HEAD has no target")
                ))?;
                Some(repo.find_commit(oid)?)
            }
            Err(_) => None, // 首次提交
        };
        
        let parents: Vec<&git2::Commit> = parent_commit.as_ref().map(|c| vec![c]).unwrap_or_default();
        
        let commit_id = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;
        
        Ok(commit_id)
    }
    
    /// 推送到远程仓库
    pub fn push<P: AsRef<Path>>(repo_path: P) -> Result<(), RepositoryError> {
        let path = repo_path.as_ref();
        
        // 获取当前分支名
        let repo = Repository::open(path)?;
        let head = repo.head()?;
        let branch_name = head.shorthand().unwrap_or("main");
        
        // 使用 git 命令行推送，更可靠地处理 SSH 认证和首次推送
        let output = std::process::Command::new("git")
            .args(["-C", path.to_str().unwrap_or("."), "push", "-u", "origin", branch_name])
            .output()
            .map_err(|e| RepositoryError::IoError(e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // 如果是 "everything up-to-date" 或类似消息，不算错误
            if stderr.contains("Everything up-to-date") || stderr.contains("up to date") {
                return Ok(());
            }
            return Err(RepositoryError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("git push failed: {}", stderr)
            )));
        }
        
        Ok(())
    }
    
    /// 获取 git 状态
    pub fn get_status<P: AsRef<Path>>(repo_path: P) -> Result<String, RepositoryError> {
        let repo = Repository::open(repo_path)?;
        let statuses = repo.statuses(None)?;
        
        let mut status_lines = Vec::new();
        
        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("unknown");
            
            if status.contains(git2::Status::INDEX_NEW) {
                status_lines.push(format!("A  {}", path));
            } else if status.contains(git2::Status::INDEX_MODIFIED) {
                status_lines.push(format!("M  {}", path));
            } else if status.contains(git2::Status::INDEX_DELETED) {
                status_lines.push(format!("D  {}", path));
            } else if status.contains(git2::Status::WT_NEW) {
                status_lines.push(format!("?? {}", path));
            } else if status.contains(git2::Status::WT_MODIFIED) {
                status_lines.push(format!(" M {}", path));
            } else if status.contains(git2::Status::WT_DELETED) {
                status_lines.push(format!(" D {}", path));
            }
        }
        
        if status_lines.is_empty() {
            Ok("nothing to commit, working tree clean".to_string())
        } else {
            Ok(status_lines.join("\n"))
        }
    }
    
    /// 克隆仓库
    pub fn clone_repository(url: &str, path: &Path) -> Result<Repository, RepositoryError> {
        Repository::clone(url, path).map_err(RepositoryError::GitError)
    }
    
    /// 获取 git signature
    fn get_signature(repo: &Repository) -> Result<Signature<'_>, RepositoryError> {
        let config = repo.config()?;
        
        let name = config.get_string("user.name")
            .unwrap_or_else(|_| "dot-cli".to_string());
        let email = config.get_string("user.email")
            .unwrap_or_else(|_| "dot-cli@example.com".to_string());
            
        Signature::now(&name, &email).map_err(RepositoryError::GitError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_generate_base_key_ssh() {
        let url = "git@github.com:user/repo.git";
        let key = GitOperations::generate_base_key(url).unwrap();
        assert_eq!(key, "github.com:user/repo");
    }
    
    #[test]
    fn test_generate_base_key_https() {
        let url = "https://github.com/user/repo.git";
        let key = GitOperations::generate_base_key(url).unwrap();
        assert_eq!(key, "github.com/user/repo");
    }
    
    #[test]
    fn test_generate_base_key_no_git_suffix() {
        let url = "git@github.com:user/repo";
        let key = GitOperations::generate_base_key(url).unwrap();
        assert_eq!(key, "github.com:user/repo");
    }
    
    #[test]
    fn test_generate_repository_key_with_directory() {
        let url = "git@github.com:user/repo.git";
        let key = GitOperations::generate_repository_key(url, Some(".kiro")).unwrap();
        assert_eq!(key, "github.com:user/repo/.kiro");
    }
    
    #[test]
    fn test_generate_repository_key_without_directory() {
        let url = "git@github.com:user/repo.git";
        let key = GitOperations::generate_repository_key(url, None).unwrap();
        assert_eq!(key, "github.com:user/repo");
    }
    
    #[test]
    fn test_invalid_remote_url() {
        let url = "";
        let result = GitOperations::generate_base_key(url);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_git_operations_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        // 初始化仓库
        let repo = GitOperations::init_repository(repo_path).unwrap();
        assert!(GitOperations::is_git_initialized(repo_path));
        
        // 创建一个测试文件
        let test_file = repo_path.join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        
        // 添加文件
        GitOperations::add_files(repo_path, &["test.txt".to_string()]).unwrap();
        
        // 获取状态
        let status = GitOperations::get_status(repo_path).unwrap();
        assert!(status.contains("test.txt"));
    }
}