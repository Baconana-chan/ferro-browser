// MIT License
//
// Copyright (c) 2025-2026 Ferro Browser Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! # Boa JavaScript Engine Bindings for Ferro Browser
//!
//! **boa_bindings** is the default JavaScript engine integration for Ferro Browser.
//! It provides a pure Rust implementation using the [Boa](https://boajs.dev/) engine,
//! eliminating the need for C++ toolchains and complex build dependencies.
//!
//! ## Why Boa?
//!
//! - **Pure Rust**: No C++ dependencies, faster compile times, easier debugging
//! - **Cross-platform**: Works on any target Rust supports (including WASM)
//! - **Modern ECMAScript**: 94% conformance with ECMAScript specification
//! - **Memory Safe**: Leverages Rust's ownership model for GC safety
//!
//! ## Module Overview
//!
//! | Module | Description |
//! |--------|-------------|
//! | `runtime` | JavaScript Context and evaluation |
//! | `gc` | Garbage collection integration with DOM |
//! | `reflector` | DOM object reflection (Rust ↔ JS binding) |
//! | `root` | Smart pointers for DOM objects (Dom, DomRoot) |
//! | `weakref` | Weak references for GC |
//! | `event_loop` | Task/microtask scheduling per HTML spec |
//! | `builtins` | Web APIs (setTimeout, fetch, console, etc.) |
//! | `codegen` | WebIDL bindings infrastructure |
//!
//! ## Feature Flags
//!
//! Enable/disable Web APIs via Cargo features:
//! - `console` - Console API (default)
//! - `fetch` - Fetch API with AbortController (default)
//! - `timers` - setTimeout/setInterval (default)
//! - `webgl` - WebGL/WebGL2 support
//! - `web_audio` - Web Audio API
//! - `gc_debug` - GC safety debug assertions
//!
//! ## Example
//!
//! ```rust,no_run
//! use boa_bindings::{JsRuntime, JsResult};
//!
//! fn main() -> JsResult<()> {
//!     let mut runtime = JsRuntime::new();
//!     let result = runtime.eval("1 + 2")?;
//!     println!("Result: {:?}", result);
//!     Ok(())
//! }
//! ```

pub mod runtime;
pub mod gc;
pub mod conversions;
pub mod error;
pub mod reflector;
pub mod builtins;
pub mod codegen;
pub mod dom;
pub mod js_compat;
pub mod root;
pub mod trace;
pub mod weakref;
pub mod cell;
pub mod dom_conversions;
pub mod settings_stack;
pub mod gc_safety;
pub mod event_loop;

// Re-export key Boa types
pub use boa_engine::{
    Context, JsResult, JsValue, JsError, JsNativeError,
    Source, JsString,
    object::JsObject,
    property::Attribute,
};

pub use boa_gc::{Finalize, Trace, Gc, GcRefCell};

// Re-export js_compat as 'js' for SpiderMonkey API compatibility
// This allows existing code using `use js::*` to work with Boa
pub use js_compat as js;

// Re-export GC safety utilities
pub use gc_safety::{
    init_gc_debug, gc_debug_enabled, record_allocation, record_gc_cycle,
    is_in_trace, is_in_finalize, is_gc_running, root_depth,
    assert_rooted, assert_not_in_gc, assert_traced, assert_finalize_order,
    TraceGuard, FinalizeGuard, GcGuard, RootGuard,
    WeakRefState, WeakRefDebugInfo,
    gc_cycle_count, allocations_since_gc,
};

// Re-export our types
// Note: root::Dom uses NonNull<T> for SpiderMonkey API compatibility
// reflector::Dom uses Gc<T> for Boa-native usage (aliased as GcDom)
pub use reflector::{Reflector, DomObject, MutDomObject, DomRefCell};
pub use reflector::{DomTypes, DomObjectWrap, Castable, DerivedFrom};
pub use reflector::Dom as GcDom;  // Boa-native GC-managed Dom
pub use root::{Dom, DomRoot, Root, RootCollection, MaybeUnreflectedDom, assert_in_script, trace_roots};
pub use root::{DomExtractionError, DomExtractionResult, RootFromObject, extraction_to_js_result};
pub use trace::BoaTraceable;
pub use weakref::{WeakRef, WeakBox, WeakReferenceable, MutableWeakRef};
pub use cell::{MutDom, MutNullableDom, DomOnceCell, LayoutDom, assert_in_layout};
pub use runtime::JsRuntime;
pub use error::JsException;
pub use conversions::{ToJsValue, FromJsValue};
pub use dom_conversions::{
    ConversionResult, StringificationBehavior, IDLInterface,
    ToJSValConvertible, FromJSValConvertible, NativeFromObject,
    // Phase B2: Enhanced native_from_object types
    NativeFromObjectError, NativeFromObjectResult, NativeFromObjectExt,
    CrossRealmExtractable, type_error_for_interface, is_dom_wrapper,
};

