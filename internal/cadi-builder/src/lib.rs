//! CADI Builder - Build engine and transformation pipeline
//!
//! This crate provides the build engine that transforms source CADI chunks
//! through intermediate representations to final artifacts.

pub mod engine;
pub mod cache;
pub mod transform;
pub mod plan;

pub use engine::*;
pub use cache::*;
pub use transform::*;
pub use plan::*;
