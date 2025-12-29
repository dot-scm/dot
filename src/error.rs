use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Home directory not found")]
    HomeDirectoryNotFound,
    
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Failed to parse config file: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Organization not authorized in ~/.dot/dot.conf")]
    OrganizationNotAuthorized,
}

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("No default organization configured")]
    NoDefaultOrganization,
    
    #[error("Failed to get GitHub token")]
    GitHubTokenNotFound,
    
    #[error("GitHub API error: {0}")]
    GitHubError(#[from] octocrab::Error),
    
    #[error("Project already exists: {0}")]
    ProjectAlreadyExists(String),
    
    #[error("Failed to access index repository: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Failed to parse index data: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("git is not installed or not in PATH")]
    GitNotFound,
    
    #[error("Invalid git remote origin URL")]
    InvalidRemoteUrl,
    
    #[error("Project already exists: {0}")]
    ProjectAlreadyExists(String),
    
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),
    
    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Atomic operation failed")]
    AtomicOperationFailed,
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),
    
    #[error("Index error: {0}")]
    IndexError(#[from] IndexError),
}

#[derive(Error, Debug)]
pub enum OperationError {
    #[error("Operation failed: {message}")]
    ExecutionFailed { message: String },
    
    #[error("Rollback failed: {message}")]
    RollbackFailed { message: String },
    
    #[error("Atomic operation failed at {failed_operation}: {original_error}. {completed_count} operations were reverted")]
    AtomicOperationFailed {
        failed_operation: String,
        original_error: Box<dyn std::error::Error + Send + Sync>,
        completed_count: usize,
    },
    
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),
    
    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),
}

// 通用错误类型，用于主程序
#[derive(Error, Debug)]
pub enum DotError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Index repository error: {0}")]
    Index(#[from] IndexError),
    
    #[error("Repository operation error: {0}")]
    Repository(#[from] RepositoryError),
    
    #[error("Operation error: {0}")]
    Operation(#[from] OperationError),
}