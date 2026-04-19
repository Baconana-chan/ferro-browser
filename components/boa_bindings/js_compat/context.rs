// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey context module compatibility layer for Boa

use std::ptr;
use std::ffi::c_void;

use super::jsapi::{AsmJSOption, JSObject, Value};
use super::rust::{HandleObject, HandleValue, MutableHandleValue};
use std::marker::PhantomData;

// Re-export JSContext and RawJSContext from jsapi for convenience
pub use super::jsapi::{JSContext, RawJSContext};

/// CurrentRealm - get the current realm
pub struct CurrentRealm<'a> {
    realm: *mut c_void,
    cx: *mut RawJSContext,
    _marker: PhantomData<&'a mut RawJSContext>,
}

impl<'a> CurrentRealm<'a> {
    pub fn new(cx: *mut RawJSContext) -> Self {
        Self {
            realm: ptr::null_mut(),
            cx,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> *mut c_void {
        self.realm
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.realm
    }

    pub fn realm(&self) -> &Self {
        self
    }

    pub fn raw_cx_no_gc(&self) -> *mut RawJSContext {
        self.cx
    }

    pub fn global(&self) -> HandleObject<'static> {
        let global = unsafe { GetRealmGlobalOrNull(self.realm) };
        unsafe { HandleObject::from_raw(&global) }
    }

    /// Assert that the current realm matches expected
    pub fn assert(_cx: *mut RawJSContext, _realm: *mut c_void) {
        // Stub - in debug mode would assert realms match
    }
}

/// Context options
#[repr(C)]
pub struct CompileOptions {
    pub asmJSOption_: AsmJSOption,
    pub wasm_: bool,
    pub wasmBaseline_: bool,
    pub wasmIon_: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            asmJSOption_: AsmJSOption::Enabled,
            wasm_: true,
            wasmBaseline_: true,
            wasmIon_: true,
        }
    }
}

#[repr(C)]
pub struct ContextOptions {
    pub compileOptions_: CompileOptions,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            compileOptions_: CompileOptions::default(),
        }
    }
}

impl ContextOptions {
    pub fn set_wasm_(&mut self, enabled: bool) {
        self.compileOptions_.wasm_ = enabled;
    }

    pub fn set_wasmBaseline_(&mut self, enabled: bool) {
        self.compileOptions_.wasmBaseline_ = enabled;
    }

    pub fn set_wasmIon_(&mut self, enabled: bool) {
        self.compileOptions_.wasmIon_ = enabled;
    }
}

/// Get context options
pub unsafe fn ContextOptionsRef(_cx: *mut RawJSContext) -> *mut ContextOptions {
    ptr::null_mut()
}

/// Realm options
#[repr(C)]
pub struct RealmOptions {
    pub class_is_dom: bool,
}

impl Default for RealmOptions {
    fn default() -> Self {
        Self {
            class_is_dom: false,
        }
    }
}

/// CompartmentOptions - options for a compartment
#[repr(C)]
pub struct CompartmentOptions {
    _private: (),
}

impl Default for CompartmentOptions {
    fn default() -> Self {
        Self { _private: () }
    }
}

/// Get current thread context
pub fn CurrentContext() -> *mut RawJSContext {
    ptr::null_mut()
}

/// Report OOM error
pub unsafe fn ReportOutOfMemory(_cx: *mut RawJSContext) {
}

/// Is exception pending
pub unsafe fn JS_IsExceptionPending(_cx: *mut RawJSContext) -> bool {
    false
}

/// Clear pending exception
pub unsafe fn JS_ClearPendingException(_cx: *mut RawJSContext) {
}

/// Report error
pub unsafe fn JS_ReportErrorASCII(_cx: *mut RawJSContext, _msg: *const i8) {
}

/// Report error with flags
pub unsafe fn JS_ReportErrorFlagsAndNumberASCII(
    _cx: *mut RawJSContext,
    _flags: u32,
    _error_callback: Option<unsafe extern "C" fn() -> *const i8>,
    _error_number: u32,
) {
}

