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
//! - `traits`: Core traits for WebIDL interfaces
//! - `prototype_list`: Interface prototype identifiers (equivalent to SpiderMonkey's PrototypeList)
//! - `register_bindings`: Interface registration system (equivalent to SpiderMonkey's RegisterBindings)

pub mod types;
pub mod interface;
pub mod traits;
pub mod prototype_list;
pub mod register_bindings;

pub use types::*;
pub use interface::*;
pub use traits::*;
pub use prototype_list::{
    ID, Constructor, InterfaceChain, 
    MAX_PROTO_CHAIN_LENGTH,
    proto_count, constructor_count, proto_or_iface_length,
    allocate_proto_id, allocate_constructor_id,
    proto_id_to_name, register_interface_name,
    well_known,
};
pub use register_bindings::{
    InterfaceDescriptor, RegisterBinding, ProtoAndIfaceCache,
    RegisterFn, CreatePrototypeFn, CreateConstructorFn,
    register_interface, get_interface_by_name,
    get_interface_by_proto_id, get_interface_by_constructor_id,
    get_all_interfaces, register_all_bindings,
    register_window_bindings, register_worker_bindings,
    init as init_bindings,
};
