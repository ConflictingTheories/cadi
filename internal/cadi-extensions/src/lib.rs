//! # CADI Extensions
//!
//! Plugin system for extending CADI's capabilities with custom atomizers,
//! build backends, registry plugins, and MCP tools.

pub mod loader;
pub mod manifest;
pub mod traits;
pub mod types;

pub use loader::ExtensionLoader;
pub use manifest::ExtensionManifest;
pub use traits::*;
pub use types::*;