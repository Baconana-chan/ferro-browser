// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey gc module compatibility layer for Boa

use std::ptr;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::jsapi::{JSTracer, Value, JSObject};

/// GCMethods - trait for types that need GC management
pub trait GCMethods: Sized {
    unsafe fn initial() -> Self;
    unsafe fn post_barrier(_v: *mut Self, _prev: Self, _next: Self) {}
    unsafe fn write_barriers(_v: *mut Self, _next: Self) {}
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

/// Traceable - trait for types that can be traced by GC
pub trait Traceable {
    unsafe fn trace(&self, _tracer: *mut JSTracer);
}

impl Traceable for Value {
    unsafe fn trace(&self, _tracer: *mut JSTracer) {}
}

impl Traceable for *mut JSObject {
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

/// rooted! macro replacement
#[macro_export]
macro_rules! rooted {
    ($cx:expr, let $name:ident = $val:expr) => {
        let mut $name = $crate::js_compat::gc::Root::new($val);
    };
    ($cx:expr, let mut $name:ident = $val:expr) => {
        let mut $name = $crate::js_compat::gc::Root::new($val);
    };
}

/// IsMarkedUnbarriered - check if object is marked
pub unsafe fn IsMarkedUnbarriered(_obj: *mut JSObject) -> bool {
    true
}

/// TraceEdge - trace a GC edge
pub unsafe fn TraceEdge(_tracer: *mut JSTracer, _ptr: *mut Value, _name: *const i8) {
}
// Re-export Handle types for convenience (some code imports them from gc)
pub use super::rust::{Handle, MutableHandle, HandleObject, HandleValue, MutableHandleValue, MutableHandleObject};