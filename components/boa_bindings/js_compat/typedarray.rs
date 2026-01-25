// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey typedarray module compatibility layer for Boa

use std::ptr;
use std::slice;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::jsapi::{JSContext, RawJSContext, JSObject, Value};
use super::rust::{Handle, HandleObject, HandleValue, MutableHandle, MutableHandleObject};

/// ArrayBufferViewContents - type for typed array contents
pub struct ArrayBufferViewContents<T> {
    ptr: *const T,
    len: usize,
}

impl<T> ArrayBufferViewContents<T> {
    pub fn as_slice(&self) -> &[T] {
        if self.ptr.is_null() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.ptr, self.len) }
        }
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// CreateWith - enum for creating typed arrays
#[derive(Clone, Copy)]
pub enum CreateWith<'a, T> {
    Length(usize),
    Slice(&'a [T]),
}

/// TypedArray trait
pub trait TypedArray: Sized {
    type Element;
    
    fn create(
        _cx: *mut RawJSContext,
        _with: CreateWith<Self::Element>,
        _res: MutableHandleObject<'_>,
    ) -> Result<(), ()> {
        Err(())
    }
    
    unsafe fn from_object(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
    
    fn as_slice(&self) -> &[Self::Element] {
        &[]
    }
    
    fn as_mut_slice(&mut self) -> &mut [Self::Element] {
        &mut []
    }
    
    fn len(&self) -> usize {
        0
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        ptr::null_mut()
    }
}

/// ArrayBuffer - JavaScript ArrayBuffer
pub struct ArrayBuffer {
    obj: *mut JSObject,
    data: Vec<u8>,
}

impl ArrayBuffer {
    pub fn create(_cx: *mut RawJSContext, len: usize, _res: MutableHandleObject<'_>) -> bool {
        let _ = len;
        true
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
    
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
    
    pub fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
    
    pub unsafe fn create_external(
        _cx: *mut RawJSContext,
        _data: *mut u8,
        _len: usize,
        _free_func: Option<unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void)>,
        _free_user_data: *mut std::ffi::c_void,
        _res: MutableHandleObject<'_>,
    ) -> bool {
        true
    }
    
    pub unsafe fn is_detached(&self) -> bool {
        false
    }
}

/// ArrayBufferU8 - ArrayBuffer with u8 element type
pub type ArrayBufferU8 = ArrayBuffer;

/// ArrayBufferView - base trait for typed array views
pub trait ArrayBufferView {
    fn underlying_object(&self) -> *mut JSObject;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn byte_offset(&self) -> usize { 0 }
    fn byte_length(&self) -> usize;
    fn is_shared_memory(&self) -> bool { false }
}

/// Uint8Array - JavaScript Uint8Array
pub struct Uint8Array {
    obj: *mut JSObject,
    data: Vec<u8>,
}

impl Uint8Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for Uint8Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for Uint8Array {
    type Element = u8;
    
    fn as_slice(&self) -> &[u8] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

impl ArrayBufferView for Uint8Array {
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn byte_length(&self) -> usize {
        self.data.len()
    }
}

/// Uint8ClampedArray - JavaScript Uint8ClampedArray
pub struct Uint8ClampedArray {
    obj: *mut JSObject,
    data: Vec<u8>,
}

impl Uint8ClampedArray {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for Uint8ClampedArray {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for Uint8ClampedArray {
    type Element = u8;
    
    fn as_slice(&self) -> &[u8] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

impl ArrayBufferView for Uint8ClampedArray {
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn byte_length(&self) -> usize {
        self.data.len()
    }
}

/// Int8Array
pub struct Int8Array {
    obj: *mut JSObject,
    data: Vec<i8>,
}

impl Int8Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for Int8Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for Int8Array {
    type Element = i8;
    
    fn as_slice(&self) -> &[i8] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [i8] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

/// Uint16Array
pub struct Uint16Array {
    obj: *mut JSObject,
    data: Vec<u16>,
}

impl Uint16Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for Uint16Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for Uint16Array {
    type Element = u16;
    
    fn as_slice(&self) -> &[u16] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [u16] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

/// Int16Array
pub struct Int16Array {
    obj: *mut JSObject,
    data: Vec<i16>,
}

impl Int16Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for Int16Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for Int16Array {
    type Element = i16;
    
    fn as_slice(&self) -> &[i16] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [i16] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

/// Uint32Array
pub struct Uint32Array {
    obj: *mut JSObject,
    data: Vec<u32>,
}

impl Uint32Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for Uint32Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for Uint32Array {
    type Element = u32;
    
    fn as_slice(&self) -> &[u32] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [u32] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

/// Int32Array
pub struct Int32Array {
    obj: *mut JSObject,
    data: Vec<i32>,
}

impl Int32Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for Int32Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for Int32Array {
    type Element = i32;
    
    fn as_slice(&self) -> &[i32] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [i32] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

/// Float32Array
pub struct Float32Array {
    obj: *mut JSObject,
    data: Vec<f32>,
}

impl Float32Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for Float32Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for Float32Array {
    type Element = f32;
    
    fn as_slice(&self) -> &[f32] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

impl ArrayBufferView for Float32Array {
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn byte_length(&self) -> usize {
        self.data.len() * 4
    }
}

/// Float64Array
pub struct Float64Array {
    obj: *mut JSObject,
    data: Vec<f64>,
}

impl Float64Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for Float64Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for Float64Array {
    type Element = f64;
    
