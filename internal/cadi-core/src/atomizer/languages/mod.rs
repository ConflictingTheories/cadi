//! Language-specific atomizer configurations and utilities
//!
//! This module will contain Tree-sitter query files and language-specific
//! extraction logic when the `ast-parsing` feature is enabled.

pub mod rust;
pub mod typescript;
pub mod python;

pub use rust::RustAtomizer;
pub use typescript::TypeScriptAtomizer;
pub use python::PythonAtomizer;
