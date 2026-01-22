// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey jsapi compatibility layer for Boa

use std::ptr;
use std::ffi::c_void;
use std::marker::PhantomData;
// Note: We don't use Boa's Context directly here, we define our own opaque types

/// Opaque JSContext equivalent - wraps Boa Context
pub struct JSContext {
    inner: *mut c_void,
}

impl JSContext {
    pub fn new() -> Self {
        Self { inner: ptr::null_mut() }
    }
    
    pub fn as_ptr(&self) -> *mut RawJSContext {
        self.inner as *mut RawJSContext
    }
}

/// Raw JSContext pointer type
pub type RawJSContext = c_void;

/// JSObject - represents a JavaScript object
#[repr(C)]
pub struct JSObject {
    _private: [u8; 0],
}

impl JSObject {
    pub fn is_null(&self) -> bool {
        (self as *const Self).is_null()
    }
}

/// JSString - represents a JavaScript string
#[repr(C)]
pub struct JSString {
    _private: [u8; 0],
}

/// JSScript - represents compiled JavaScript
#[repr(C)]
pub struct JSScript {
    _private: [u8; 0],
}

/// JSTracer - for garbage collection tracing
#[repr(C)]
pub struct JSTracer {
    _private: [u8; 0],
}

/// JSClass - class definition for JS objects
#[repr(C)]
pub struct JSClass {
    pub name: *const i8,
    pub flags: u32,
    pub c_ops: *const JSClassOps,
    pub spec: *const c_void,
    pub ext: *const c_void,
    pub o_ops: *const c_void,
}

unsafe impl Sync for JSClass {}

/// JSClassOps - operations for JSClass
#[repr(C)]
pub struct JSClassOps {
    pub add_property: Option<unsafe extern "C" fn()>,
    pub del_property: Option<unsafe extern "C" fn()>,
    pub get_property: Option<unsafe extern "C" fn()>,
    pub set_property: Option<unsafe extern "C" fn()>,
    pub enumerate: Option<unsafe extern "C" fn()>,
    pub new_enumerate: Option<unsafe extern "C" fn()>,
    pub resolve: Option<unsafe extern "C" fn()>,
    pub may_resolve: Option<unsafe extern "C" fn()>,
    pub finalize: Option<unsafe extern "C" fn()>,
    pub call: Option<unsafe extern "C" fn()>,
    pub has_instance: Option<unsafe extern "C" fn()>,
    pub construct: Option<unsafe extern "C" fn()>,
    pub trace: Option<unsafe extern "C" fn()>,
}

/// Handle types - safe wrappers around raw pointers
#[repr(transparent)]
pub struct Handle<'a, T> {
    ptr: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Handle<'a, T> {
    pub fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }
    
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        Self { ptr, _marker: PhantomData }
    }
}

impl<'a, T> Clone for Handle<'a, T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr, _marker: PhantomData }
    }
}

impl<'a, T> Copy for Handle<'a, T> {}

/// HandleObject - Handle to a JSObject
pub type HandleObject<'a> = Handle<'a, *mut JSObject>;
pub type RawHandleObject = *mut JSObject;

/// HandleValue - Handle to a Value
pub type HandleValue<'a> = Handle<'a, Value>;
pub type RawHandleValue = *mut Value;

/// HandleString - Handle to a JSString
pub type HandleString<'a> = Handle<'a, *mut JSString>;

/// MutableHandle types
#[repr(transparent)]
pub struct MutableHandle<'a, T> {
    ptr: *mut T,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> MutableHandle<'a, T> {
    pub fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }
    
    pub fn set(&mut self, val: T) {
        unsafe { *self.ptr = val; }
    }
    
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self { ptr, _marker: PhantomData }
    }
}

/// MutableHandleObject
pub type MutableHandleObject<'a> = MutableHandle<'a, *mut JSObject>;

/// MutableHandleString
pub type MutableHandleString<'a> = MutableHandle<'a, *mut JSString>;

/// MutableHandleValue
pub type MutableHandleValue<'a> = MutableHandle<'a, Value>;

/// Value - JavaScript value (NaN-boxed in SpiderMonkey, we use Boa's JsValue)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Value {
    pub data: u64,
}

