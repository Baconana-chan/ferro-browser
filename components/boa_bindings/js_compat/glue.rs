// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey glue module compatibility layer for Boa

use std::ptr;
use std::ffi::c_void;

use super::jsapi::{JSContext, RawJSContext, JSObject, JSString, JSTracer, Value, JSClass};
use super::rust::{HandleObject, HandleValue, MutableHandleValue, MutableHandleObject};

/// Get reserved slot from object
pub unsafe fn GetReservedSlot(_obj: *mut JSObject, _slot: u32) -> Value {
    Value::undefined()
}

/// Set reserved slot on object
pub unsafe fn SetReservedSlot(_obj: *mut JSObject, _slot: u32, _val: Value) {
}

/// Get object class
pub unsafe fn GetObjectClass(_obj: *mut JSObject) -> *const JSClass {
    ptr::null()
}

/// Get object prototype
pub unsafe fn GetObjectPrototype(_cx: *mut RawJSContext, _obj: HandleObject<'_>) -> *mut JSObject {
    ptr::null_mut()
}

/// Get object realm
pub unsafe fn GetObjectRealm(_obj: *mut JSObject) -> *mut c_void {
    ptr::null_mut()
}

/// Get object global
pub unsafe fn GetNonCCWObjectGlobal(_obj: *mut JSObject) -> *mut JSObject {
    ptr::null_mut()
}

/// Unwrap object (remove wrappers)
pub unsafe fn UnwrapObject(_obj: *mut JSObject, _stopAtWindowProxy: bool) -> *mut JSObject {
    ptr::null_mut()
}

/// Unwrap object without CCW
pub unsafe fn UnwrapObjectNoThrow(_obj: *mut JSObject) -> *mut JSObject {
    ptr::null_mut()
}

/// Is wrapper object
pub unsafe fn IsWrapper(_obj: *mut JSObject) -> bool {
    false
}

/// Is DOM object
pub unsafe fn IsDOMObject(_obj: *mut JSObject) -> bool {
    false
}

/// Get DOM class
pub unsafe fn GetDOMClass(_obj: *mut JSObject) -> *const c_void {
    ptr::null()
}

/// Get DOM private
pub unsafe fn GetDOMPrivate(_obj: *mut JSObject) -> *mut c_void {
    ptr::null_mut()
}

/// Set DOM private
pub unsafe fn SetDOMPrivate(_obj: *mut JSObject, _priv: *mut c_void) {
}

/// JSID type
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct jsid {
    pub bits: usize,
}

impl jsid {
    pub const VOID: Self = Self { bits: 0 };
    
    pub fn is_void(&self) -> bool {
        self.bits == 0
    }
    
    pub fn is_int(&self) -> bool {
        (self.bits & 1) != 0
    }
    
    pub fn is_string(&self) -> bool {
        !self.is_int() && !self.is_void()
    }
}

/// JSID_VOID constant
pub const JSID_VOID: jsid = jsid::VOID;

/// PropertyKey - alias for jsid
pub type PropertyKey = jsid;

/// Property descriptor
#[repr(C)]
pub struct PropertyDescriptor {
    pub value: Value,
    pub getter: *mut JSObject,
    pub setter: *mut JSObject,
    pub attrs: u32,
}

impl Default for PropertyDescriptor {
    fn default() -> Self {
        Self {
            value: Value::undefined(),
            getter: ptr::null_mut(),
            setter: ptr::null_mut(),
            attrs: 0,
        }
    }
}

/// HandleId
pub type HandleId<'a> = super::rust::Handle<'a, jsid>;

/// MutableHandleId
pub type MutableHandleId<'a> = super::rust::MutableHandle<'a, jsid>;

/// GCDescription
#[repr(C)]
pub struct GCDescription {
    pub invocation_kind: u32,
    pub reason: u32,
}

/// GCOptions
#[repr(C)]
pub struct GCOptions {
    pub options: u32,
}

impl Default for GCOptions {
    fn default() -> Self {
        Self { options: 0 }
    }
}

/// JSType - type tag for JS values
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JSType {
    Undefined = 0,
    Object = 1,
    Function = 2,
    String = 3,
    Number = 4,
    Boolean = 5,
    Null = 6,
    Symbol = 7,
    BigInt = 8,
}

