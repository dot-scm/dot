//! dot - A Git proxy for managing hidden directories
//!
//! This library provides the core functionality for the dot CLI,
//! which manages hidden directories with version control through
//! multiple Git repositories.

pub mod error;
pub mod config;
pub mod index;
pub mod git_operations;
pub mod atomic;
pub mod repository;
pub mod setup;
pub mod github;

pub use error::*;
