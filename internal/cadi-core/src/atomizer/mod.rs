//! Smart Atomizer - Language-aware code parsing
//!
//! This module uses Tree-sitter for AST-based code analysis to extract
//! properly bounded code atoms with correct dependency tracking.
//!
//! ## Key Features
//!
//! - **Language-Aware Parsing**: Uses Tree-sitter grammars for accurate parsing
//! - **Symbol Resolution**: Resolves imports to content-addressed chunk IDs
//! - **Boundary Detection**: Never cuts functions in half
//! - **Dependency Tracking**: Tracks what each atom imports/exports
//!
//! ## Supported Languages (Phase 1)
//!
//! - Rust
//! - TypeScript/JavaScript
//! - Python
//!
//! ## Example
//!
//! ```rust,ignore
//! use cadi_core::atomizer::{Atomizer, AtomizerConfig};
//!
//! let atomizer = Atomizer::new(AtomizerConfig::default());
//!
//! // Parse a file into atoms
//! let atoms = atomizer.atomize_file(path, content)?;
//!
//! for atom in atoms {
//!     println!("Atom: {} ({:?})", atom.name, atom.kind);
//!     println!("  Defines: {:?}", atom.defines);
//!     println!("  References: {:?}", atom.references);
//! }
//! ```

pub mod config;
pub mod parser;
pub mod extractor;
pub mod resolver;
pub mod languages;

pub use config::{AtomizerConfig, LanguageConfig};
pub use parser::AstParser;
pub use extractor::{AtomExtractor, ExtractedAtom, AtomKind};
pub use resolver::{SymbolResolver, ResolvedImport, ImportedSymbol};
