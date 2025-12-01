//! Core library for Unduler.
//!
//! This crate provides the main orchestration logic for version management
//! and changelog generation.

mod error;
mod pipeline;
mod release;
mod version;

pub use error::{CoreError, CoreResult};
pub use pipeline::Pipeline;
pub use release::ReleaseManager;
pub use version::VersionManager;
