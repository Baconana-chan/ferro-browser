// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey gc module compatibility layer for Boa

use std::ptr;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::jsapi::{JSTracer, Value, JSObject, JSString, StringId};
use super::glue::{PropertyDescriptor, jsid, PropertyKey};

/// GCMethods - trait for types that need GC management
pub trait GCMethods: Sized {
    unsafe fn initial() -> Self;
    unsafe fn post_barrier(_v: *mut Self, _prev: Self, _next: Self) {}
    unsafe fn write_barriers(_v: *mut Self, _next: Self) {}
}

/// SpiderMonkey compatibility marker for types that can live in rooted slots.
pub trait Rootable {}

/// SpiderMonkey compatibility trait for types that provide an initial rooted value.
pub trait Initialize: Sized {
    unsafe fn initial() -> Option<Self>;
}

impl GCMethods for Value {
    unsafe fn initial() -> Self {
        Value::undefined()
    }
}

impl GCMethods for *mut JSObject {
    unsafe fn initial() -> Self {
        ptr::null_mut()
    }
}

impl GCMethods for *mut JSString {
    unsafe fn initial() -> Self {
        ptr::null_mut()
    }
}

impl GCMethods for jsid {
    unsafe fn initial() -> Self {
        jsid::VOID
    }
}

// Note: PropertyKey is a type alias for jsid, so no separate impl needed

impl GCMethods for PropertyDescriptor {
    unsafe fn initial() -> Self {
        PropertyDescriptor::default()
    }
}

impl GCMethods for StringId {
    unsafe fn initial() -> Self {
        StringId(ptr::null_mut())
    }
}

/// Traceable - trait for types that can be traced by GC
/// # Safety
/// Implementations must correctly trace all GC-managed values
pub unsafe trait Traceable {
    unsafe fn trace(&self, _tracer: *mut JSTracer);
}

unsafe impl Traceable for Value {
    unsafe fn trace(&self, _tracer: *mut JSTracer) {}
}

unsafe impl Traceable for *mut JSObject {
    unsafe fn trace(&self, _tracer: *mut JSTracer) {}
}

unsafe impl Traceable for *mut JSString {
    unsafe fn trace(&self, _tracer: *mut JSTracer) {}
}

unsafe impl Traceable for StringId {
    unsafe fn trace(&self, _tracer: *mut JSTracer) {}
}

unsafe impl Traceable for jsid {
    unsafe fn trace(&self, _tracer: *mut JSTracer) {}
}

unsafe impl Traceable for PropertyDescriptor {
    unsafe fn trace(&self, _tracer: *mut JSTracer) {}
}

/// RootedGuard - RAII guard for a rooted value
pub struct RootedGuard<'a, T: GCMethods> {
    value: T,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T: GCMethods> RootedGuard<'a, T> {
    pub fn new(value: T) -> Self {
        Self { value, _marker: PhantomData }
    }
    
    pub fn get(&self) -> &T {
        &self.value
    }
    
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
    
    pub fn handle(&self) -> super::rust::Handle<'_, T> {
        unsafe { super::rust::Handle::from_raw(&self.value) }
    }
    
    pub fn handle_mut(&mut self) -> super::rust::MutableHandle<'_, T> {
        unsafe { super::rust::MutableHandle::from_raw(&mut self.value) }
    }
}

impl<'a, T: GCMethods> Deref for RootedGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T: GCMethods> DerefMut for RootedGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// CustomAutoRooter - for custom rooted types
pub struct CustomAutoRooter<T> {
    value: T,
}

impl<T> CustomAutoRooter<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

/// CustomAutoRooterGuard - guard for custom rooted types
pub struct CustomAutoRooterGuard<'a, T> {
    rooter: &'a mut CustomAutoRooter<T>,
}

impl<'a, T> CustomAutoRooterGuard<'a, T> {
    pub fn new(_cx: *mut super::jsapi::RawJSContext, rooter: &'a mut CustomAutoRooter<T>) -> Self {
        Self { rooter }
    }
}

impl<'a, T> Deref for CustomAutoRooterGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.rooter.value
    }
}

impl<'a, T> DerefMut for CustomAutoRooterGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rooter.value
    }
}

/// RootedVec - a vector that roots all its elements
pub struct RootedVec<T> {
    vec: Vec<T>,
}

