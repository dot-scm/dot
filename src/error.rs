use std::fmt;

/// Error types for the dot CLI
#[derive(Debug)]
pub enum Error {
    /// Git executable is not found in PATH
    GitNotFound,
    /// Failed to execute git command
    ExecutionFailed(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::GitNotFound => write!(f, "git is not installed or not in PATH"),
            Error::ExecutionFailed(e) => write!(f, "failed to execute git: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::GitNotFound => None,
            Error::ExecutionFailed(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::ExecutionFailed(err)
    }
}
