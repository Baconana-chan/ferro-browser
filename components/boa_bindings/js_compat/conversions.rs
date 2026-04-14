// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey conversions module compatibility layer for Boa

use std::ptr;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

use super::gc::Root as GcRoot;
use super::jsapi::{Heap, JSContext, RawJSContext, JSObject, JSString, Value};
use super::rust::{HandleValue, MutableHandleValue};
use super::typedarray::{ArrayBuffer, ArrayBufferView, ArrayBufferViewTrait, Float32Array, HeapArrayBuffer, HeapArrayBufferView};

/// Conversion result - Ok, Failed, or couldn't convert
#[derive(Debug, Clone)]
pub enum ConversionResult<T> {
    Success(T),
    Failure(String),
}

impl<T> ConversionResult<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, ConversionResult::Success(_))
    }
    
    pub fn is_failure(&self) -> bool {
        matches!(self, ConversionResult::Failure(_))
    }
    
    pub fn get(self) -> Option<T> {
        match self {
            ConversionResult::Success(v) => Some(v),
            ConversionResult::Failure(_) => None,
        }
    }
}

/// ConversionBehavior - how to handle type coercion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionBehavior {
    Default,
    Clamp,
    EnforceRange,
}

impl Default for ConversionBehavior {
    fn default() -> Self {
        ConversionBehavior::Default
    }
}

/// StringificationBehavior - how to handle string conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringificationBehavior {
    Default,
    Empty,
    Null,
}

impl Default for StringificationBehavior {
    fn default() -> Self {
        StringificationBehavior::Default
    }
}

/// ToJSValConvertible - trait for types that can be converted to JS values
pub trait ToJSValConvertible {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>);
}

impl ToJSValConvertible for bool {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_bool(*self));
    }
}

impl ToJSValConvertible for i8 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_i32(*self as i32));
    }
}

impl ToJSValConvertible for u8 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_i32(*self as i32));
    }
}

impl ToJSValConvertible for i16 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_i32(*self as i32));
    }
}

impl ToJSValConvertible for u16 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_i32(*self as i32));
    }
}

impl ToJSValConvertible for i32 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_i32(*self));
    }
}

impl ToJSValConvertible for u32 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        if *self <= i32::MAX as u32 {
            rval.set(Value::from_i32(*self as i32));
        } else {
            rval.set(Value::from_f64(*self as f64));
        }
    }
}

impl ToJSValConvertible for i64 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_f64(*self as f64));
    }
}

impl ToJSValConvertible for u64 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_f64(*self as f64));
    }
}

impl ToJSValConvertible for f32 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_f64(*self as f64));
    }
}

impl ToJSValConvertible for f64 {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::from_f64(*self));
    }
}

impl ToJSValConvertible for String {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        // SAFETY: we are in an unsafe fn, call unsafe fn safely
        unsafe { self.as_str().to_jsval(cx, rval) };
    }
}

impl ToJSValConvertible for str {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        // TODO: Create actual JS string
        rval.set(Value::undefined());
    }
}

impl ToJSValConvertible for () {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(Value::undefined());
    }
}

impl ToJSValConvertible for Value {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        rval.set(*self);
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        match self {
            // SAFETY: we are in an unsafe fn, call unsafe fn safely
            Some(v) => unsafe { v.to_jsval(cx, rval) },
            None => rval.set(Value::null()),
        }
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Vec<T> {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        // TODO: Create actual JS array
        rval.set(Value::undefined());
    }
}

impl<T: ToJSValConvertible + ?Sized> ToJSValConvertible for Rc<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { (**self).to_jsval(cx, rval) };
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for GcRoot<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { (**self).to_jsval(cx, rval) };
    }
}

impl ToJSValConvertible for *mut JSObject {
    unsafe fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        if self.is_null() {
            rval.set(Value::null());
        } else {
            rval.set(Value::from_object(*self));
        }
    }
}

impl ToJSValConvertible for NonNull<JSObject> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { self.as_ptr().to_jsval(cx, rval) };
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for RootedTraceableBox<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { (**self).to_jsval(cx, rval) };
    }
}

