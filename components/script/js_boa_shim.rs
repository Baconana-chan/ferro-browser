// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// JavaScript engine shim for Boa
// 
// This module provides the same API as `extern crate js` (mozjs)
// but backed by boa_bindings::js_compat
//
// This allows existing code with `use js::...` to work unchanged.

// Re-export all modules from js_compat
pub use boa_bindings::js_compat::jsapi;
pub use boa_bindings::js_compat::jsval;
pub use boa_bindings::js_compat::rust;
pub use boa_bindings::js_compat::gc;
pub use boa_bindings::js_compat::typedarray;
pub use boa_bindings::js_compat::conversions;
pub use boa_bindings::js_compat::glue;
pub use boa_bindings::js_compat::panic;

// Re-export root-level types and constants
pub use boa_bindings::js_compat::{
    JSCLASS_IS_DOMJSCLASS,
    JSCLASS_IS_GLOBAL,
    JSCLASS_RESERVED_SLOTS_SHIFT,
    JSCLASS_RESERVED_SLOTS_MASK,
};

// Re-export Value constructors at root level
pub use boa_bindings::js_compat::jsval::{
    UndefinedValue,
    NullValue,
    BooleanValue,
    Int32Value,
    DoubleValue,
    StringValue,
    ObjectValue,
    ObjectOrNullValue,
    PrivateValue,
};

// Re-export commonly used jsapi types at root level
pub use boa_bindings::js_compat::jsapi::{
    JSContext,
    RawJSContext,
    JSObject,
    JSString,
    JSScript,
    JSTracer,
    JSClass,
    JSClassOps,
    JSSecurityCallbacks,
    Value,
    Heap,
    Handle,
    MutableHandle,
    HandleValue,
    HandleObject,
    MutableHandleValue,
    MutableHandleObject,
    JSAutoRealm,
    CallArgs,
    HandleValueArray,
    JobQueue,
    CurrentGlobalOrNull,
};
