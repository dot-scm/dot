//! dot - A git command proxy CLI tool
//!
//! This library provides the core functionality for the dot CLI,
//! which acts as a transparent proxy for git commands.

pub mod error;
pub mod git_proxy;

pub use error::Error;
pub use git_proxy::execute;