// Note: rooted! macro is exported via #[macro_export] in js_compat/gc.rs
// and is available at crate root level

pub use settings_stack::{
    StackEntryKind, StackEntry, SettingsStackAccess,
    AutoEntryScript, AutoIncumbentScript,
    entry_global, incumbent_global, has_entry_global, is_stack_empty,
    realm_depth, max_realm_depth,
};
// Phase B3: Event loop integration
pub use event_loop::{
    TaskSource, Task,
    schedule_task, schedule_js_callback, cancel_task, pending_task_count,
    queue_microtask, pending_microtask_count, perform_microtask_checkpoint,
    microtask_checkpoint_count, process_one_task, run_until_empty,
    has_pending_work, is_processing_task,
};

// Re-export codegen types
pub use codegen::types::{DOMString, USVString, ByteString, ToJsValueBoa, FromJsValueBoa};
pub use codegen::traits::{WebIdlInterface, WebIdlConstructable, InterfaceBuilder, ConstantValue};
pub use codegen::interface::{check_args_length, get_arg, get_optional_arg, get_arg_with_default};
pub use codegen::prototype_list::{
    ID as ProtoId, Constructor as ConstructorId, InterfaceChain,
    MAX_PROTO_CHAIN_LENGTH, proto_count, constructor_count, proto_or_iface_length,
    allocate_proto_id, allocate_constructor_id, proto_id_to_name, register_interface_name,
    well_known as well_known_protos,
};
pub use codegen::register_bindings::{
    InterfaceDescriptor, RegisterBinding, ProtoAndIfaceCache,
    register_interface, get_interface_by_name, get_interface_by_proto_id,
    get_all_interfaces, register_all_bindings, register_window_bindings, register_worker_bindings,
    init as init_bindings,
};

// Also re-export as PrototypeList for SpiderMonkey API compatibility
pub mod PrototypeList {
    pub use super::codegen::prototype_list::{
        ID, Constructor, InterfaceChain,
        MAX_PROTO_CHAIN_LENGTH, 
        proto_count, constructor_count, proto_or_iface_length,
        allocate_proto_id, allocate_constructor_id,
        proto_id_to_name, register_interface_name,
        well_known,
    };
    
    /// Alias for proto_or_iface_length() for SpiderMonkey compatibility
    pub fn PROTO_OR_IFACE_LENGTH() -> usize {
        proto_or_iface_length()
    }
}

// Also re-export as RegisterBindings for SpiderMonkey API compatibility
pub mod RegisterBindings {
    pub use super::codegen::register_bindings::{
        InterfaceDescriptor, RegisterBinding, ProtoAndIfaceCache,
        RegisterFn, CreatePrototypeFn, CreateConstructorFn,
        register_interface, get_interface_by_name, get_interface_by_proto_id,
        get_interface_by_constructor_id, get_all_interfaces,
        register_all_bindings, register_window_bindings, register_worker_bindings,
        init,
    };
}

/// Initialize the Boa JavaScript engine.
pub fn init() {
    log::info!("Initializing Boa JavaScript engine v{}", version());
}

/// Get the Boa engine version string.
pub fn version() -> &'static str {
    "0.21.0"
}

#[cfg(test)]
mod tests {
    use super::runtime::JsRuntime;

    #[test]
    fn test_basic_eval() {
        let mut runtime = JsRuntime::new();
        let result = runtime.eval("1 + 1").unwrap();
        assert_eq!(result.as_number(), Some(2.0));
    }

    #[test]
    fn test_string_eval() {
        let mut runtime = JsRuntime::new();
        let result = runtime.eval("'hello' + ' ' + 'world'").unwrap();
        let s = result.as_string().unwrap();
        assert_eq!(s.to_std_string_escaped(), "hello world");
    }

    #[test]
    fn test_function_call() {
        let mut runtime = JsRuntime::new();
        runtime.eval("function add(a, b) { return a + b; }").unwrap();
        let result = runtime.eval("add(2, 3)").unwrap();
        assert_eq!(result.as_number(), Some(5.0));
    }
}
