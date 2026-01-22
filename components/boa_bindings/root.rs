// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// DOM root types for Boa engine - equivalent to script_bindings/root.rs
// These types provide GC-safe smart pointers for DOM objects

use std::cell::UnsafeCell;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::{fmt, mem, ptr};

use boa_gc::{Finalize, Trace, Gc, GcRefCell};

use super::reflector::{Reflector, DomObject, MutDomObject};
use super::trace::BoaTraceable;

/// Check if we are in the script thread (stub for now)
fn is_script_thread() -> bool {
    // TODO: integrate with style::thread_state when available
    true
}

/// Assert that we are in the script thread
pub fn assert_in_script() {
    debug_assert!(is_script_thread());
}

/// A rooted value that will be traced by Boa's GC
/// 
/// Unlike SpiderMonkey's Root which uses a RootCollection,
/// Boa uses Gc<T> for garbage collection. This type wraps
/// the Boa GC mechanism to provide a similar API.
#[derive(Clone)]
pub struct Root<T: StableTraceObject> {
    /// The value to root
    value: T,
}

impl<T: StableTraceObject + 'static> Root<T> {
    /// Create a new rooted value
    /// 
    /// # Safety
    /// The value must be properly traceable
    #[allow(unused_unsafe)]
    pub unsafe fn new(value: T) -> Self {
        assert_in_script();
        Root { value }
    }
}

/// Trait for objects that can be stably traced
/// 
/// # Safety
/// Implementors must ensure trace is implemented correctly
pub unsafe trait StableTraceObject: Sized {
    /// Returns a stable trace object
    fn stable_trace_object(&self) -> *const dyn BoaTraceable;
}

unsafe impl<T: DomObject> StableTraceObject for Dom<T> {
    fn stable_trace_object(&self) -> *const dyn BoaTraceable {
        self.ptr.as_ptr() as *const dyn BoaTraceable
    }
}

impl<T: Deref + StableTraceObject> Deref for Root<T> {
    type Target = <T as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        assert_in_script();
        &self.value
    }
}

impl<T: fmt::Debug + StableTraceObject> fmt::Debug for Root<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: StableTraceObject + PartialEq> PartialEq for Root<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: StableTraceObject + Eq> Eq for Root<T> {}

impl<T: StableTraceObject + Hash> Hash for Root<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

/// A traced reference to a DOM object
///
/// This type is critical to making garbage collection work with the DOM,
/// but it is very dangerous; if garbage collection happens with a `Dom<T>`
/// on the stack, the `Dom<T>` can point to freed memory.
///
/// This should only be used as a field in other DOM objects.
#[repr(transparent)]
pub struct Dom<T> {
    ptr: ptr::NonNull<T>,
}

impl<T> Dom<T> {
    /// Get the raw pointer
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }
}

impl<T: DomObject> Dom<T> {
    /// Create a `Dom<T>` from a `&T`
    pub fn from_ref(obj: &T) -> Dom<T> {
        assert_in_script();
        Dom {
            ptr: ptr::NonNull::from(obj),
        }
    }

    /// Return a rooted version of this DOM object
    pub fn as_rooted(&self) -> DomRoot<T> {
        DomRoot::from_ref(self)
    }
}

impl<T: DomObject> Deref for Dom<T> {
    type Target = T;

    fn deref(&self) -> &T {
        assert_in_script();
        unsafe { &*self.ptr.as_ptr() }
    }
}

impl<T> PartialEq for Dom<T> {
    fn eq(&self, other: &Dom<T>) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<'a, T: DomObject> PartialEq<&'a T> for Dom<T> {
    fn eq(&self, other: &&'a T) -> bool {
        *self == Dom::from_ref(*other)
    }
}

impl<T> Eq for Dom<T> {}

impl<T> Hash for Dom<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.as_ptr().hash(state)
    }
}

impl<T> Clone for Dom<T> {
    #[inline]
    fn clone(&self) -> Self {
        assert_in_script();
        Dom { ptr: self.ptr }
    }
}

impl<T: fmt::Debug + DomObject> fmt::Debug for Dom<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

// Boa GC integration
unsafe impl<T: DomObject + Trace> Trace for Dom<T> {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        // Trace the underlying DOM object
        unsafe {
            (*self.ptr.as_ptr()).trace(tracer);
        }
    }
    
    unsafe fn trace_non_roots(&self) {
        // Trace non-roots
        unsafe {
            (*self.ptr.as_ptr()).trace_non_roots();
        }
    }
    
    fn run_finalizer(&self) {
        // For potential future GC optimizations
    }
}