impl<T: Copy + ToJSValConvertible> ToJSValConvertible for Heap<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { self.get().to_jsval(cx, rval) };
    }
}

impl ToJSValConvertible for ArrayBuffer {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { self.underlying_object().to_jsval(cx, rval) };
    }
}

impl ToJSValConvertible for ArrayBufferView {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { self.underlying_object().to_jsval(cx, rval) };
    }
}

impl ToJSValConvertible for HeapArrayBuffer {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { self.underlying_object().get().to_jsval(cx, rval) };
    }
}

impl ToJSValConvertible for HeapArrayBufferView {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { self.underlying_object().get().to_jsval(cx, rval) };
    }
}

impl ToJSValConvertible for Float32Array {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue<'_>) {
        unsafe { self.underlying_object().to_jsval(cx, rval) };
    }
}

/// FromJSValConvertible - trait for types that can be converted from JS values
pub trait FromJSValConvertible: Sized {
    type Config;
    
    unsafe fn from_jsval(
        cx: *mut JSContext,
        val: HandleValue<'_>,
        config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()>;
}

impl FromJSValConvertible for bool {
    type Config = ();
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_boolean()))
    }
}

impl FromJSValConvertible for i8 {
    type Config = ConversionBehavior;
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_i32() as i8))
    }
}

impl FromJSValConvertible for u8 {
    type Config = ConversionBehavior;
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_i32() as u8))
    }
}

impl FromJSValConvertible for i16 {
    type Config = ConversionBehavior;
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_i32() as i16))
    }
}

impl FromJSValConvertible for u16 {
    type Config = ConversionBehavior;
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_i32() as u16))
    }
}

impl FromJSValConvertible for i32 {
    type Config = ConversionBehavior;
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_i32()))
    }
}

impl FromJSValConvertible for u32 {
    type Config = ConversionBehavior;
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_i32() as u32))
    }
}

impl FromJSValConvertible for i64 {
    type Config = ConversionBehavior;
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_f64() as i64))
    }
}

impl FromJSValConvertible for u64 {
    type Config = ConversionBehavior;
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_f64() as u64))
    }
}

impl FromJSValConvertible for f32 {
    type Config = ();
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_f64() as f32))
    }
}

impl FromJSValConvertible for f64 {
    type Config = ();
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get().to_f64()))
    }
}

impl FromJSValConvertible for String {
    type Config = ();
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        _val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        // TODO: Extract actual string from JS value
        Ok(ConversionResult::Success(String::new()))
    }
}

impl FromJSValConvertible for *mut JSObject {
    type Config = ();
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        // Extract object pointer from value
        Ok(ConversionResult::Success(val.get().to_object_or_null()))
    }
}

impl FromJSValConvertible for NonNull<JSObject> {
    type Config = ();
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        match NonNull::new(val.get().to_object_or_null()) {
            Some(value) => Ok(ConversionResult::Success(value)),
            None => Ok(ConversionResult::Failure("expected non-null object".into())),
        }
    }
}

impl FromJSValConvertible for Value {
    type Config = ();
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Success(val.get()))
    }
}

impl<T: FromJSValConvertible> FromJSValConvertible for Option<T> {
    type Config = T::Config;
    
    unsafe fn from_jsval(
        cx: *mut JSContext,
        val: HandleValue<'_>,
        config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        let value = val.get();
        if value.is_null_or_undefined() {
            return Ok(ConversionResult::Success(None));
        }
        
        match unsafe { T::from_jsval(cx, val, config) }? {
            ConversionResult::Success(value) => Ok(ConversionResult::Success(Some(value))),
            ConversionResult::Failure(error) => Ok(ConversionResult::Failure(error)),
        }
    }
}

impl<T: FromJSValConvertible> FromJSValConvertible for Vec<T> {
    type Config = T::Config;
    
    unsafe fn from_jsval(
        _cx: *mut JSContext,
        _val: HandleValue<'_>,
        _config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        Ok(ConversionResult::Failure(
            "array conversion is not implemented in Boa compat layer".into(),
        ))
    }
}

