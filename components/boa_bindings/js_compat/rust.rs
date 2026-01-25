// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey rust module compatibility layer for Boa

use std::ptr;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::jsapi::{
    JSObject, JSContext, RawJSContext, JSString, JSTracer, Value, JSClass,
    Handle as RawHandle, MutableHandle as RawMutableHandle,
};

// Re-export CustomAutoRooter from gc for compatibility (some code imports from rust)
pub use super::gc::{CustomAutoRooter, CustomAutoRooterGuard};

/// Runtime - the JavaScript runtime
pub struct Runtime {
    _private: (),
}

impl Runtime {
    pub fn new(_engine: JSEngineHandle) -> Result<Self, ()> {
        Ok(Self { _private: () })
    }
    
    pub fn cx(&self) -> *mut RawJSContext {
        ptr::null_mut()
    }
}

/// ParentRuntime - for creating child runtimes
pub struct ParentRuntime {
    _private: (),
}

/// JSEngine - the JS engine singleton
pub struct JSEngine {
    _private: (),
}

impl JSEngine {
    pub fn init() -> Result<JSEngineHandle, ()> {
        Ok(JSEngineHandle { _private: () })
    }
}

/// Handle to the JS engine
#[derive(Clone)]
pub struct JSEngineHandle {
    _private: (),
}

/// Thread-safe JSContext reference
pub struct ThreadSafeJSContext {
    _private: (),
}

unsafe impl Send for ThreadSafeJSContext {}
unsafe impl Sync for ThreadSafeJSContext {}

/// Handle - immutable rooted reference
#[repr(transparent)]
pub struct Handle<'a, T> {
    ptr: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Handle<'a, T> {
    pub fn get(&self) -> T where T: Copy {
        unsafe { *self.ptr }
    }
    
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        Self { ptr, _marker: PhantomData }
    }
    
    pub fn into_handle(self) -> Self {
        self
    }
}

impl<'a, T> Clone for Handle<'a, T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr, _marker: PhantomData }
    }
}

impl<'a, T> Copy for Handle<'a, T> {}

/// HandleValue - Handle to a Value
pub type HandleValue<'a> = Handle<'a, Value>;

/// HandleObject - Handle to an object pointer
pub type HandleObject<'a> = Handle<'a, *mut JSObject>;

/// MutableHandle - mutable rooted reference
#[repr(transparent)]
pub struct MutableHandle<'a, T> {
    ptr: *mut T,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> MutableHandle<'a, T> {
    pub fn get(&self) -> T where T: Copy {
        unsafe { *self.ptr }
    }
    
    pub fn set(&self, val: T) {
        unsafe { *self.ptr = val; }
    }
    
    pub fn handle(&self) -> Handle<'a, T> {
        Handle { ptr: self.ptr, _marker: PhantomData }
    }
    
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self { ptr, _marker: PhantomData }
    }
}

/// MutableHandleValue
pub type MutableHandleValue<'a> = MutableHandle<'a, Value>;

/// MutableHandleObject
pub type MutableHandleObject<'a> = MutableHandle<'a, *mut JSObject>;

/// IntoHandle trait
pub trait IntoHandle<'a, T> {
    fn into_handle(self) -> Handle<'a, T>;
}

impl<'a, T> IntoHandle<'a, T> for Handle<'a, T> {
    fn into_handle(self) -> Handle<'a, T> {
        self
    }
}

/// IntoMutableHandle trait
pub trait IntoMutableHandle<'a, T> {
    fn into_mutable_handle(self) -> MutableHandle<'a, T>;
}

impl<'a, T> IntoMutableHandle<'a, T> for MutableHandle<'a, T> {
    fn into_mutable_handle(self) -> MutableHandle<'a, T> {
        self
    }
}

/// RustHandleObject alias
pub type RustHandleObject<'a> = HandleObject<'a>;

/// ToString - convert to string
pub unsafe fn ToString(_cx: *mut RawJSContext, _v: HandleValue<'_>) -> *mut JSString {
    ptr::null_mut()
}

/// Wrappers module - contains SpiderMonkey API wrappers
pub mod wrappers {
    use super::*;
    