impl Value {
    pub const fn undefined() -> Self {
        Self { data: 0 }
    }
    
    pub const fn null() -> Self {
        Self { data: 1 }
    }
    
    pub fn is_undefined(&self) -> bool {
        self.data == 0
    }
    
    pub fn is_null(&self) -> bool {
        self.data == 1
    }
    
    pub fn is_object(&self) -> bool {
        self.data >= 0x1000
    }
    
    pub fn is_boolean(&self) -> bool {
        (self.data & 0xFFFF_FFFF_FFFF_0000) == 0x0001_0000
    }
    
    pub fn is_int32(&self) -> bool {
        (self.data & 0xFFFF_FFFF_0000_0000) == 0x0002_0000_0000_0000
    }
    
    pub fn is_double(&self) -> bool {
        (self.data & 0xFFFF_0000_0000_0000) == 0x0003_0000_0000_0000
    }
    
    pub fn is_string(&self) -> bool {
        (self.data & 0xFFFF_0000_0000_0000) == 0x0004_0000_0000_0000
    }
    
    pub fn is_symbol(&self) -> bool {
        (self.data & 0xFFFF_0000_0000_0000) == 0x0005_0000_0000_0000
    }
    
    pub fn is_bigint(&self) -> bool {
        (self.data & 0xFFFF_0000_0000_0000) == 0x0006_0000_0000_0000
    }
    
    pub fn to_boolean(&self) -> bool {
        if self.is_boolean() {
            (self.data & 1) != 0
        } else {
            false
        }
    }
    
    pub fn to_i32(&self) -> i32 {
        if self.is_int32() {
            (self.data & 0xFFFF_FFFF) as i32
        } else {
            0
        }
    }
    
    pub fn to_f64(&self) -> f64 {
        if self.is_double() {
            f64::from_bits(self.data)
        } else if self.is_int32() {
            self.to_i32() as f64
        } else {
            f64::NAN
        }
    }
    
    pub fn to_object(&self) -> *mut JSObject {
        if self.is_object() {
            self.data as *mut JSObject
        } else {
            ptr::null_mut()
        }
    }
    
    pub fn from_bool(b: bool) -> Self {
        Self { data: 0x0001_0000 | (b as u64) }
    }
    
    pub fn from_i32(i: i32) -> Self {
        Self { data: 0x0002_0000_0000_0000 | ((i as u32) as u64) }
    }
    
    pub fn from_f64(f: f64) -> Self {
        Self { data: f.to_bits() }
    }
    
    pub fn from_object(obj: *mut JSObject) -> Self {
        Self { data: obj as u64 }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::undefined()
    }
}

/// Heap - GC-traced storage for values
#[repr(C)]
pub struct Heap<T> {
    value: T,
}

impl<T: Default> Heap<T> {
    pub fn new() -> Self {
        Self { value: T::default() }
    }
    
    pub fn get(&self) -> T where T: Copy {
        self.value
    }
    
    pub fn set(&mut self, val: T) {
        self.value = val;
    }
    
    pub fn handle(&self) -> Handle<'_, T> {
        Handle {
            ptr: &self.value,
            _marker: PhantomData,
        }
    }
}

impl<T: Default> Default for Heap<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// JSAutoRealm - RAII guard for entering a realm
pub struct JSAutoRealm {
    _private: (),
}

impl JSAutoRealm {
    pub fn new(_cx: *mut RawJSContext, _obj: *mut JSObject) -> Self {
        Self { _private: () }
    }
}

/// CallArgs - arguments passed to a JS function
pub struct CallArgs {
    argc: u32,
    vp: *mut Value,
}

impl CallArgs {
    pub fn argc(&self) -> u32 {
        self.argc
    }
    
    pub fn get(&self, i: u32) -> HandleValue<'_> {
        unsafe {
            Handle::from_raw(self.vp.add(2 + i as usize))
        }
    }
    
    pub fn rval(&self) -> MutableHandleValue<'_> {
        unsafe {
            MutableHandle::from_raw(self.vp)
        }
    }
    
    pub fn this(&self) -> HandleValue<'_> {
        unsafe {
            Handle::from_raw(self.vp.add(1))
        }
    }
}

