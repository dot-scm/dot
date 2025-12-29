use crate::error::OperationError;
use crate::git_operations::GitOperations;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

#[async_trait::async_trait]
pub trait Operation: Send + Sync {
    async fn execute(&self) -> Result<(), OperationError>;
    async fn rollback(&self) -> Result<(), OperationError>;
    fn description(&self) -> String;
}

pub struct AtomicOperations {
    operations: Vec<Box<dyn Operation>>,
    atomic: bool,
}

impl AtomicOperations {
    pub fn new(no_atomic: bool) -> Self {
        Self {
            operations: Vec::new(),
            atomic: !no_atomic,
        }
    }
    
    pub fn add_operation(&mut self, operation: Box<dyn Operation>) {
        self.operations.push(operation);
    }
    
    pub async fn execute(&self) -> Result<(), OperationError> {
        if !self.atomic {
            // 非原子模式：继续执行所有操作，即使有失败
            for operation in &self.operations {
                if let Err(e) = operation.execute().await {
                    eprintln!("Operation failed: {} - {}", operation.description(), e);
                }
            }
            return Ok(());
        }
        
        // 原子模式：全部成功或全部回滚
        let mut completed_operations = Vec::new();
        
        for operation in &self.operations {
            match operation.execute().await {
                Ok(_) => completed_operations.push(operation),
                Err(e) => {
                    // 回滚已完成的操作
                    for completed in completed_operations.iter().rev() {
                        if let Err(rollback_err) = completed.rollback().await {
                            eprintln!("Rollback failed for {}: {}", 
                                completed.description(), rollback_err);
                        }
                    }
                    return Err(OperationError::AtomicOperationFailed {
                        failed_operation: operation.description(),
                        original_error: Box::new(e),
                        completed_count: completed_operations.len(),
                    });
                }
            }
        }
        
        Ok(())
    }
}

pub struct AddOperation {
    repository_path: PathBuf,
    files: Vec<String>,
    staged_files: Arc<AsyncMutex<Vec<String>>>,
}

