//! Core library for Unduler.
//!
//! This crate provides the main orchestration logic for version management
//! and changelog generation.

mod error;
mod files;
mod pipeline;
mod release;
mod version;

pub use error::{CoreError, CoreResult};
pub use files::{FileResult, FileUpdateError, read_version_from_file, update_version_file};
pub use pipeline::Pipeline;
pub use release::ReleaseManager;
pub use version::VersionManager;