/// Report error with Latin1
pub unsafe fn JS_ReportErrorNumberLatin1(
    _cx: *mut RawJSContext,
    _error_callback: Option<unsafe extern "C" fn() -> *const i8>,
    _closure: *mut c_void,
    _error_number: u32,
) {
}

/// Report error with Unicode
pub unsafe fn JS_ReportErrorNumberUC(
    _cx: *mut RawJSContext,
    _error_callback: Option<unsafe extern "C" fn() -> *const i8>,
    _closure: *mut c_void,
    _error_number: u32,
) {
}

/// Throw error
pub unsafe fn JS_SetPendingException(
    _cx: *mut RawJSContext,
    _exception: HandleValue<'_>,
    _behavior: u32,
) {
}

/// Get pending exception
pub unsafe fn JS_GetPendingException(
    _cx: *mut RawJSContext,
    _rval: MutableHandleValue<'_>,
) -> bool {
    false
}

/// Is stopping due to exception
pub unsafe fn JS_IsStopIteration(_cx: *mut RawJSContext, _v: Value) -> bool {
    false
}

/// Check if context is on correct thread
pub unsafe fn JS_IsOnCurrentThread(_cx: *mut RawJSContext) -> bool {
    true
}

/// Enter realm
pub unsafe fn JS_EnterRealm(cx: *mut RawJSContext, obj: HandleObject<'_>) -> *mut c_void {
    super::jsapi::EnterRealm(cx, obj.get()) as *mut c_void
}

/// Leave realm
pub unsafe fn JS_LeaveRealm(cx: *mut RawJSContext, old_realm: *mut c_void) {
    super::jsapi::LeaveRealm(cx, old_realm as *mut super::jsapi::Realm)
}

/// Get current global
pub unsafe fn CurrentGlobalOrNull(cx: *mut RawJSContext) -> *mut JSObject {
    let _ = cx;
    super::jsapi::CurrentGlobalOrNull(cx)
}

/// Get realm global
pub unsafe fn GetRealmGlobalOrNull(realm: *mut c_void) -> *mut JSObject {
    super::jsapi::GetRealmGlobalOrNull(realm as *mut super::jsapi::Realm)
}

/// Init standard classes
pub unsafe fn JS_InitStandardClasses(_cx: *mut RawJSContext, _global: HandleObject<'_>) -> bool {
    true
}

/// Init reflection parse
pub unsafe fn JS_InitReflectParse(_cx: *mut RawJSContext, _global: HandleObject<'_>) -> bool {
    true
}

/// Get script filename
pub unsafe fn JS_GetScriptFilename(_script: HandleObject<'_>) -> *const i8 {
    ptr::null()
}

/// Get function display name
pub unsafe fn JS_GetFunctionDisplayId(_fun: *mut JSObject) -> *mut super::jsapi::JSString {
    ptr::null_mut()
}

/// Request interrupt callback
pub unsafe fn JS_RequestInterruptCallback(_cx: *mut RawJSContext) {
}

/// Disable interrupt callback
pub unsafe fn JS_DisableInterruptCallback(_cx: *mut RawJSContext) {
}

/// Enable interrupt callback
pub unsafe fn JS_ResetInterruptCallback(_cx: *mut RawJSContext, _enable: bool) {
}

/// Malloc bytes
pub unsafe fn JS_malloc(_cx: *mut RawJSContext, _nbytes: usize) -> *mut c_void {
    ptr::null_mut()
}

/// Free bytes
pub unsafe fn JS_free(_cx: *mut RawJSContext, _p: *mut c_void) {
}

/// Realloc bytes
pub unsafe fn JS_realloc(
    _cx: *mut RawJSContext,
    _p: *mut c_void,
    _old_bytes: usize,
    _new_bytes: usize,
) -> *mut c_void {
    ptr::null_mut()
}

/// Add ref to context
pub unsafe fn JS_AddRef(_cx: *mut RawJSContext) {
}

/// Release ref from context
pub unsafe fn JS_Release(_cx: *mut RawJSContext) {
}
