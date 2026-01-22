// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey error module compatibility layer for Boa

use std::ptr;
use std::ffi::c_void;

use super::jsapi::{RawJSContext, JSObject, Value};
use super::rust::{HandleObject, HandleValue, MutableHandleValue};

/// ErrorReport - represents a JS error
#[repr(C)]
pub struct JSErrorReport {
    pub message: *const i8,
    pub filename: *const i8,
    pub lineno: u32,
    pub column: u32,
    pub error_number: u32,
    pub exception_object: *mut JSObject,
}

impl Default for JSErrorReport {
    fn default() -> Self {
        Self {
            message: ptr::null(),
            filename: ptr::null(),
            lineno: 0,
            column: 0,
            error_number: 0,
            exception_object: ptr::null_mut(),
        }
    }
}

/// Error report flags
pub const JSREPORT_ERROR: u32 = 0x0;
pub const JSREPORT_WARNING: u32 = 0x1;
pub const JSREPORT_EXCEPTION: u32 = 0x2;
pub const JSREPORT_STRICT: u32 = 0x4;
pub const JSREPORT_STRICT_MODE_ERROR: u32 = 0x8;

/// Error report callback
pub type JSErrorReporter = Option<unsafe extern "C" fn(
    cx: *mut RawJSContext,
    message: *const i8,
    report: *mut JSErrorReport,
)>;

/// Set error reporter
pub unsafe fn JS_SetErrorReporter(
    _cx: *mut RawJSContext,
    _reporter: JSErrorReporter,
) {
}

/// Get error reporter
pub unsafe fn JS_GetErrorReporter(
    _cx: *mut RawJSContext,
) -> JSErrorReporter {
    None
}

/// ExceptionStack - exception with stack trace
#[repr(C)]
pub struct ExceptionStack {
    pub exception: Value,
    pub stack: *mut JSObject,
}

impl Default for ExceptionStack {
    fn default() -> Self {
        Self {
            exception: Value::undefined(),
            stack: ptr::null_mut(),
        }
    }
}

/// Get pending exception with stack
pub unsafe fn GetPendingExceptionStack(
    _cx: *mut RawJSContext,
    _exception_stack: *mut ExceptionStack,
) -> bool {
    false
}

/// Set pending exception with stack
pub unsafe fn SetPendingExceptionStack(
    _cx: *mut RawJSContext,
    _exception_stack: *const ExceptionStack,
) {
}

/// Error kind
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JSExnType {
    None = 0,
    Error = 1,
    InternalError = 2,
    AggregateError = 3,
    EvalError = 4,
    RangeError = 5,
    ReferenceError = 6,
    SyntaxError = 7,
    TypeError = 8,
    URIError = 9,
    DebuggeeWouldRun = 10,
    CompileError = 11,
    LinkError = 12,
    RuntimeError = 13,
}

/// Create error object
pub unsafe fn JS_NewError(
    _cx: *mut RawJSContext,
    _exn_type: JSExnType,
    _message: *const i8,
) -> *mut JSObject {
    ptr::null_mut()
}

/// Create TypeError
pub unsafe fn JS_NewTypeError(
    _cx: *mut RawJSContext,
    _message: *const i8,
) -> *mut JSObject {
    ptr::null_mut()
}

/// Create RangeError
pub unsafe fn JS_NewRangeError(
    _cx: *mut RawJSContext,
    _message: *const i8,
) -> *mut JSObject {
    ptr::null_mut()
}

/// Create SyntaxError
pub unsafe fn JS_NewSyntaxError(
    _cx: *mut RawJSContext,
    _message: *const i8,
) -> *mut JSObject {
    ptr::null_mut()
}

/// Create URIError
pub unsafe fn JS_NewURIError(
    _cx: *mut RawJSContext,
    _message: *const i8,
) -> *mut JSObject {
    ptr::null_mut()
}

/// Create ReferenceError
pub unsafe fn JS_NewReferenceError(
    _cx: *mut RawJSContext,
    _message: *const i8,
) -> *mut JSObject {
    ptr::null_mut()
}

/// Create EvalError
pub unsafe fn JS_NewEvalError(
    _cx: *mut RawJSContext,
    _message: *const i8,
) -> *mut JSObject {
    ptr::null_mut()
}

/// Throw type error
pub unsafe fn ThrowTypeError(
    _cx: *mut RawJSContext,
    _message: *const i8,
) -> bool {
    false
}

/// Throw range error
pub unsafe fn ThrowRangeError(
    _cx: *mut RawJSContext,
    _message: *const i8,
) -> bool {
    false
}

/// Throw syntax error
pub unsafe fn ThrowSyntaxError(
    _cx: *mut RawJSContext,
    _message: *const i8,
) -> bool {
    false
}

/// Check for pending exception and handle it
pub unsafe fn ReportException(_cx: *mut RawJSContext) -> bool {
    false
}

/// Check for uncatchable exception
pub unsafe fn IsUncatchableException(_cx: *mut RawJSContext, _val: HandleValue<'_>) -> bool {
    false
}

/// Capture current stack
pub unsafe fn CaptureCurrentStack(
    _cx: *mut RawJSContext,
    _stack_p: *mut *mut JSObject,
    _max_frame_count: u32,
) -> bool {
    false
}

/// Build stack string
pub unsafe fn BuildStackString(
    _cx: *mut RawJSContext,
    _principals: *mut c_void,
    _stack: HandleObject<'_>,
    _string_p: MutableHandleValue<'_>,
    _indent: u32,
    _max_string_length: usize,
) -> bool {
    false
}

/// Error number callback type
pub type JSErrorCallback = Option<unsafe extern "C" fn(
    user_data: *mut c_void,
    error_number: u32,
) -> *const JSErrorFormatString>;

/// Error format string
#[repr(C)]
pub struct JSErrorFormatString {
    pub name: *const i8,
    pub format: *const i8,
    pub arg_count: u16,
    pub exn_type: i16,
}

/// Set error callback
pub unsafe fn JS_SetErrorMessageCallback(
    _cx: *mut RawJSContext,
    _callback: JSErrorCallback,
    _user_data: *mut c_void,
) {
}

/// DOM exception error numbers
pub mod dom {
    pub const DOMEXCEPTION_NAMES: [&str; 25] = [
        "IndexSizeError",
        "DOMStringSizeError",
        "HierarchyRequestError",
        "WrongDocumentError",
        "InvalidCharacterError",
        "NoDataAllowedError",
        "NoModificationAllowedError",
        "NotFoundError",
        "NotSupportedError",
        "InUseAttributeError",
        "InvalidStateError",
        "SyntaxError",
        "InvalidModificationError",
        "NamespaceError",
        "InvalidAccessError",
        "ValidationError",
        "TypeMismatchError",
        "SecurityError",
        "NetworkError",
        "AbortError",
        "URLMismatchError",
        "QuotaExceededError",
        "TimeoutError",
        "InvalidNodeTypeError",
        "DataCloneError",
    ];
}
