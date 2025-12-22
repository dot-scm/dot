use std::process::{Command, Stdio};

use crate::error::Error;

/// Check if git is available in the system PATH
fn check_git_available() -> Result<(), Error> {
    Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| Error::GitNotFound)?;
    Ok(())
}

/// Execute a git command with the given arguments
/// 
/// This function acts as a transparent proxy to git, passing all arguments
/// directly and inheriting stdin/stdout/stderr for seamless interaction.
/// 
/// # Arguments
/// * `args` - The command and arguments to pass to git
/// 
/// # Returns
/// * `Ok(i32)` - The exit code from git
/// * `Err(Error)` - If git is not found or execution fails
pub fn execute(args: &[String]) -> Result<i32, Error> {
    // First check if git is available
    check_git_available()?;

    // Execute git with all provided arguments
    let status = Command::new("git")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_available() {
        // This test assumes git is installed on the system
        assert!(check_git_available().is_ok());
    }
}
