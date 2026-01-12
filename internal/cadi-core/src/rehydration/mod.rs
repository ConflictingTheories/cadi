//! Rehydration Engine
//!
//! Dynamically assembles atoms into syntactically valid "virtual files"
//! that exist only in memory, optimized for LLM consumption.
//!
//! This is the **killer feature** for agents - they get exactly the code
//! they need, nothing more, nothing less.
//!
//! ## Example
//!
//! ```rust,ignore
//! use cadi_core::rehydration::{RehydrationEngine, ViewConfig};
//!
//! let engine = RehydrationEngine::new(graph_store);
//!
//! // Create a view from specific atoms
//! let view = engine.create_view(vec![
//!     "chunk:sha256:abc123".to_string(),
//!     "chunk:sha256:def456".to_string(),
//! ], ViewConfig::default()).await?;
//!
//! // The view contains syntactically valid code
//! println!("{}", view.source);
//! println!("Tokens: ~{}", view.token_estimate);
//! ```

pub mod engine;
pub mod view;
pub mod config;
pub mod assembler;

pub use engine::RehydrationEngine;
pub use view::{VirtualView, ViewFragment};
pub use config::ViewConfig;
pub use assembler::Assembler;
