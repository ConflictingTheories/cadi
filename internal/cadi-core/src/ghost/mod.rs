//! Ghost Import Resolver
//!
//! Automatically expands atom context to include necessary dependencies
//! so LLMs don't hallucinate missing types.

pub mod resolver;
pub mod policy;
pub mod analyzer;

pub use resolver::GhostResolver;
pub use policy::ExpansionPolicy;