impl<T: DomObject + Finalize> Finalize for Dom<T> {}

/// A rooted reference to a DOM object
pub type DomRoot<T> = Root<Dom<T>>;

impl<T: DomObject> DomRoot<T> {
    /// Generate a new root from a reference
    pub fn from_ref(unrooted: &T) -> DomRoot<T> {
        unsafe { DomRoot::new(Dom::from_ref(unrooted)) }
    }

    /// Create a traced version of this rooted object
    pub fn as_traced(&self) -> Dom<T> {
        Dom::from_ref(self)
    }
}

/// A traced reference to a DOM object that may not be reflected yet
pub struct MaybeUnreflectedDom<T> {
    ptr: ptr::NonNull<T>,
}

impl<T: DomObject> MaybeUnreflectedDom<T> {
    /// Create a new MaybeUnreflectedDom from a boxed value
    /// 
    /// # Safety
    /// The boxed value must be properly initialized
    pub unsafe fn from_box(value: Box<T>) -> Self {
        Self {
            ptr: Box::leak(value).into(),
        }
    }
}

unsafe impl<T: DomObject> StableTraceObject for MaybeUnreflectedDom<T> {
    fn stable_trace_object(&self) -> *const dyn BoaTraceable {
        self.ptr.as_ptr() as *const dyn BoaTraceable
    }
}

impl<T: DomObject> Root<MaybeUnreflectedDom<T>> {
    /// Get the raw pointer
    pub fn as_ptr(&self) -> *const T {
        self.value.ptr.as_ptr()
    }
}

impl<T: MutDomObject> Root<MaybeUnreflectedDom<T>> {
    /// Reflect the unreflected object with the given JS object
    /// 
    /// # Safety
    /// obj must point to a valid JS object
    pub unsafe fn reflect_with(self, obj: *mut super::js_compat::jsapi::JSObject) -> DomRoot<T> {
        let ptr = self.as_ptr();
        drop(self);
        let root = DomRoot::from_ref(unsafe { &*ptr });
        unsafe { root.init_reflector(obj) };
        root
    }
}

/// A rooting mechanism for reflectors on the stack
/// Uses Boa's GC instead of SpiderMonkey's RootCollection
pub struct RootCollection {
    roots: UnsafeCell<Vec<*const dyn BoaTraceable>>,
}

impl RootCollection {
    /// Create an empty collection of roots
    pub const fn new() -> RootCollection {
        RootCollection {
            roots: UnsafeCell::new(vec![]),
        }
    }

    /// Start tracking a trace object
    /// 
    /// # Safety
    /// Object must be valid
    pub unsafe fn root(&self, object: *const dyn BoaTraceable) {
        assert_in_script();
        unsafe { (*self.roots.get()).push(object) };
    }

    /// Stop tracking a trace object
    /// 
    /// # Safety
    /// Object must have been previously rooted
    pub unsafe fn unroot(&self, object: *const dyn BoaTraceable) {
        assert_in_script();
        let roots = unsafe { &mut *self.roots.get() };
        match roots
            .iter()
            .rposition(|r| std::ptr::addr_eq(*r as *const (), object as *const ()))
        {
            Some(idx) => {
                roots.swap_remove(idx);
            },
            None => panic!("Can't remove a root that was never rooted!"),
        }
    }
}

impl Default for RootCollection {
    fn default() -> Self {
        Self::new()
    }
}

thread_local!(pub static STACK_ROOTS: RootCollection = const { RootCollection::new() });

/// Trace all roots (for GC integration)
/// 
/// # Safety
/// Must only be called from GC tracer
pub unsafe fn trace_roots(tracer: &mut boa_gc::Tracer) {
    STACK_ROOTS.with(|collection| {
        let collection = unsafe { &*collection.roots.get() };
        for root in collection {
            unsafe {
                (**root).trace_boa(tracer);
            }
        }
    });
}

/// Get a slice of references to DOM objects
pub trait DomSlice<T>
where
    T: DomObject,
{
    /// Returns the slice of T references
    fn r(&self) -> &[&T];
}

impl<T: DomObject> DomSlice<T> for [Dom<T>] {
    #[inline]
    fn r(&self) -> &[&T] {
        let _ = mem::transmute::<Dom<T>, &T>;
        unsafe { &*(self as *const [Dom<T>] as *const [&T]) }
    }
}
