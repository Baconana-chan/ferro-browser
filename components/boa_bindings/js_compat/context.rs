// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey context module compatibility layer for Boa

use std::ptr;
use std::ffi::c_void;

use super::jsapi::{RawJSContext, JSObject, Value};
use super::rust::{HandleObject, HandleValue, MutableHandleValue};

/// Context options
#[repr(C)]
pub struct ContextOptions {
    pub baseline: bool,
    pub ion: bool,
    pub asmjs: bool,
    pub wasm: bool,
    pub wasm_verbose: bool,
    pub wasm_baseline: bool,
    pub wasm_ion: bool,
    pub native_regexp: bool,
    pub async_stack: bool,
    pub throw_on_debuggee_would_run: bool,
    pub dump_stack_on_debuggee_would_run: bool,
    pub strict: bool,
    pub extra_warnings: bool,
    pub werror: bool,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            baseline: true,
            ion: true,
            asmjs: true,
            wasm: true,
            wasm_verbose: false,
            wasm_baseline: true,
            wasm_ion: true,
            native_regexp: true,
            async_stack: true,
            throw_on_debuggee_would_run: false,
            dump_stack_on_debuggee_would_run: false,
            strict: false,
            extra_warnings: false,
            werror: false,
        }
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
pub unsafe fn JS_EnterRealm(_cx: *mut RawJSContext, _obj: HandleObject<'_>) -> *mut c_void {
    ptr::null_mut()
}

/// Leave realm
pub unsafe fn JS_LeaveRealm(_cx: *mut RawJSContext, _old_realm: *mut c_void) {
}

/// Get current global
pub unsafe fn CurrentGlobalOrNull(_cx: *mut RawJSContext) -> *mut JSObject {
    ptr::null_mut()
}

/// Get realm global
pub unsafe fn GetRealmGlobalOrNull(_realm: *mut c_void) -> *mut JSObject {
    ptr::null_mut()
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
