//! Language-specific atomizer configurations and utilities
//!
//! This module will contain Tree-sitter query files and language-specific
//! extraction logic when the `ast-parsing` feature is enabled.

pub mod rust;
pub mod typescript;
pub mod python;
pub mod c;
pub mod csharp;
pub mod css;
pub mod jsx;
pub mod tsx;
pub mod html;
pub mod glsl;

pub use rust::RustAtomizer;
pub use typescript::TypeScriptAtomizer;
pub use python::PythonAtomizer;
pub use c::CAtomizer;
pub use csharp::CSharpAtomizer;
pub use css::CSSAtomizer;
pub use jsx::JSXAtomizer;
pub use tsx::TSXAtomizer;
pub use html::HtmlAtomizer;
pub use glsl::GLSLAtomizer;
