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

//! Boa JavaScript Engine bindings for Ferro Browser.
//!
//! This module provides a pure Rust alternative to SpiderMonkey-based
//! script_bindings. Boa is a JavaScript engine written entirely in Rust,
//! offering better debugging, faster builds, and seamless integration.
//!
//! ## Features
//!
//! - **Pure Rust**: No C++ dependencies, no mozjs build complexity
//! - **Boa 0.21**: 94% ECMAScript conformance, NaN-boxing, register-based VM
//! - **Web APIs**: setTimeout, fetch, console via boa_runtime
//! - **DOM Integration**: Reflector pattern for connecting Rust DOM objects to JS
//!
//! ## Architecture
//!
//! - `runtime` - JavaScript runtime management (Context)
//! - `gc` - Garbage collection integration with DOM
//! - `conversions` - Type conversions between Rust and JavaScript
//! - `error` - JavaScript error handling
//! - `reflector` - DOM object reflection (Rust ↔ JS binding)
//! - `builtins` - Servo-specific Web API extensions

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

// Re-export our types
// Note: root::Dom uses NonNull<T> for SpiderMonkey API compatibility
// reflector::Dom uses Gc<T> for Boa-native usage (aliased as GcDom)
pub use reflector::{Reflector, DomObject, MutDomObject, DomRefCell};
pub use reflector::{DomTypes, DomObjectWrap, Castable, DerivedFrom};
pub use reflector::Dom as GcDom;  // Boa-native GC-managed Dom
pub use root::{Dom, DomRoot, Root, RootCollection, MaybeUnreflectedDom, assert_in_script, trace_roots};
pub use trace::BoaTraceable;
pub use weakref::{WeakRef, WeakBox, WeakReferenceable, MutableWeakRef};
pub use cell::{MutDom, MutNullableDom, DomOnceCell, LayoutDom, assert_in_layout};
pub use runtime::JsRuntime;
pub use error::JsException;
pub use conversions::{ToJsValue, FromJsValue};
pub use dom_conversions::{
    ConversionResult, StringificationBehavior, IDLInterface,
    ToJSValConvertible, FromJSValConvertible, NativeFromObject,
};
pub use settings_stack::{
    StackEntryKind, StackEntry, SettingsStackAccess,
    AutoEntryScript, AutoIncumbentScript,
    entry_global, incumbent_global, has_entry_global, is_stack_empty,
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
