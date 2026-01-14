//! CADI Builder - Build engine and transformation pipeline
//!
//! This crate provides the build engine that transforms source CADI chunks
//! through intermediate representations to final artifacts.

pub mod engine;
pub mod cache;
pub mod transform;
pub mod plan;
pub mod importer;
pub mod builder;
pub mod cbs;
pub mod build_spec;

pub use engine::*;
pub use cache::*;
pub use transform::*;
pub use plan::*;
pub use importer::*;
pub use builder::*;
pub use cbs::*;
pub use build_spec::{BuildSpec, ComponentSpec, GenerateComponent, ReuseComponent, SearchComponent, BuildSpecValidator, ReusePlan, GeneratePlan};
pub mod dependency_resolver;