impl AddOperation {
    pub fn new(repository_path: PathBuf, files: Vec<String>) -> Self {
        Self {
            repository_path,
            files,
            staged_files: Arc::new(AsyncMutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl Operation for AddOperation {
    async fn execute(&self) -> Result<(), OperationError> {
        let mut staged = self.staged_files.lock().await;
        
        // 检查哪些文件实际存在并需要添加
        let mut files_to_add = Vec::new();
        for file in &self.files {
            let file_path = if file == "." {
                // 处理 "." 的情况，添加所有文件
                GitOperations::add_all(&self.repository_path)?;
                staged.push(".".to_string());
                return Ok(());
            } else {
                self.repository_path.join(file)
            };
            
            if file_path.exists() {
                files_to_add.push(file.clone());
            }
        }
        
        if !files_to_add.is_empty() {
            GitOperations::add_files(&self.repository_path, &files_to_add)?;
            staged.extend(files_to_add);
        }
        
        Ok(())
    }
    
    async fn rollback(&self) -> Result<(), OperationError> {
        // Git add 的回滚比较复杂，这里简化处理
        // 在实际应用中，可能需要保存操作前的 index 状态
        let staged = self.staged_files.lock().await;
        
        if !staged.is_empty() {
            // 重置 index 到 HEAD
            let repo = git2::Repository::open(&self.repository_path)?;
            let head = repo.head()?.peel_to_commit()?;
            let tree = head.tree()?;
            repo.reset(tree.as_object(), git2::ResetType::Mixed, None)?;
        }
        
        Ok(())
    }
    
    fn description(&self) -> String {
        format!("Add files to {}", self.repository_path.display())
    }
}

pub struct CommitOperation {
    repository_path: PathBuf,
    message: String,
    commit_id: Arc<AsyncMutex<Option<git2::Oid>>>,
}

impl CommitOperation {
    pub fn new(repository_path: PathBuf, message: String) -> Self {
        Self {
            repository_path,
            message,
            commit_id: Arc::new(AsyncMutex::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl Operation for CommitOperation {
    async fn execute(&self) -> Result<(), OperationError> {
        let commit_id = GitOperations::commit(&self.repository_path, &self.message)?;
        let mut stored_id = self.commit_id.lock().await;
        *stored_id = Some(commit_id);
        Ok(())
    }
    
    async fn rollback(&self) -> Result<(), OperationError> {
        let stored_id = self.commit_id.lock().await;
        
        if let Some(commit_id) = *stored_id {
            // 回滚到上一个 commit
            let repo = git2::Repository::open(&self.repository_path)?;
            let commit = repo.find_commit(commit_id)?;
            
            let parent_opt = commit.parents().next();
            if let Some(parent) = parent_opt {
                repo.reset(parent.as_object(), git2::ResetType::Hard, None)?;
            } else {
                // 如果是第一个 commit，创建一个空的 tree
                let tree_builder = repo.treebuilder(None)?;
                let empty_tree_id = tree_builder.write()?;
                let empty_tree = repo.find_tree(empty_tree_id)?;
                repo.reset(
                    empty_tree.as_object(), 
                    git2::ResetType::Hard, 
                    None
                )?;
            }
        }
        
        Ok(())
    }
    
    fn description(&self) -> String {
        format!("Commit to {}", self.repository_path.display())
    }
}

pub struct PushOperation {
    repository_path: PathBuf,
    pushed: Arc<AsyncMutex<bool>>,
}

impl PushOperation {
    pub fn new(repository_path: PathBuf) -> Self {
        Self {
            repository_path,
            pushed: Arc::new(AsyncMutex::new(false)),
        }
    }
}

#[async_trait::async_trait]
impl Operation for PushOperation {
    async fn execute(&self) -> Result<(), OperationError> {
        GitOperations::push(&self.repository_path)?;
        let mut pushed = self.pushed.lock().await;
        *pushed = true;
        Ok(())
    }
    
    async fn rollback(&self) -> Result<(), OperationError> {
        let pushed = self.pushed.lock().await;
        
        if *pushed {
            // Push 操作的回滚比较复杂，通常需要 force push 回滚
            // 这里简化处理，实际应用中需要更复杂的逻辑
            return Err(OperationError::RollbackFailed {
                message: "Cannot rollback push operation automatically".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn description(&self) -> String {
        format!("Push {}", self.repository_path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[tokio::test]
    async fn test_atomic_operations_success() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        
        // 初始化 git 仓库
        GitOperations::init_repository(&repo_path).unwrap();
        
        // 创建测试文件
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content").unwrap();
        
        let mut operations = AtomicOperations::new(false);
        operations.add_operation(Box::new(AddOperation::new(
            repo_path.clone(),
            vec!["test.txt".to_string()],
        )));
        
        let result = operations.execute().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_atomic_operations_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        
        // 初始化 git 仓库
        GitOperations::init_repository(&repo_path).unwrap();
        
        let mut operations = AtomicOperations::new(false);
        
        // 添加一个会成功的操作
        operations.add_operation(Box::new(AddOperation::new(
            repo_path.clone(),
            vec!["nonexistent.txt".to_string()],
        )));
        
        // 添加一个会失败的操作（提交空的更改）
        operations.add_operation(Box::new(CommitOperation::new(
            repo_path.clone(),
            "test commit".to_string(),
        )));
        
        let result = operations.execute().await;
        // 应该失败，因为没有文件可以提交
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_non_atomic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        
        // 初始化 git 仓库
        GitOperations::init_repository(&repo_path).unwrap();
        
        let mut operations = AtomicOperations::new(true); // 非原子模式
        
        operations.add_operation(Box::new(AddOperation::new(
            repo_path.clone(),
            vec!["nonexistent.txt".to_string()],
        )));
        
        let result = operations.execute().await;
        // 非原子模式下应该成功，即使某些操作失败
        assert!(result.is_ok());
    }
}