/// HandleValueArray - array of HandleValues
pub struct HandleValueArray {
    length: usize,
    elements: *const Value,
}

impl HandleValueArray {
    pub fn new() -> Self {
        Self { length: 0, elements: ptr::null() }
    }
    
    pub fn from_rooted_slice(slice: &[Value]) -> Self {
        Self {
            length: slice.len(),
            elements: slice.as_ptr(),
        }
    }
    
    pub fn len(&self) -> usize {
        self.length
    }
}

// GC-related types
pub type GCReason = u32;
pub type GCOptions = u32;
pub type GCProgress = u32;
pub type GCDescription = u32;
pub type JSGCStatus = u32;
pub type JSGCParamKey = u32;
pub type TraceKind = u32;

// Compilation types
pub type CompilationType = u32;
pub type RuntimeCode = u32;
pub type MimeType = u32;

// Promise types
pub type PromiseRejectionHandlingState = u32;
pub type PromiseUserInputEventHandlingState = u32;

// Security
#[repr(C)]
pub struct JSSecurityCallbacks {
    pub content_security_policy_allows: Option<unsafe extern "C" fn() -> bool>,
    pub subsumes: Option<unsafe extern "C" fn() -> bool>,
}

/// JobQueue for promise jobs
#[repr(C)]
pub struct JobQueue {
    _private: [u8; 0],
}

/// BuildIdCharVector
#[repr(C)]
pub struct BuildIdCharVector {
    _private: [u8; 0],
}

/// AsmJS options
pub type AsmJSOption = u32;

/// Exception stack behavior
pub type ExceptionStackBehavior = u32;

/// JSJitCompilerOption
pub type JSJitCompilerOption = u32;

// Function stubs
pub unsafe fn JS_NewObject(_cx: *mut RawJSContext, _class: *const JSClass) -> *mut JSObject {
    ptr::null_mut()
}

pub unsafe fn JS_NewPlainObject(_cx: *mut RawJSContext) -> *mut JSObject {
    ptr::null_mut()
}

pub unsafe fn JS_NewStringCopyUTF8N(
    _cx: *mut RawJSContext,
    _s: *const i8,
    _len: usize,
) -> *mut JSString {
    ptr::null_mut()
}

pub unsafe fn JS_SetReservedSlot(_obj: *mut JSObject, _slot: u32, _val: Value) {}

pub unsafe fn CurrentGlobalOrNull(_cx: *mut RawJSContext) -> *mut JSObject {
    ptr::null_mut()
}

pub unsafe fn IsCallable(_obj: *mut JSObject) -> bool {
    false
}

pub unsafe fn IsConstructor(_obj: *mut JSObject) -> bool {
    false
}

pub unsafe fn IsPromiseObject(_obj: HandleObject<'_>) -> bool {
    false
}

pub unsafe fn NewArrayObject(_cx: *mut RawJSContext, _contents: &HandleValueArray) -> *mut JSObject {
    ptr::null_mut()
}

pub unsafe fn GetPromiseUserInputEventHandlingState(
    _promise: HandleObject<'_>,
) -> PromiseUserInputEventHandlingState {
    0
}

pub unsafe fn GetObjectRealmOrNull(_obj: *mut JSObject) -> *mut c_void {
    ptr::null_mut()
}

pub unsafe fn GetRealmPrincipals(_realm: *mut c_void) -> *mut c_void {
    ptr::null_mut()
}

pub unsafe fn GetScriptedCallerGlobal(_cx: *mut RawJSContext) -> *mut JSObject {
    ptr::null_mut()
}

pub unsafe fn JS_ClearPendingException(_cx: *mut RawJSContext) {}

pub unsafe fn JS_IsExceptionPending(_cx: *mut RawJSContext) -> bool {
    false
}

pub unsafe fn GCTraceKindToAscii(_kind: TraceKind) -> *const i8 {
    b"unknown\0".as_ptr() as *const i8
}

/// Dispatchable for event loop
#[repr(C)]
pub struct Dispatchable {
    _private: [u8; 0],
}

pub type Dispatchable_MaybeShuttingDown = u32;
