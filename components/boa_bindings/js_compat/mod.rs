// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey compatibility layer for Boa
// This module provides types and functions with the same interface as mozjs
// to allow components/script to compile with Boa instead of SpiderMonkey.

pub mod jsapi;
pub mod jsval;
pub mod rust;
pub mod gc;
pub mod typedarray;
pub mod conversions;
pub mod glue;
pub mod context;
pub mod realm;
pub mod error;
pub mod panic;

// Re-export common types at the root level
pub use jsapi::*;
pub use jsval::*;

/// JSCLASS flags
pub const JSCLASS_IS_DOMJSCLASS: u32 = 1 << 0;
pub const JSCLASS_IS_GLOBAL: u32 = 1 << 1;
pub const JSCLASS_RESERVED_SLOTS_SHIFT: u32 = 8;
pub const JSCLASS_RESERVED_SLOTS_MASK: u32 = 0xFF;
pub const JSCLASS_DELAY_METADATA_BUILDER: u32 = 1 << 16;
pub const JSCLASS_IS_PROXY: u32 = 1 << 17;
pub const JSCLASS_FOREGROUND_FINALIZE: u32 = 1 << 19;
pub const JSCLASS_BACKGROUND_FINALIZE: u32 = 1 << 20;
pub const JSCLASS_HAS_PRIVATE: u32 = 1 << 21;

/// JSClass NON_NATIVE constant
pub const JSClass_NON_NATIVE: u32 = 1 << 18;

/// Global slot count (SpiderMonkey reserves slots for built-ins)
pub const JSCLASS_GLOBAL_SLOT_COUNT: u32 = 78;

/// User-defined bit flags for JSCLASS
pub const JSCLASS_USERBIT1: u32 = 1 << 22;
pub const JSCLASS_USERBIT2: u32 = 1 << 23;
pub const JSCLASS_USERBIT3: u32 = 1 << 24;

/// Undefined handle value (as a function for compatibility)
pub fn UndefinedHandleValue() -> jsapi::HandleValue<'static> {
    static UNDEFINED: jsapi::Value = jsapi::Value::undefined();
    unsafe { jsapi::HandleValue::from_raw(&UNDEFINED as *const _) }
}

/// Get well-known symbol
pub unsafe fn GetWellKnownSymbol(
    _cx: *mut jsapi::RawJSContext,
    _which: SymbolCode,
) -> *mut Symbol {
    std::ptr::null_mut()
}

/// Symbol type - JS symbol
#[repr(C)]
pub struct Symbol {
    _private: [u8; 0],
}

/// Symbol code enum
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolCode {
    Iterator = 0,
    Match = 1,
    Replace = 2,
    Search = 3,
    Split = 4,
    HasInstance = 5,
    IsConcatSpreadable = 6,
    Unscopables = 7,
    Species = 8,
    ToPrimitive = 9,
    ToStringTag = 10,
    AsyncIterator = 11,
    MatchAll = 12,
}

/// Set immutable prototype
pub unsafe fn JS_SetImmutablePrototype(
    _cx: *mut jsapi::RawJSContext,
    _obj: jsapi::HandleObject<'_>,
    _succeeded: *mut bool,
) -> bool {
    true
}

/// Proxy class extension
#[repr(C)]
pub struct ProxyClassExtension {
    _private: [u8; 0],
}

/// Proxy class ops
#[repr(C)]
pub struct ProxyClassOps {
    _private: [u8; 0],
}

/// Proxy object ops
#[repr(C)]
pub struct ProxyObjectOps {
    _private: [u8; 0],
}

/// Mutable handle ID vector
pub type MutableHandleIdVector<'a> = rust::MutableHandle<'a, Vec<glue::jsid>>;

/// Stream consumer
#[repr(C)]
pub struct StreamConsumer {
    _private: [u8; 0],
}

/// Set process build ID operation
pub unsafe fn SetProcessBuildIdOp(
    _build_id_op: Option<unsafe extern "C" fn(*mut jsapi::BuildIdCharVector) -> bool>,
) {
}

// Re-export jsid from glue at root level
pub use glue::jsid;
pub use jsapi::HandleId;
pub use jsapi::MutableHandleId;

// Re-export CurrentRealm from realm module
pub use realm::CurrentRealm;

/// GetPropertyKeys wrapper at root level
pub unsafe fn GetPropertyKeys(
    _cx: *mut jsapi::RawJSContext,
    _obj: rust::HandleObject<'_>,
    _flags: u32,
    _props: *mut rust::IdVector,
) -> bool {
    true
}

/// JS_GetPropertyById wrapper at root level
pub unsafe fn JS_GetPropertyById(
    _cx: *mut jsapi::RawJSContext,
    _obj: rust::HandleObject<'_>,
    _id: glue::HandleId<'_>,
    _vp: rust::MutableHandleValue<'_>,
) -> bool {
    true
}

// Note: The rooted! macro is defined with #[macro_export] in gc.rs
// and is available at crate root level as boa_bindings::rooted