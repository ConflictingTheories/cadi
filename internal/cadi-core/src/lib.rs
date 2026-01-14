//! CADI Core - Core types and utilities for Content-Addressed Development Interface
//!
//! This crate provides the fundamental types and utilities used across the CADI ecosystem.
//!
//! ## Modules
//!
//! - `chunk` - Basic chunk types
//! - `manifest` - CADI manifest parsing
//! - `hash` - Content hashing utilities
//! - `atomic` - Atomic chunk system with aliases
//! - `smart_chunker` - Intelligent code analysis
//! - `graph` - Merkle DAG graph store for dependencies
//! - `atomizer` - Language-aware AST parsing (Phase 1)
//! - `rehydration` - Virtual view assembly (Phase 2)
//!
//! ## The Graph Store
//!
//! The `graph` module provides the core infrastructure for O(1) dependency queries:
//!
//! ```rust,ignore
//! use cadi_core::graph::{GraphStore, GraphNode, EdgeType};
//!
//! let store = GraphStore::open(".cadi/graph")?;
//! store.insert_node(&node)?;
//! let deps = store.get_dependencies("chunk:sha256:abc...")?;
//! ```
//!
//! ## The Atomizer
//!
//! The `atomizer` module provides language-aware code parsing:
//!
//! ```rust,ignore
//! use cadi_core::atomizer::{AtomExtractor, AtomizerConfig};
//!
//! let extractor = AtomExtractor::new("rust", AtomizerConfig::default());
//! let atoms = extractor.extract(source)?;
//! ```
//!
//! ## Virtual Views (Rehydration)
//!
//! The `rehydration` module assembles atoms into coherent contexts:
//!
//! ```rust,ignore
//! use cadi_core::rehydration::{RehydrationEngine, ViewConfig};
//!
//! let engine = RehydrationEngine::new(graph);
//! let view = engine.create_view(atom_ids, ViewConfig::default()).await?;
//! ```
//!
//! ## Ghost Import Resolver
//!
//! The `ghost` module automatically expands atom context with dependencies:
//!
//! ```rust,ignore
//! use cadi_core::ghost::{GhostResolver, ExpansionPolicy};
//!
//! let resolver = GhostResolver::new(graph);
//! let result = resolver.resolve(&atom_ids).await?;
//! ```

pub mod chunk;
pub mod manifest;
pub mod hash;
pub mod error;

pub use chunk::*;
pub use manifest::*;
pub use hash::*;
pub use error::*;

pub mod ast;
pub mod parser;
pub mod validator;

// New atomic chunk and smart chunking system
pub mod atomic;
pub mod smart_chunker;
pub mod project_analyzer;

pub use atomic::*;
pub use smart_chunker::*;
pub use project_analyzer::*;

// Semantic hashing and deduplication (Stage 2)
pub mod normalizer;
pub mod deduplication;

// Phase 0: Graph Store for O(1) dependency queries
pub mod graph;

// Phase 1: Language-aware AST parsing
pub mod atomizer;

// Phase 2: Virtual view assembly (rehydration)
pub mod rehydration;

// Phase 3: Ghost Import Resolver
pub mod ghost;
