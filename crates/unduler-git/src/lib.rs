//! Git abstraction layer for Unduler.
//!
//! This crate provides Git operations:
//! - Repository management
//! - Commit retrieval
//! - Tag management

mod error;
mod repository;

pub use error::{GitError, GitResult};
pub use repository::Repository;
