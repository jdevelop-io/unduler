//! WASM runtime for Unduler plugins.
//!
//! This crate provides the infrastructure to load and execute WASM plugins
//! using the wasmtime runtime with Component Model support.

pub mod bumper;
pub mod engine;
pub mod error;
pub mod formatter;
pub mod hook;
pub mod parser;

pub use bumper::WasmBumper;
pub use engine::WasmEngine;
pub use error::{WasmError, WasmResult};
pub use formatter::WasmFormatter;
pub use hook::WasmHook;
pub use parser::WasmParser;