    pub unsafe fn JS_GetProperty(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _name: *const i8,
        _vp: MutableHandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_SetProperty(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _name: *const i8,
        _v: HandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_HasProperty(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _name: *const i8,
        _found: *mut bool,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_HasOwnProperty(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _name: *const i8,
        _found: *mut bool,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_DefineProperty(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _name: *const i8,
        _v: HandleValue<'_>,
        _attrs: u32,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_GetPendingException(
        _cx: *mut RawJSContext,
        _vp: MutableHandleValue<'_>,
    ) -> bool {
        false
    }
    
    pub unsafe fn JS_SetPendingException(
        _cx: *mut RawJSContext,
        _v: HandleValue<'_>,
        _behavior: u32,
    ) {
    }
    
    pub unsafe fn JS_ErrorFromException(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
    ) -> *mut JSObject {
        ptr::null_mut()
    }
    
    pub unsafe fn Call(
        _cx: *mut RawJSContext,
        _this: HandleValue<'_>,
        _func: HandleObject<'_>,
        _args: *const super::super::jsapi::HandleValueArray,
        _rval: MutableHandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn Construct1(
        _cx: *mut RawJSContext,
        _ctor: HandleObject<'_>,
        _args: *const super::super::jsapi::HandleValueArray,
        _rval: MutableHandleObject<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn SameValue(
        _cx: *mut RawJSContext,
        _v1: HandleValue<'_>,
        _v2: HandleValue<'_>,
        _same: *mut bool,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_ParseJSON(
        _cx: *mut RawJSContext,
        _src: *const u16,
        _len: u32,
        _vp: MutableHandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_Stringify(
        _cx: *mut RawJSContext,
        _vp: MutableHandleValue<'_>,
        _replacer: HandleObject<'_>,
        _space: HandleValue<'_>,
        _callback: Option<unsafe extern "C" fn()>,
        _data: *mut c_void,
    ) -> bool {
        true
    }
    
    pub unsafe fn IsArrayObject(
        _cx: *mut RawJSContext,
        _v: HandleValue<'_>,
        _is_array: *mut bool,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_TypeOfValue(
        _cx: *mut RawJSContext,
        _v: HandleValue<'_>,
    ) -> u32 {
        0
    }
    
    pub unsafe fn JS_IsIdentifier(
        _cx: *mut RawJSContext,
        _str: HandleObject<'_>,
        _is_ident: *mut bool,
    ) -> bool {
        true
    }
    
    pub unsafe fn GetPromiseIsHandled(_promise: HandleObject<'_>) -> bool {
        false
    }
    
    pub unsafe fn JS_GetPromiseResult(_promise: HandleObject<'_>, _vp: MutableHandleValue<'_>) {
    }
    
    pub unsafe fn JS_CallFunctionName(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _name: *const i8,
        _args: *const super::super::jsapi::HandleValueArray,
        _rval: MutableHandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_ExecuteScript(
        _cx: *mut RawJSContext,
        _script: HandleObject<'_>,
        _rval: MutableHandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_GetScriptPrivate(_script: HandleObject<'_>) -> Value {
        Value::undefined()
    }
    
    pub unsafe fn JS_GetModulePrivate(_module: HandleObject<'_>) -> Value {
        Value::undefined()
    }
    
    pub unsafe fn JS_SetPrototype(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _proto: HandleObject<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_WrapObject(
        _cx: *mut RawJSContext,
        _obj: MutableHandleObject<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_TransplantObject(
        _cx: *mut RawJSContext,
        _old_obj: HandleObject<'_>,
        _new_obj: HandleObject<'_>,
    ) -> *mut JSObject {
        ptr::null_mut()
    }
    
    pub unsafe fn NewWindowProxy(
        _cx: *mut RawJSContext,
        _handler: HandleObject<'_>,
        _class: *const super::super::jsapi::JSClass,
    ) -> *mut JSObject {
        ptr::null_mut()
    }
    
    pub unsafe fn SetWindowProxy(
        _cx: *mut RawJSContext,
        _global: HandleObject<'_>,
        _proxy: HandleObject<'_>,
    ) {
    }
    
    pub unsafe fn CheckRegExpSyntax(
        _cx: *mut RawJSContext,
        _pattern: *const u16,
        _len: usize,
        _flags: u32,
    ) -> bool {
        true
    }
    
    pub unsafe fn ExecuteRegExpNoStatics(
        _cx: *mut RawJSContext,
        _regexp: HandleObject<'_>,
        _str: *const u16,
        _len: usize,
        _index: *mut usize,
        _test_only: bool,
        _rval: MutableHandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn ObjectIsRegExp(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _is_regexp: *mut bool,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_ReadStructuredClone(
        _cx: *mut RawJSContext,
        _data: *const u8,
        _len: usize,
        _version: u32,
        _callbacks: *const c_void,
        _closure: *mut c_void,
        _vp: MutableHandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_WriteStructuredClone(
        _cx: *mut RawJSContext,
        _v: HandleValue<'_>,
        _data: *mut *mut u8,
        _len: *mut usize,
        _callbacks: *const c_void,
        _closure: *mut c_void,
        _transferables: HandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn DetachArrayBuffer(_cx: *mut RawJSContext, _obj: HandleObject<'_>) -> bool {
        true
    }
    
    pub unsafe fn JS_DefineDebuggerObject(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn GetBuiltinClass(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _class: *mut super::super::jsapi::ESClass,
    ) -> bool {
        true
    }
    
    pub unsafe fn GetPropertyKeys(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _flags: u32,
        _props: *mut IdVector,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_GetOwnPropertyDescriptorById(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _id: super::super::glue::HandleId<'_>,
        _desc: *mut super::super::glue::PropertyDescriptor,
        _is_none: *mut bool,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_GetPropertyById(
        _cx: *mut RawJSContext,
        _obj: HandleObject<'_>,
        _id: super::super::glue::HandleId<'_>,
        _vp: MutableHandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_IdToValue(
        _cx: *mut RawJSContext,
        _id: super::super::glue::jsid,
        _vp: MutableHandleValue<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn JS_ValueToSource(
        _cx: *mut RawJSContext,
        _v: HandleValue<'_>,
    ) -> *mut JSString {
        ptr::null_mut()
    }
    
    // ===================
    // Promise Wrappers
    // ===================
    
    /// Check if object is a promise
    pub unsafe fn IsPromiseObject(obj: HandleObject<'_>) -> bool {
        let _ = obj;
        false
    }
    
    /// Get promise state
    pub unsafe fn GetPromiseState(obj: HandleObject<'_>) -> super::super::jsapi::PromiseState {
        let _ = obj;
        super::super::jsapi::PromiseState::Pending
    }
    
    /// Create a new promise object
    pub unsafe fn NewPromiseObject(
        cx: *mut RawJSContext,
        executor: HandleObject<'_>,
    ) -> *mut JSObject {
        let _ = (cx, executor);
        std::ptr::null_mut()
    }
    
    /// Resolve a promise
    pub unsafe fn ResolvePromise(
        cx: *mut RawJSContext,
        promise: HandleObject<'_>,
        value: HandleValue<'_>,
    ) -> bool {
        let _ = (cx, promise, value);
        true
    }
    
    /// Reject a promise
    pub unsafe fn RejectPromise(
        cx: *mut RawJSContext,
        promise: HandleObject<'_>,
        reason: HandleValue<'_>,
    ) -> bool {
        let _ = (cx, promise, reason);
        true
    }
    
    /// Add promise reactions (then/catch)
    pub unsafe fn AddPromiseReactions(
        cx: *mut RawJSContext,
        promise: HandleObject<'_>,
        on_fulfilled: HandleObject<'_>,
        on_rejected: HandleObject<'_>,
    ) -> bool {
        let _ = (cx, promise, on_fulfilled, on_rejected);
        true
    }
    
    /// Call original Promise.resolve
    pub unsafe fn CallOriginalPromiseResolve(
        cx: *mut RawJSContext,
        value: HandleValue<'_>,
    ) -> *mut JSObject {
        let _ = (cx, value);
        std::ptr::null_mut()
    }
    
    /// Call original Promise.reject
    pub unsafe fn CallOriginalPromiseReject(
        cx: *mut RawJSContext,
        reason: HandleValue<'_>,
    ) -> *mut JSObject {
        let _ = (cx, reason);
        std::ptr::null_mut()
    }
    
    /// Set promise as handled
    pub unsafe fn SetAnyPromiseIsHandled(
        cx: *mut RawJSContext,
        promise: HandleObject<'_>,
    ) -> bool {
        let _ = (cx, promise);
        true
    }
    
    /// Set promise user input event handling state
    pub unsafe fn SetPromiseUserInputEventHandlingState(
        promise: HandleObject<'_>,
        state: bool,
    ) {
        let _ = (promise, state);
    }
}

/// Wrappers2 module - additional wrappers
pub mod wrappers2 {
    use super::*;
    
    pub unsafe fn JS_GC(_cx: *mut RawJSContext, _reason: u32) {}
    
    pub unsafe fn JS_GetGCParameter(_cx: *mut RawJSContext, _param: u32) -> u32 {
        0
    }
    
    pub unsafe fn JS_SetPendingException(
        _cx: *mut RawJSContext,
        _v: HandleValue<'_>,
        _behavior: u32,
    ) {
    }
    
    pub unsafe fn JS_AddInterruptCallback(
        _cx: *mut RawJSContext,
        _callback: Option<unsafe extern "C" fn() -> bool>,
    ) {
    }
    
    pub unsafe fn SetWindowProxyClass(_cx: *mut RawJSContext, _class: *const super::super::jsapi::JSClass) {
    }
    
    pub fn ContextOptionsRef(_cx: *mut RawJSContext) -> *mut c_void {
        ptr::null_mut()
    }
    
    pub unsafe fn InitConsumeStreamCallback(
        _cx: *mut RawJSContext,
        _callback: Option<unsafe extern "C" fn()>,
    ) {
    }
    
    pub unsafe fn JS_AddExtraGCRootsTracer(
        _cx: *mut RawJSContext,
        _tracer: Option<unsafe extern "C" fn(*mut JSTracer, *mut c_void)>,
        _data: *mut c_void,
    ) {
    }
    
    pub unsafe fn JS_InitDestroyPrincipalsCallback(
        _cx: *mut RawJSContext,
        _callback: Option<unsafe extern "C" fn()>,
    ) {
    }
    
    pub unsafe fn JS_InitReadPrincipalsCallback(
        _cx: *mut RawJSContext,
        _callback: Option<unsafe extern "C" fn()>,
    ) {
    }
    
    pub unsafe fn JS_SetGCCallback(
        _cx: *mut RawJSContext,
        _callback: Option<unsafe extern "C" fn()>,
        _data: *mut c_void,
    ) {
    }
    
    pub unsafe fn JS_SetGCParameter(_cx: *mut RawJSContext, _param: u32, _value: u32) {
    }
    
    pub unsafe fn JS_SetGlobalJitCompilerOption(_cx: *mut RawJSContext, _opt: u32, _val: u32) {
    }
    
    pub unsafe fn JS_SetOffthreadIonCompilationEnabled(_cx: *mut RawJSContext, _enabled: bool) {
    }
    
    pub unsafe fn JS_SetSecurityCallbacks(
        _cx: *mut RawJSContext,
        _callbacks: *const super::super::jsapi::JSSecurityCallbacks,
    ) {
    }
    
    pub unsafe fn SetDOMCallbacks(_cx: *mut RawJSContext, _callbacks: *const c_void) {
    }
    
    pub unsafe fn SetGCSliceCallback(
        _cx: *mut RawJSContext,
        _callback: Option<unsafe extern "C" fn()>,
    ) {
    }
    
    pub unsafe fn SetJobQueue(_cx: *mut RawJSContext, _queue: *mut super::super::jsapi::JobQueue) {
    }
    
    pub unsafe fn SetPreserveWrapperCallbacks(
        _cx: *mut RawJSContext,
        _pre: Option<unsafe extern "C" fn() -> bool>,
        _post: Option<unsafe extern "C" fn() -> bool>,
    ) {
    }
    
    pub unsafe fn SetPromiseRejectionTrackerCallback(
        _cx: *mut RawJSContext,
        _callback: Option<unsafe extern "C" fn()>,
        _data: *mut c_void,
    ) {
    }
    
    pub unsafe fn SetUpEventLoopDispatch(_cx: *mut RawJSContext, _dispatch: *const c_void) {
    }
}
// ============================================================================
// Additional types for SpiderMonkey compatibility
// ============================================================================

/// Describe the scripted caller for error reporting
pub fn describe_scripted_caller(_cx: *mut RawJSContext) -> Option<ScriptedCaller> {
    None
}

/// Information about a scripted caller
#[derive(Debug, Clone)]
pub struct ScriptedCaller {
    /// Filename of the script
    pub filename: String,
    /// Line number
    pub line: u32,
    /// Column number  
    pub col: u32,
}

/// Check if a class is a DOM class
pub unsafe fn is_dom_class(_class: *const JSClass) -> bool {
    false
}

/// Get the class of an object
pub unsafe fn get_object_class(_obj: *mut JSObject) -> *const JSClass {
    ptr::null()
}

/// Structured clone buffer wrapper
pub struct JSAutoStructuredCloneBufferWrapper {
    _private: [u8; 0],
}

impl JSAutoStructuredCloneBufferWrapper {
    pub fn new() -> Self {
        Self { _private: [] }
    }
}

impl Default for JSAutoStructuredCloneBufferWrapper {
    fn default() -> Self {
        Self::new()
    }
}
/// CapturedJSStack - captured JavaScript stack trace
pub struct CapturedJSStack {
    _private: [u8; 0],
}

impl CapturedJSStack {
    pub fn new() -> Self {
        Self { _private: [] }
    }
    
    pub fn is_empty(&self) -> bool {
        true
    }
    
    pub fn as_str(&self) -> &str {
        ""
    }
}

impl Default for CapturedJSStack {
    fn default() -> Self {
        Self::new()
    }
}

/// IdVector - vector of JS property IDs
pub struct IdVector {
    ids: Vec<super::glue::jsid>,
}

impl IdVector {
    pub fn new(_cx: *mut RawJSContext) -> Self {
        Self { ids: Vec::new() }
    }
    
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    
    pub fn get(&self, index: usize) -> Option<&super::glue::jsid> {
        self.ids.get(index)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &super::glue::jsid> {
        self.ids.iter()
    }
}

impl Default for IdVector {
    fn default() -> Self {
        Self { ids: Vec::new() }
    }
}

// ===================
// Compile Options
// ===================

/// Wrapper for compile options
pub struct CompileOptionsWrapper<'a> {
    _marker: PhantomData<&'a ()>,
    filename: Option<String>,
    line: u32,
    column: u32,
    force_full_parse: bool,
    no_script_rval: bool,
}

impl<'a> CompileOptionsWrapper<'a> {
    pub fn new(_cx: *mut RawJSContext) -> Self {
        Self {
            _marker: PhantomData,
            filename: None,
            line: 1,
            column: 0,
            force_full_parse: false,
            no_script_rval: false,
        }
    }
    
    pub fn set_file(&mut self, filename: &str) {
        self.filename = Some(filename.to_string());
    }
    
    pub fn set_line(&mut self, line: u32) {
        self.line = line;
    }
    
    pub fn set_column(&mut self, column: u32) {
        self.column = column;
    }
    
    pub fn set_force_full_parse(&mut self, value: bool) {
        self.force_full_parse = value;
    }
    
    pub fn set_no_script_rval(&mut self, value: bool) {
        self.no_script_rval = value;
    }
}

// ===================
// Source Text Transformations
// ===================

/// Transform a u16 slice to source text (for UTF-16 compilation)
pub fn transform_u16_to_source_text(
    source: &[u16],
) -> super::jsapi::SourceText<u16> {
    let mut st = super::jsapi::SourceText::new();
    // The actual implementation would set up the source text properly
    let _ = source; // suppress unused warning
    st
}

/// Transform a str to source text (for UTF-8 compilation)
pub fn transform_str_to_source_text(
    source: &str,
) -> super::jsapi::SourceText<u8> {
    let mut st = super::jsapi::SourceText::new();
    // The actual implementation would set up the source text properly
    let _ = source; // suppress unused warning
    st
}

/// Transform a char16 slice to source text
pub fn transform_char16_to_source_text(
    source: &[u16],
) -> super::jsapi::SourceText<u16> {
    transform_u16_to_source_text(source)
}

// ===================
// Stencil
// ===================

/// Stencil - compiled script representation
pub struct Stencil {
    _private: [u8; 0],
}

impl Stencil {
    pub fn new() -> Self {
        Self { _private: [] }
    }
}

impl Default for Stencil {
    fn default() -> Self {
        Self::new()
    }
}

// ===================
// DOMClass
// ===================

/// DOM class descriptor for JS bindings
#[repr(C)]
pub struct DOMClass {
    /// Interface chain
    pub interface_chain: [u16; 8],
    /// Depth in prototype chain
    pub depth: u16,
    /// Type ID
    pub type_id: std::any::TypeId,
}