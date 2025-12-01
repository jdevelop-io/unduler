//! Commit types for Unduler.
//!
//! This crate provides the core commit types used throughout Unduler:
//! - [`RawCommit`]: A commit as retrieved from Git
//! - [`ParsedCommit`]: A commit after parsing by a parser plugin

mod parsed;
mod raw;

pub use parsed::ParsedCommit;
pub use raw::RawCommit;