impl<T: GCMethods> RootedVec<T> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }
    
    pub fn push(&mut self, value: T) {
        self.vec.push(value);
    }
    
    pub fn pop(&mut self) -> Option<T> {
        self.vec.pop()
    }
    
    pub fn len(&self) -> usize {
        self.vec.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.vec.clear();
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.vec.iter()
    }
    
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.vec.iter_mut()
    }
}

impl<T: GCMethods> Default for RootedVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for RootedVec<T> {
    type Target = Vec<T>;
    
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> DerefMut for RootedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

/// StackGCVector - a stack-based GC vector
pub type StackGCVector<T> = RootedVec<T>;

/// GC status types
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GCProgress {
    GcCycleBegin = 0,
    GcSliceBegin = 1,
    GcSliceEnd = 2,
    GcCycleEnd = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GCReason {
    API = 0,
    EagerAllocTrigger = 1,
    DestroyRuntime = 2,
    RootingApi = 3,
    LastContext = 4,
    TooMuchMalloc = 5,
    AllocTrigger = 6,
    DebugGC = 7,
    CompartmentRevived = 8,
    Reset = 9,
    OutOfNursery = 10,
    EvictNursery = 11,
    FullCellPtrBuffer = 12,
    SharedMemoryLimit = 13,
    Periodic = 14,
    Incremental = 15,
    DomWorker = 16,
    InterSliceGC = 17,
    UnusedChunks = 18,
    FullStoreBuffer = 19,
    PageHide = 20,
    NativeStack = 21,
    CCWFinalized = 22,
    NurseryFull = 23,
    DomUtils = 24,
    UserInactive = 25,
    XpConnect = 26,
    MutationCallback = 27,
    SmallPaintDelay = 28,
    FullGenGC = 29,
    AbortGC = 30,
    FullWholeCell = 31,
    NoSuchReason = 32,
}

/// Root - a rooted value
#[repr(C)]
pub struct Root<T> {
    value: T,
}

impl<T: GCMethods> Root<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Copy> Root<T> {
    /// Get a Handle to this rooted value
    pub fn handle(&self) -> super::rust::Handle<'_, T> {
        unsafe { super::rust::Handle::from_raw(&self.value) }
    }
    
    /// Get a MutableHandle to this rooted value
    pub fn handle_mut(&mut self) -> super::rust::MutableHandle<'_, T> {
        unsafe { super::rust::MutableHandle::from_raw(&mut self.value) }
    }
    
    /// Get the contained value
    pub fn get(&self) -> T {
        self.value
    }
    
    /// Set the contained value (accepts values that can be converted into T)
    pub fn set<V: Into<T>>(&mut self, val: V) {
        self.value = val.into();
    }
}

// Generic implementations that work for non-Copy types too
impl<T> Root<T> {
    /// Get a Handle to this rooted value (reference version for non-Copy types)
    pub fn handle_ref(&self) -> super::rust::Handle<'_, T> {
        unsafe { super::rust::Handle::from_raw(&self.value) }
    }
    
    /// Get a MutableHandle to this rooted value (reference version for non-Copy types)
    pub fn handle_mut_ref(&mut self) -> super::rust::MutableHandle<'_, T> {
        unsafe { super::rust::MutableHandle::from_raw(&mut self.value) }
    }
    
    /// Get a reference to the contained value
    pub fn get_ref(&self) -> &T {
        &self.value
    }
    
    /// Set the contained value
    pub fn set_val(&mut self, val: T) {
        self.value = val;
    }
}

impl<T> Deref for Root<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Root<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// Rooted - SpiderMonkey's Rooted type
pub type Rooted<T> = Root<T>;

