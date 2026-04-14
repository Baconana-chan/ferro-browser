// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// JavaScript engine shim for Boa
// 
// This module provides the same API as `extern crate js` (mozjs)
// but backed by boa_bindings::js_compat
//
// This allows existing code with `use crate::js::...` to work unchanged.

// Re-export all modules from js_compat
pub use boa_bindings::js_compat::jsapi;
pub use boa_bindings::js_compat::jsval;
pub use boa_bindings::js_compat::rust;
pub use boa_bindings::js_compat::gc;
pub use boa_bindings::js_compat::typedarray;
pub use boa_bindings::js_compat::conversions;
pub use boa_bindings::js_compat::glue;
pub use boa_bindings::js_compat::panic;
pub use boa_bindings::js_compat::context;
pub use boa_bindings::js_compat::error;
// Re-export jsid module for crate::js::jsid::SymbolId imports
pub use boa_bindings::js_compat::jsid;

// Realm is a sub-module of context in our compat layer
pub mod realm {
    pub use boa_bindings::js_compat::context::*;
}

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
// Re-export GC types that some code imports from js::rust
pub use boa_bindings::js_compat::gc::CustomAutoRooter;
pub use boa_bindings::js_compat::gc::CustomAutoRooterGuard;

// Re-export jsid type at root level (the struct, not the module)
pub use boa_bindings::js_compat::glue::jsid as JsIdType;

// Re-export additional JSCLASS constants
pub use boa_bindings::js_compat::{
    JSCLASS_DELAY_METADATA_BUILDER,
    JSCLASS_IS_PROXY,
    JSClass_NON_NATIVE,
    UndefinedHandleValue,
    GetWellKnownSymbol,
    SymbolCode,
    Symbol,
    JS_SetImmutablePrototype,
    ProxyClassExtension,
    ProxyClassOps,
    ProxyObjectOps,
    MutableHandleIdVector,
    StreamConsumer,
    SetProcessBuildIdOp,
    GetPropertyKeys,
    JS_GetPropertyById,
};