/// Get the type of a value
pub unsafe fn JS_TypeOfValue(_cx: *mut RawJSContext, v: Value) -> JSType {
    if v.is_undefined() {
        JSType::Undefined
    } else if v.is_null() {
        JSType::Null
    } else if v.is_boolean() {
        JSType::Boolean
    } else if v.is_int32() || v.is_double() {
        JSType::Number
    } else if v.is_string() {
        JSType::String
    } else if v.is_object() {
        JSType::Object
    } else {
        JSType::Undefined
    }
}

/// JS_GetLatin1StringCharsAndLength
pub unsafe fn JS_GetLatin1StringCharsAndLength(
    _cx: *mut RawJSContext,
    _nogc: *const c_void,
    _str: *mut JSString,
    _length: *mut usize,
) -> *const u8 {
    ptr::null()
}

/// JS_GetTwoByteStringCharsAndLength
pub unsafe fn JS_GetTwoByteStringCharsAndLength(
    _cx: *mut RawJSContext,
    _nogc: *const c_void,
    _str: *mut JSString,
    _length: *mut usize,
) -> *const u16 {
    ptr::null()
}

/// JS_StringHasLatin1Chars
pub unsafe fn JS_StringHasLatin1Chars(_str: *mut JSString) -> bool {
    true
}

/// JS_GetStringLength
pub unsafe fn JS_GetStringLength(_str: *mut JSString) -> usize {
    0
}

/// AppendToIdVector
pub unsafe fn AppendToIdVector(
    _ids: *mut c_void,
    _id: jsid,
) -> bool {
    true
}

/// JSFlatString - a flattened string
pub type JSFlatString = JSString;

/// JS_FlattenString
pub unsafe fn JS_FlattenString(_cx: *mut RawJSContext, _str: *mut JSString) -> *mut JSFlatString {
    ptr::null_mut()
}

/// GetFlatStringChars
pub unsafe fn GetFlatStringChars(_flat: *const JSFlatString) -> *const u16 {
    ptr::null()
}

/// JS_NewStringCopyN
pub unsafe fn JS_NewStringCopyN(_cx: *mut RawJSContext, _chars: *const i8, _len: usize) -> *mut JSString {
    ptr::null_mut()
}

/// JS_NewUCStringCopyN
pub unsafe fn JS_NewUCStringCopyN(_cx: *mut RawJSContext, _chars: *const u16, _len: usize) -> *mut JSString {
    ptr::null_mut()
}

/// JS_AtomizeAndPinString
pub unsafe fn JS_AtomizeAndPinString(_cx: *mut RawJSContext, _chars: *const i8) -> *mut JSString {
    ptr::null_mut()
}

/// JSPrincipals - security principals
#[repr(C)]
pub struct JSPrincipals {
    pub refcount: i32,
}

impl JSPrincipals {
    pub const fn new() -> Self {
        Self { refcount: 1 }
    }
}

/// Evaluate script
pub unsafe fn Evaluate(
    _cx: *mut RawJSContext,
    _options: *const c_void,
    _src: *const u16,
    _len: usize,
    _rval: MutableHandleValue<'_>,
) -> bool {
    true
}

/// Compile script
pub unsafe fn Compile(
    _cx: *mut RawJSContext,
    _options: *const c_void,
    _src: *const u16,
    _len: usize,
    _script: MutableHandleObject<'_>,
) -> bool {
    true
}

/// Symbol types
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
    Species = 7,
    ToPrimitive = 8,
    ToStringTag = 9,
    Unscopables = 10,
    AsyncIterator = 11,
    MatchAll = 12,
    Length = 13,
}

/// GetWellKnownSymbol
pub unsafe fn GetWellKnownSymbol(_cx: *mut RawJSContext, _which: SymbolCode) -> *mut JSString {
    ptr::null_mut()
}

/// Proxy handling types
pub type ProxyHandler = c_void;

/// Create a new proxy
pub unsafe fn NewProxyObject(
    _cx: *mut RawJSContext,
    _handler: *const ProxyHandler,
    _priv: HandleValue<'_>,
    _proto: HandleObject<'_>,
    _clasp: *const JSClass,
) -> *mut JSObject {
    ptr::null_mut()
}

/// Get proxy private
pub unsafe fn GetProxyPrivate(_obj: *mut JSObject) -> Value {
    Value::undefined()
}

/// Set proxy private
pub unsafe fn SetProxyPrivate(_obj: *mut JSObject, _priv: Value) {
}

/// Get proxy handler
pub unsafe fn GetProxyHandler(_obj: *mut JSObject) -> *const ProxyHandler {
    ptr::null()
}

/// Is proxy
pub unsafe fn IsProxy(_obj: *mut JSObject) -> bool {
    false
}
