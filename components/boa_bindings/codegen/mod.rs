//! WebIDL Code Generation for Boa JavaScript Engine
//!
//! This module provides a Rust-based WebIDL code generator that produces
//! bindings for the Boa JavaScript engine, replacing the Python-based
//! SpiderMonkey generator.
//!
//! # Architecture
//!
//! The codegen module is organized as follows:
//! - `types`: WebIDL type mappings to Boa types
//! - `interface`: Interface generation (attributes, methods, constants)
//! - `callback`: Callback function generation
//! - `dictionary`: Dictionary generation
//! - `template`: Code templates for generated bindings

pub mod types;
pub mod interface;
pub mod traits;

pub use types::*;
pub use interface::*;
pub use traits::*;
