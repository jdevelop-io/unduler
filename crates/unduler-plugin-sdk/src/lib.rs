//! SDK for building Unduler plugins as `WebAssembly` components.
//!
//! This crate provides macros and types to easily create WASM plugins
//! that can be loaded by Unduler at runtime.
//!
//! # Example: Parser Plugin
//!
//! ```ignore
//! use unduler_plugin_sdk::parser::*;
//!
//! struct MyParser;
//!
//! impl Guest for MyParser {
//!     fn info() -> PluginInfo {
//!         PluginInfo {
//!             name: "my-parser".to_string(),
//!             version: "0.1.0".to_string(),
//!             description: "My custom parser".to_string(),
//!             plugin_type: PluginType::Parser,
//!         }
//!     }
//!
//!     fn parse(commit: RawCommit) -> Option<ParsedCommit> {
//!         // Parse logic here
//!         None
//!     }
//!
//!     fn can_parse(commit: RawCommit) -> bool {
//!         false
//!     }
//! }
//!
//! export!(MyParser);
//! ```

/// Parser plugin bindings.
pub mod parser {
    wit_bindgen::generate!({
        world: "unduler-parser",
        path: "wit",
        pub_export_macro: true,
        export_macro_name: "export",
    });

    pub use self::exports::unduler::plugin::parser::Guest;
    pub use self::unduler::plugin::types::*;
}

/// Bumper plugin bindings.
pub mod bumper {
    wit_bindgen::generate!({
        world: "unduler-bumper",
        path: "wit",
        pub_export_macro: true,
        export_macro_name: "export",
    });

    pub use self::exports::unduler::plugin::bumper::Guest;
    pub use self::unduler::plugin::types::*;
}

/// Formatter plugin bindings.
pub mod formatter {
    wit_bindgen::generate!({
        world: "unduler-formatter",
        path: "wit",
        pub_export_macro: true,
        export_macro_name: "export",
    });

    pub use self::exports::unduler::plugin::formatter::Guest;
    pub use self::unduler::plugin::types::*;
}

/// Hook plugin bindings.
pub mod hook {
    wit_bindgen::generate!({
        world: "unduler-hook",
        path: "wit",
        pub_export_macro: true,
        export_macro_name: "export",
    });

    pub use self::exports::unduler::plugin::hook::Guest;
    pub use self::unduler::plugin::types::*;
}
