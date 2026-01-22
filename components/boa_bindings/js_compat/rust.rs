// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey rust module compatibility layer for Boa

use std::ptr;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::jsapi::{
    JSObject, JSContext, RawJSContext, JSString, JSTracer, Value,
    Handle as RawHandle, MutableHandle as RawMutableHandle,
};

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