/// Convert a JSString to a Rust String
/// Accepts both *mut JSString and NonNull<JSString>
pub unsafe fn jsstr_to_string<T: IntoJSStringPtr>(_cx: *mut JSContext, s: T) -> String {
    let _ptr = s.into_jsstring_ptr();
    String::new()
}

/// Trait for types that can be converted to *mut JSString
pub trait IntoJSStringPtr {
    fn into_jsstring_ptr(self) -> *mut JSString;
}

impl IntoJSStringPtr for *mut JSString {
    fn into_jsstring_ptr(self) -> *mut JSString {
        self
    }
}

impl IntoJSStringPtr for std::ptr::NonNull<JSString> {
    fn into_jsstring_ptr(self) -> *mut JSString {
        self.as_ptr()
    }
}

/// Convert a Latin1 encoded buffer to a String
pub fn latin1_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

/// ForOfIterator - iterator for for-of loops
pub struct ForOfIterator<'a> {
    _cx: *mut JSContext,
    _marker: PhantomData<&'a ()>,
}

impl<'a> ForOfIterator<'a> {
    pub fn new(_cx: *mut JSContext) -> Self {
        Self {
            _cx,
            _marker: PhantomData,
        }
    }
    
    pub unsafe fn init(
        &mut self,
        _iterable: HandleValue<'_>,
        _allow_async: AllowAsync,
    ) -> bool {
        true
    }
    
    pub unsafe fn next(&mut self, _val: MutableHandleValue<'_>, _done: *mut bool) -> bool {
        // SAFETY: we are in unsafe fn, dereference is safe
        unsafe { *_done = true };
        true
    }
}

/// AllowAsync - whether async iteration is allowed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllowAsync {
    AllowAsync,
    DisallowAsync,
}

impl Default for AllowAsync {
    fn default() -> Self {
        AllowAsync::DisallowAsync
    }
}

/// IDLInterface marker trait
pub trait IDLInterface {
    fn derives(_proto: u16) -> bool { false }
}

/// RootedTraceableBox for boxing traceable values
#[repr(transparent)]
pub struct RootedTraceableBox<T> {
    value: T,
}

impl<T> RootedTraceableBox<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
    
    pub fn from_box(value: Box<T>) -> Self {
        Self { value: *value }
    }
}

impl<T> std::ops::Deref for RootedTraceableBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for RootedTraceableBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// FromJSValConvertibleRc - trait for converting from JS values to Rc types
pub trait FromJSValConvertibleRc: Sized {
    /// Convert from a JS value to an Rc
    unsafe fn from_jsval_rc(
        cx: *mut JSContext,
        val: HandleValue<'_>,
    ) -> ConversionResult<std::rc::Rc<Self>>;
}

/// root_from_handlevalue - get root from handle value
pub unsafe fn root_from_handlevalue<T>(
    _val: HandleValue<'_>,
    _cx: *mut JSContext,
) -> Option<T> {
    // Stub - would extract DOM object from JS value
    None
}

/// root_from_object - get root from object
pub unsafe fn root_from_object<T>(
    _obj: *mut JSObject,
    _cx: *mut JSContext,
) -> Option<T> {
    // Stub - would extract DOM object from JS object
    None
}

/// is_array_like - check if value is array-like
pub unsafe fn is_array_like(
    _cx: *mut JSContext,
    _val: HandleValue<'_>,
) -> bool {
    false
}

/// Get DOM class from object
pub unsafe fn get_dom_class(_obj: *mut JSObject) -> Option<&'static super::rust::DOMClass> {
    None
}

/// Get private from object (DOM object pointer)
pub unsafe fn private_from_object(_obj: *mut JSObject) -> *const std::ffi::c_void {
    std::ptr::null()
}

/// Convert jsid to string
pub unsafe fn jsid_to_string(
    _cx: *mut JSContext,
    _id: super::glue::HandleId<'_>,
) -> Option<String> {
    None
}