    fn as_slice(&self) -> &[f64] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [f64] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

impl ArrayBufferView for Float64Array {
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn byte_length(&self) -> usize {
        self.data.len() * 8
    }
}

/// BigInt64Array
pub struct BigInt64Array {
    obj: *mut JSObject,
    data: Vec<i64>,
}

impl BigInt64Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for BigInt64Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for BigInt64Array {
    type Element = i64;
    
    fn as_slice(&self) -> &[i64] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [i64] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

/// BigUint64Array
pub struct BigUint64Array {
    obj: *mut JSObject,
    data: Vec<u64>,
}

impl BigUint64Array {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for BigUint64Array {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedArray for BigUint64Array {
    type Element = u64;
    
    fn as_slice(&self) -> &[u64] {
        &self.data
    }
    
    fn as_mut_slice(&mut self) -> &mut [u64] {
        &mut self.data
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
}

/// DataView
pub struct DataView {
    obj: *mut JSObject,
    data: Vec<u8>,
}

impl DataView {
    pub fn new() -> Self {
        Self { obj: ptr::null_mut(), data: Vec::new() }
    }
    
    pub unsafe fn from(_obj: HandleObject<'_>) -> Option<Self> {
        None
    }
}

impl Default for DataView {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrayBufferView for DataView {
    fn underlying_object(&self) -> *mut JSObject {
        self.obj
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn byte_length(&self) -> usize {
        self.data.len()
    }
}

/// ClampedU8 - clamped u8 for Uint8ClampedArray
#[derive(Clone, Copy, Debug, Default)]
pub struct ClampedU8(pub u8);

impl From<u8> for ClampedU8 {
    fn from(v: u8) -> Self {
        ClampedU8(v)
    }
}

impl From<ClampedU8> for u8 {
    fn from(v: ClampedU8) -> Self {
        v.0
    }
}

impl From<i32> for ClampedU8 {
    fn from(v: i32) -> Self {
        if v < 0 {
            ClampedU8(0)
        } else if v > 255 {
            ClampedU8(255)
        } else {
            ClampedU8(v as u8)
        }
    }
}

impl From<f64> for ClampedU8 {
    fn from(v: f64) -> Self {
        if v.is_nan() {
            ClampedU8(0)
        } else if v < 0.0 {
            ClampedU8(0)
        } else if v > 255.0 {
            ClampedU8(255)
        } else {
            ClampedU8(v.round() as u8)
        }
    }
}
// ============================================================================
// Additional type aliases and traits for SpiderMonkey compatibility
// ============================================================================

/// Type alias for Uint8 typed array (SpiderMonkey compatibility)
pub type Uint8 = Uint8Array;

/// Type alias for Float32 typed array (SpiderMonkey compatibility)
pub type Float32 = Float32Array;

/// Type alias for Float64 typed array (SpiderMonkey compatibility)
pub type Float64 = Float64Array;

/// Type alias for Int8 typed array (SpiderMonkey compatibility)
pub type Int8 = Int8Array;

/// Type alias for Int16 typed array (SpiderMonkey compatibility)
pub type Int16 = Int16Array;

/// Type alias for Int32 typed array (SpiderMonkey compatibility)
pub type Int32 = Int32Array;

/// Type alias for Uint16 typed array (SpiderMonkey compatibility)
pub type Uint16 = Uint16Array;

/// Type alias for Uint32 typed array (SpiderMonkey compatibility)
pub type Uint32 = Uint32Array;

/// Type alias for BigInt64 typed array (SpiderMonkey compatibility)
pub type BigInt64 = BigInt64Array;

/// Type alias for BigUint64 typed array (SpiderMonkey compatibility)
pub type BigUint64 = BigUint64Array;

/// ArrayBufferViewU8 - trait for views that expose u8 data
pub trait ArrayBufferViewU8: ArrayBufferView {
    fn to_vec(&self) -> Vec<u8>;
}

impl ArrayBufferViewU8 for Uint8Array {
    fn to_vec(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl ArrayBufferViewU8 for Uint8ClampedArray {
    fn to_vec(&self) -> Vec<u8> {
        self.data.clone()
    }
}

/// TypedArrayElement - marker trait for typed array element types
pub trait TypedArrayElement: Copy + Default {
    /// Type of the typed array for this element
    type ArrayType;
}

impl TypedArrayElement for u8 {
    type ArrayType = Uint8Array;
}

impl TypedArrayElement for i8 {
    type ArrayType = Int8Array;
}

impl TypedArrayElement for u16 {
    type ArrayType = Uint16Array;
}

impl TypedArrayElement for i16 {
    type ArrayType = Int16Array;
}

impl TypedArrayElement for u32 {
    type ArrayType = Uint32Array;
}

impl TypedArrayElement for i32 {
    type ArrayType = Int32Array;
}

impl TypedArrayElement for f32 {
    type ArrayType = Float32Array;
}

impl TypedArrayElement for f64 {
    type ArrayType = Float64Array;
}

impl TypedArrayElement for i64 {
    type ArrayType = BigInt64Array;
}

impl TypedArrayElement for u64 {
    type ArrayType = BigUint64Array;
}

/// TypedArrayElementCreator - trait for creating typed arrays from elements
pub trait TypedArrayElementCreator: TypedArrayElement {
    /// Create a typed array of this element type
    fn create_array(cx: *mut RawJSContext, len: usize, res: MutableHandleObject<'_>) -> bool;
}

impl<T: TypedArrayElement> TypedArrayElementCreator for T {
    fn create_array(_cx: *mut RawJSContext, _len: usize, _res: MutableHandleObject<'_>) -> bool {
        // Stub - actual creation handled by Boa
        true
    }
}