/// rooted! macro replacement - mimics SpiderMonkey's rooted! macro syntax
#[macro_export]
macro_rules! rooted {
    // SpiderMonkey syntax: rooted!(in(cx) let name = val)
    (in($cx:expr) let $name:ident = $val:expr) => {
        let _ = &$cx;  // Suppress unused warning
        let mut $name = $crate::js_compat::gc::Root::new($val);
    };
    (in($cx:expr) let mut $name:ident = $val:expr) => {
        let _ = &$cx;
        let mut $name = $crate::js_compat::gc::Root::new($val);
    };
    (in($cx:expr) let $name:ident: $ty:ty = $val:expr) => {
        let _ = &$cx;
        let mut $name: $crate::js_compat::gc::Root<$ty> = $crate::js_compat::gc::Root::new($val);
    };
    (in($cx:expr) let mut $name:ident: $ty:ty = $val:expr) => {
        let _ = &$cx;
        let mut $name: $crate::js_compat::gc::Root<$ty> = $crate::js_compat::gc::Root::new($val);
    };
    (in($cx:expr) let $name:ident: $ty:ty) => {
        let _ = &$cx;
        let mut $name = $crate::js_compat::gc::Root::<$ty>::new(unsafe { $crate::js_compat::gc::GCMethods::initial() });
    };
    (in($cx:expr) let mut $name:ident: $ty:ty) => {
        let _ = &$cx;
        let mut $name = $crate::js_compat::gc::Root::<$ty>::new(unsafe { $crate::js_compat::gc::GCMethods::initial() });
    };
    (&in($cx:expr) let $name:ident = $val:expr) => {
        let _ = &$cx;
        let mut $name = $crate::js_compat::gc::Root::new($val);
    };
    (&in($cx:expr) let mut $name:ident = $val:expr) => {
        let _ = &$cx;
        let mut $name = $crate::js_compat::gc::Root::new($val);
    };
    (&in($cx:expr) let $name:ident: $ty:ty) => {
        let _ = &$cx;
        let mut $name = $crate::js_compat::gc::Root::<$ty>::new(unsafe { $crate::js_compat::gc::GCMethods::initial() });
    };
    (&in($cx:expr) let mut $name:ident: $ty:ty) => {
        let _ = &$cx;
        let mut $name = $crate::js_compat::gc::Root::<$ty>::new(unsafe { $crate::js_compat::gc::GCMethods::initial() });
    };
    // Legacy syntax: rooted!($cx, let name = val)
    ($cx:expr, let $name:ident = $val:expr) => {
        let mut $name = $crate::js_compat::gc::Root::new($val);
    };
    ($cx:expr, let mut $name:ident = $val:expr) => {
        let mut $name = $crate::js_compat::gc::Root::new($val);
    };
}

#[macro_export]
macro_rules! auto_root {
    (in($cx:expr) let $name:ident = $val:expr) => {
        let mut $name = $crate::js_compat::gc::CustomAutoRooter::new($val);
        let $name = $crate::js_compat::gc::CustomAutoRooterGuard::new($cx, &mut $name);
    };
    (in($cx:expr) let mut $name:ident = $val:expr) => {
        let mut $name = $crate::js_compat::gc::CustomAutoRooter::new($val);
        let mut $name = $crate::js_compat::gc::CustomAutoRooterGuard::new($cx, &mut $name);
    };
    (&in($cx:expr) let $name:ident = $val:expr) => {
        let mut $name = $crate::js_compat::gc::CustomAutoRooter::new($val);
        let $name = $crate::js_compat::gc::CustomAutoRooterGuard::new($cx, &mut $name);
    };
    (&in($cx:expr) let mut $name:ident = $val:expr) => {
        let mut $name = $crate::js_compat::gc::CustomAutoRooter::new($val);
        let mut $name = $crate::js_compat::gc::CustomAutoRooterGuard::new($cx, &mut $name);
    };
}

#[macro_export]
macro_rules! rooted_vec {
    (let $name:ident) => {
        let $name = ::std::vec::Vec::new();
    };
    (let mut $name:ident) => {
        let mut $name = ::std::vec::Vec::new();
    };
    (let $name:ident <- $iter:expr) => {
        let $name: ::std::vec::Vec<_> = ($iter).collect();
    };
    (let mut $name:ident <- $iter:expr) => {
        let mut $name: ::std::vec::Vec<_> = ($iter).collect();
    };
}

/// IsMarkedUnbarriered - check if object is marked
pub unsafe fn IsMarkedUnbarriered(_obj: *mut JSObject) -> bool {
    true
}

/// TraceEdge - trace a GC edge
pub unsafe fn TraceEdge(_tracer: *mut JSTracer, _ptr: *mut Value, _name: *const i8) {
}

/// RootedTraceableBox - for rooting traceable boxes
pub struct RootedTraceableBox<T: Traceable + 'static> {
    inner: Box<T>,
}

impl<T: Traceable + 'static> RootedTraceableBox<T> {
    pub fn new(value: T) -> Self {
        Self { inner: Box::new(value) }
    }
    
    pub fn from_box(boxed: Box<T>) -> Self {
        Self { inner: boxed }
    }
    
    /// Trace the contained value
    pub unsafe fn trace(&self, tracer: *mut JSTracer) {
        (*self.inner).trace(tracer);
    }
}

