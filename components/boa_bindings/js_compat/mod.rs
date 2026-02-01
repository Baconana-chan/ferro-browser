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

/// Get well-known symbol - uses jsapi::Symbol
pub unsafe fn GetWellKnownSymbol(
    _cx: *mut jsapi::RawJSContext,
    _which: jsapi::SymbolCode,
) -> *mut jsapi::Symbol {
    std::ptr::null_mut()
}

// Note: Symbol and SymbolCode are defined in jsapi.rs and re-exported via pub use jsapi::*

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

// Re-export jsid type via glue module (accessible as crate::js::glue::jsid)
pub use jsapi::HandleId;
pub use jsapi::MutableHandleId;

// Re-export CurrentRealm from realm module
pub use realm::CurrentRealm;

// jsid module - re-exports for crate::js::jsid access pattern
#[allow(non_camel_case_types)]
pub mod jsid {
    //! jsid types - for accessing SymbolId, StringId, jsid type, etc.
    pub use super::glue::{jsid, PropertyKey, HandleId, MutableHandleId, JSID_VOID};
    pub use super::jsapi::{SymbolId, StringId};
}

// ===================
// Additional JSITER flags
// ===================
pub const JSITER_OWNONLY: u32 = 0x8;
pub const JSITER_HIDDEN: u32 = 0x10;
pub const JSITER_SYMBOLS: u32 = 0x20;

// ===================
// Additional JSCLASS constants
// ===================
pub const JSCLASS_RESERVED_SLOTS_WIDTH: u32 = 8;

// ===================
// HideScriptedCaller/UnhideScriptedCaller
// ===================
/// Hide the scripted caller
pub unsafe fn HideScriptedCaller(_cx: *mut jsapi::RawJSContext) {
}

/// Unhide the scripted caller
pub unsafe fn UnhideScriptedCaller(_cx: *mut jsapi::RawJSContext) {
}

/// AutoHideScriptedCaller - RAII guard for hiding scripted caller
pub struct AutoHideScriptedCaller {
    _private: [u8; 0],
}

// ===================
// JSAtom and LinearString APIs
// ===================
/// JSAtom - interned string type
#[repr(C)]
pub struct JSAtom {
    _private: [u8; 0],
}

/// JSAtomState - atoms table state
#[repr(C)]
pub struct JSAtomState {
    _private: [u8; 0],
}

/// Atomize a string
pub unsafe fn JS_AtomizeStringN(
    _cx: *mut jsapi::RawJSContext,
    _s: *const i8,
    _len: usize,
) -> *mut JSAtom {
    std::ptr::null_mut()
}

/// Convert atom to linear string
pub unsafe fn AtomToLinearString(_atom: *mut JSAtom) -> *mut LinearString {
    std::ptr::null_mut()
}

/// LinearString - flat (linear) JS string
#[repr(C)]
pub struct LinearString {
    _private: [u8; 0],
}

/// Get length of linear string
pub unsafe fn GetLinearStringLength(_s: *mut LinearString) -> usize {
    0
}

/// Get character at index of linear string
pub unsafe fn GetLinearStringCharAt(_s: *mut LinearString, _idx: usize) -> u16 {
    0
}

/// Check if string is array index
pub unsafe fn StringIsArrayIndex(
    _s: *mut LinearString,
    _index: *mut u32,
) -> bool {
    false
}

// ===================
// Global Object APIs
// ===================
/// Check if object is a global object
pub unsafe fn JS_IsGlobalObject(_obj: *mut jsapi::JSObject) -> bool {
    false
}

/// Check if a standard class may need resolving
pub unsafe fn JS_MayResolveStandardClass(
    _names: *const JSAtomState,
    _id: glue::jsid,
    _resolved: *mut bool,
) -> bool {
    true
}

/// Resolve a standard class
pub unsafe fn JS_ResolveStandardClass(
    _cx: *mut jsapi::RawJSContext,
    _obj: rust::HandleObject<'_>,
    _id: glue::HandleId<'_>,
    _resolved: *mut bool,
) -> bool {
    true
}

/// Enumerate standard classes
pub unsafe fn JS_NewEnumerateStandardClasses(
    _cx: *mut jsapi::RawJSContext,
    _obj: rust::HandleObject<'_>,
    _props: *mut rust::IdVector,
    _enumerate_standard: bool,
) -> bool {
    true
}

/// GetPropertyKeys wrapper at root level
pub unsafe fn GetPropertyKeys(
    _cx: *mut jsapi::RawJSContext,
    _obj: rust::HandleObject<'_>,
    _flags: u32,
    _props: rust::MutableHandle<'_, rust::IdVector>,
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