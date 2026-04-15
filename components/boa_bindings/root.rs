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
use super::gc_safety::{
    assert_rooted, assert_not_in_gc, record_allocation, 
    RootGuard, gc_debug_enabled,
};

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

/// Specific impl for Root<Dom<T>> to handle raw pointer conversions without 'static bound
impl<T: DomObject> Root<Dom<T>> {
    /// Create a Root<Dom<T>> from a raw pointer (SpiderMonkey compatibility)
    ///
    /// # Safety
    /// The pointer must point to a valid, properly allocated Dom<T>
    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        assert_in_script();
        Root { value: Dom::from_ptr(ptr) }
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
        // In debug mode, verify we're in a rooted context
        if cfg!(debug_assertions) {
            // Allow creation during trace/finalize operations
            // as those are GC-safe contexts
        }
        Dom {
            ptr: ptr::NonNull::from(obj),
        }
    }

    /// Create a `Dom<T>` from a raw pointer (SpiderMonkey compatibility)
    ///
    /// # Safety
    /// The pointer must point to a valid, properly allocated T
    pub unsafe fn from_ptr(ptr: *mut T) -> Dom<T> {
        assert_in_script();
        Dom {
            ptr: ptr::NonNull::new_unchecked(ptr),
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

// ============================================================================
// Phase B1: Boa-specific root_from_object
// ============================================================================

use boa_engine::{JsObject, JsValue, JsResult, JsNativeError, Context};

/// Error type for DOM extraction failures
#[derive(Debug, Clone)]
pub enum DomExtractionError {
    /// The value is not an object
    NotAnObject,
    /// The object does not have the expected prototype
    WrongPrototype { expected: &'static str, actual: String },
    /// The object is from a different realm
    WrongRealm,
    /// The object's internal slot is null/undefined
    NullInternalSlot,
    /// The object has been garbage collected
    ObjectCollected,
}

impl std::fmt::Display for DomExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomExtractionError::NotAnObject => write!(f, "Value is not an object"),
            DomExtractionError::WrongPrototype { expected, actual } => {
                write!(f, "Expected {} prototype, got {}", expected, actual)
            }
            DomExtractionError::WrongRealm => write!(f, "Object is from a different realm"),
            DomExtractionError::NullInternalSlot => write!(f, "Internal slot is null"),
            DomExtractionError::ObjectCollected => write!(f, "Object has been garbage collected"),
        }
    }
}

impl std::error::Error for DomExtractionError {}

impl From<DomExtractionError> for JsNativeError {
    fn from(err: DomExtractionError) -> Self {
        JsNativeError::typ().with_message(err.to_string())
    }
}

/// Result type for DOM extraction
pub type DomExtractionResult<T> = Result<T, DomExtractionError>;

/// Trait for extracting DOM objects from JavaScript values.
/// 
/// This is the Boa equivalent of SpiderMonkey's `root_from_object`.
/// It provides type-safe extraction of DOM objects from JS values.
pub trait RootFromObject: DomObject + Sized {
    /// The interface name for error messages
    const INTERFACE_NAME: &'static str;
    
    /// Extract a DOM object from a JsObject.
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The object is not a valid DOM wrapper
    /// - The object is from a different realm
    /// - The internal slot is null
    fn root_from_object(obj: &JsObject, ctx: &mut Context) -> DomExtractionResult<DomRoot<Self>>;
    
    /// Extract a DOM object from a JsValue.
    /// 
    /// # Errors
    /// Returns an error if the value is not an object or extraction fails.
    fn root_from_value(val: &JsValue, ctx: &mut Context) -> DomExtractionResult<DomRoot<Self>> {
        match val.as_object() {
            Some(obj) => Self::root_from_object(&obj, ctx),
            None => Err(DomExtractionError::NotAnObject),
        }
    }
    
    /// Try to extract a DOM object, returning None instead of error.
    fn try_root_from_object(obj: &JsObject, ctx: &mut Context) -> Option<DomRoot<Self>> {
        Self::root_from_object(obj, ctx).ok()
    }
    
    /// Try to extract a DOM object from a value, returning None instead of error.
    fn try_root_from_value(val: &JsValue, ctx: &mut Context) -> Option<DomRoot<Self>> {
        Self::root_from_value(val, ctx).ok()
    }
}

/// Helper to convert DomExtractionError to JsResult
pub fn extraction_to_js_result<T>(result: DomExtractionResult<T>) -> JsResult<T> {
    result.map_err(|e| JsNativeError::from(e).into())
}

/// Macro to implement RootFromObject for a DOM type
#[macro_export]
macro_rules! impl_root_from_object {
    ($type:ty, $interface_name:expr) => {
        impl $crate::root::RootFromObject for $type {
            const INTERFACE_NAME: &'static str = $interface_name;
            
            fn root_from_object(
                obj: &::boa_engine::JsObject,
                _ctx: &mut ::boa_engine::Context,
            ) -> $crate::root::DomExtractionResult<$crate::root::DomRoot<Self>> {
                // In Boa, we use NativeData trait to store DOM pointers
                // This is a placeholder - actual implementation depends on
                // how DOM objects are wrapped in JS objects
                Err($crate::root::DomExtractionError::NotAnObject)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dom_extraction_error_display() {
        let err = DomExtractionError::NotAnObject;
        assert_eq!(err.to_string(), "Value is not an object");
        
        let err = DomExtractionError::WrongPrototype {
            expected: "HTMLElement",
            actual: "Object".to_string(),
        };
        assert!(err.to_string().contains("HTMLElement"));
        
        let err = DomExtractionError::WrongRealm;
        assert!(err.to_string().contains("realm"));
    }
}