impl<T: Traceable + 'static> Deref for RootedTraceableBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<T: Traceable + 'static> DerefMut for RootedTraceableBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

// ===================
// Traceable implementations for standard library types
// ===================

// Primitive types
unsafe impl Traceable for () { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for bool { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for i8 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for i16 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for i32 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for i64 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for i128 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for isize { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for u8 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for u16 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for u32 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for u64 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for u128 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for usize { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for f32 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for f64 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for char { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for String { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for str { unsafe fn trace(&self, _: *mut JSTracer) {} }

// Atomics
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize};
use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize};

unsafe impl Traceable for AtomicBool { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicI8 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicI16 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicI32 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicI64 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicIsize { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicU8 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicU16 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicU32 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicU64 { unsafe fn trace(&self, _: *mut JSTracer) {} }
unsafe impl Traceable for AtomicUsize { unsafe fn trace(&self, _: *mut JSTracer) {} }

// Option
unsafe impl<T: Traceable> Traceable for Option<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        if let Some(ref v) = *self {
            v.trace(tracer);
        }
    }
}

// Result
unsafe impl<T: Traceable, E: Traceable> Traceable for Result<T, E> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        match *self {
            Ok(ref v) => v.trace(tracer),
            Err(ref e) => e.trace(tracer),
        }
    }
}

// Cell types
use std::cell::{Cell, RefCell, UnsafeCell};

unsafe impl<T: Traceable + Copy> Traceable for Cell<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        self.get().trace(tracer);
    }
}

unsafe impl<T: Traceable> Traceable for RefCell<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        (*self).borrow().trace(tracer);
    }
}

unsafe impl<T: Traceable> Traceable for UnsafeCell<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        (*self.get()).trace(tracer);
    }
}

// Box, Vec, slice
unsafe impl<T: Traceable + ?Sized> Traceable for Box<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        (**self).trace(tracer);
    }
}

unsafe impl<T: Traceable> Traceable for Vec<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for item in self.iter() {
            item.trace(tracer);
        }
    }
}

unsafe impl<T: Traceable> Traceable for [T] {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for item in self.iter() {
            item.trace(tracer);
        }
    }
}

// Arrays
unsafe impl<T: Traceable, const N: usize> Traceable for [T; N] {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for item in self.iter() {
            item.trace(tracer);
        }
    }
}

// VecDeque
use std::collections::VecDeque;

unsafe impl<T: Traceable> Traceable for VecDeque<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for item in self.iter() {
            item.trace(tracer);
        }
    }
}

// HashMap
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

unsafe impl<K: Traceable, V: Traceable, S> Traceable for HashMap<K, V, S> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for (k, v) in self.iter() {
            k.trace(tracer);
            v.trace(tracer);
        }
    }
}

// HashSet
use std::collections::HashSet;

unsafe impl<T: Traceable, S> Traceable for HashSet<T, S> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for item in self.iter() {
            item.trace(tracer);
        }
    }
}

// BTreeMap
use std::collections::BTreeMap;

unsafe impl<K: Traceable, V: Traceable> Traceable for BTreeMap<K, V> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for (k, v) in self.iter() {
            k.trace(tracer);
            v.trace(tracer);
        }
    }
}

// Rc and Arc
use std::rc::Rc;
use std::sync::Arc;

unsafe impl<T: Traceable + ?Sized> Traceable for Rc<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        (**self).trace(tracer);
    }
}

unsafe impl<T: Traceable + ?Sized> Traceable for Arc<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        (**self).trace(tracer);
    }
}

// Tuples
unsafe impl<A: Traceable, B: Traceable> Traceable for (A, B) {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        self.0.trace(tracer);
        self.1.trace(tracer);
    }
}

unsafe impl<A: Traceable, B: Traceable, C: Traceable> Traceable for (A, B, C) {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        self.0.trace(tracer);
        self.1.trace(tracer);
        self.2.trace(tracer);
    }
}

// PhantomData
unsafe impl<T: ?Sized> Traceable for PhantomData<T> {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {}
}

// References
unsafe impl<T: Traceable + ?Sized> Traceable for &T {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        (*self).trace(tracer);
    }
}

unsafe impl<T: Traceable + ?Sized> Traceable for &mut T {
    #[inline]
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        (**self).trace(tracer);
    }
}

// Re-export Handle types for convenience (some code imports them from gc)
pub use super::rust::{Handle, MutableHandle, HandleObject, HandleValue, MutableHandleValue, MutableHandleObject};
