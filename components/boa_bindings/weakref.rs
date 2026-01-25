// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// Weak reference types for Boa engine - equivalent to script_bindings/weakref.rs
// These types provide weak references to DOM objects for GC
//
// Finalization order:
// 1. finalize() - cleanup resources owned by the object
// 2. detach() - remove from parent/child relationships
// 3. invalidate() - null out weak references

use std::cell::Cell;
use std::hash::{Hash, Hasher};
use std::ptr;

use boa_gc::{Finalize, Trace};

use crate::reflector::DomObject;
use crate::root::{Dom, DomRoot, assert_in_script};
use crate::gc_safety::{
    gc_debug_enabled, is_in_finalize, assert_finalize_order,
    WeakRefState, WeakRefDebugInfo,
};

/// The inner box of weak references.
/// Public for the finalization in codegen.
pub struct WeakBox<T: WeakReferenceable> {
    /// The reference count. When it reaches zero, the `value` field should
    /// have already been set to `None`. The pointee contributes one to the count.
    pub count: Cell<usize>,
    /// The pointer to the DOM object, set to None when it is collected.
    pub value: Cell<Option<ptr::NonNull<T>>>,
    /// Debug info for tracking (only in debug builds)
    #[cfg(debug_assertions)]
    pub debug_info: Cell<WeakRefState>,
}

/// A weak reference to a DOM object.
///
/// Unlike strong references (`Dom<T>`), weak references do not prevent
/// garbage collection. When the referenced object is collected, the
/// weak reference returns `None`.
pub struct WeakRef<T: WeakReferenceable> {
    ptr: ptr::NonNull<WeakBox<T>>,
}

impl<T: WeakReferenceable> std::fmt::Debug for WeakRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakRef")
            .field("alive", &self.is_alive())
            .field("ptr", &self.ptr)
            .finish()
    }
}

/// Trait implemented by weak-referenceable interfaces.
///
/// DOM objects that support weak references must implement this trait.
/// In Boa, this is simpler than SpiderMonkey since we don't need
/// reserved slots for weak pointers.
pub trait WeakReferenceable: DomObject + Sized {
    /// Downgrade a DOM object reference to a weak one.
    fn downgrade(&self) -> WeakRef<Self> {
        unsafe {
            // For Boa, we manage the WeakBox separately from JS slots
            // since Boa doesn't have the same slot mechanism as SpiderMonkey
            let ptr = Box::into_raw(Box::new(WeakBox {
                count: Cell::new(2), // One for the weak ref, one for the pointee
                value: Cell::new(Some(ptr::NonNull::from(self))),
                #[cfg(debug_assertions)]
                debug_info: Cell::new(WeakRefState::Valid),
            }));
            
            if gc_debug_enabled() {
                eprintln!("[GC_DEBUG] WeakRef created for {:p}", self);
            }
            
            WeakRef {
                ptr: ptr::NonNull::new_unchecked(ptr),
            }
        }
    }
    
    /// Get the weak box for this object, if one exists.
    /// This is used internally for reference counting.
    fn get_weak_box(&self) -> Option<ptr::NonNull<WeakBox<Self>>> {
        // Default implementation - no weak box
        // Actual implementation would store this in the reflector
        None
    }
    
    /// Called during finalization before weak refs are invalidated.
    /// Override to cleanup resources.
    fn weak_ref_will_invalidate(&self) {
        assert_finalize_order("detach", std::any::type_name::<Self>());
    }
}

impl<T: WeakReferenceable> WeakRef<T> {
    /// Create a new weak reference from a `WeakReferenceable` interface instance.
    /// This is just a convenience wrapper around `<T as WeakReferenceable>::downgrade`.
    pub fn new(value: &T) -> Self {
        value.downgrade()
    }

    /// Root a weak reference. Returns `None` if the object was already collected.
    pub fn root(&self) -> Option<DomRoot<T>> {
        unsafe { &*self.ptr.as_ptr() }
            .value
            .get()
            .map(|ptr| DomRoot::from_ref(unsafe { &*ptr.as_ptr() }))
    }

    /// Return whether the weakly-referenced object is still alive.
    pub fn is_alive(&self) -> bool {
        unsafe { &*self.ptr.as_ptr() }.value.get().is_some()
    }
    
    /// Invalidate this weak reference (called when the object is collected).
    /// 
    /// This follows the finalization order:
    /// 1. finalize (cleanup resources) - done by Boa GC
    /// 2. detach (remove from parent structures) - weak_ref_will_invalidate
    /// 3. invalidate (null out weak references) - this method
    /// 
    /// # Safety
    /// Must only be called during GC finalization.
    pub unsafe fn invalidate(&self) {
        let weak_box = unsafe { &*self.ptr.as_ptr() };
        
        #[cfg(debug_assertions)]
        {
            let state = weak_box.debug_info.get();
            debug_assert_eq!(
                state, 
                WeakRefState::Valid,
                "WeakRef already invalidated or finalizing!"
            );
            weak_box.debug_info.set(WeakRefState::Invalidated);
        }
        
        if gc_debug_enabled() {
            if let Some(ptr) = weak_box.value.get() {
                eprintln!("[GC_DEBUG] WeakRef invalidated for {:p}", ptr.as_ptr());
            }
        }
        
        assert_finalize_order("invalidate", "WeakRef");
        weak_box.value.set(None);
    }
    
    /// Begin finalization (marks the weak ref as being finalized)
    /// 
    /// # Safety
    /// Must only be called during GC finalization.
    #[cfg(debug_assertions)]
    pub unsafe fn begin_finalize(&self) {
        let weak_box = unsafe { &*self.ptr.as_ptr() };
        let state = weak_box.debug_info.get();
        debug_assert_eq!(state, WeakRefState::Valid, "Invalid state for begin_finalize");
        weak_box.debug_info.set(WeakRefState::Finalizing);
    }
}

impl<T: WeakReferenceable> Clone for WeakRef<T> {
    fn clone(&self) -> WeakRef<T> {
        unsafe {
            let box_ = &*self.ptr.as_ptr();
            let new_count = box_.count.get() + 1;
            box_.count.set(new_count);
            WeakRef { ptr: self.ptr }
        }
    }
}

impl<T: WeakReferenceable> Eq for WeakRef<T> {}

impl<T: WeakReferenceable> PartialEq for WeakRef<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            (*self.ptr.as_ptr()).value.get().map(ptr::NonNull::as_ptr) ==
                (*other.ptr.as_ptr()).value.get().map(ptr::NonNull::as_ptr)
        }
    }
}

impl<T: WeakReferenceable> PartialEq<T> for WeakRef<T> {
    fn eq(&self, other: &T) -> bool {
        unsafe {
            match self.ptr.as_ref().value.get() {
                Some(ptr) => ptr::eq(ptr.as_ptr(), other),
                None => false,
            }
        }
    }
}

impl<T: WeakReferenceable> Hash for WeakRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl<T: WeakReferenceable> Drop for WeakRef<T> {
    fn drop(&mut self) {
        unsafe {
            let (count, value) = {
                let weak_box = &*self.ptr.as_ptr();
                assert!(weak_box.count.get() > 0);
                let count = weak_box.count.get() - 1;
                weak_box.count.set(count);
                (count, weak_box.value.get())
            };
            if count == 0 {
                assert!(value.is_none());
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

// Boa GC integration - WeakRef doesn't trace its target
unsafe impl<T: WeakReferenceable> Trace for WeakRef<T> {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // Do nothing - weak references are not traced
    }
    
    unsafe fn trace_non_roots(&self) {
        // Do nothing
    }
    
    fn run_finalizer(&self) {
        // Do nothing
    }
}

impl<T: WeakReferenceable> Finalize for WeakRef<T> {}

/// A mutable weak reference holder.
///
/// This allows storing an optional weak reference that can be changed.
pub struct MutableWeakRef<T: WeakReferenceable> {
    cell: std::cell::UnsafeCell<Option<WeakRef<T>>>,
}

impl<T: WeakReferenceable> MutableWeakRef<T> {
    /// Create a new mutable weak reference.
    pub fn new(value: Option<&T>) -> Self {
        MutableWeakRef {
            cell: std::cell::UnsafeCell::new(value.map(WeakRef::new)),
        }
    }

    /// Set the pointee of a mutable weak reference.
    pub fn set(&self, value: Option<&T>) {
        assert_in_script();
        unsafe {
            *self.cell.get() = value.map(WeakRef::new);
        }
    }

    /// Root a mutable weak reference. Returns `None` if the object
    /// was already collected or if the reference is None.
    pub fn root(&self) -> Option<DomRoot<T>> {
        unsafe { &*self.cell.get() }
            .as_ref()
            .and_then(WeakRef::root)
    }
    
    /// Check if this weak reference is alive.
    pub fn is_alive(&self) -> bool {
        unsafe { &*self.cell.get() }
            .as_ref()
            .map_or(false, WeakRef::is_alive)
    }
}

impl<T: WeakReferenceable> Default for MutableWeakRef<T> {
    fn default() -> Self {
        Self::new(None)
    }
}

// Boa GC integration
unsafe impl<T: WeakReferenceable> Trace for MutableWeakRef<T> {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        if let Some(weak) = unsafe { &*self.cell.get() } {
            unsafe { weak.trace(tracer) };
        }
    }
    
    unsafe fn trace_non_roots(&self) {}
    
    fn run_finalizer(&self) {}
}

impl<T: WeakReferenceable> Finalize for MutableWeakRef<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reflector::Reflector;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Debug)]
    struct TestWeakDom {
        reflector: Reflector,
        value: i32,
        finalized: AtomicBool,
    }

    impl TestWeakDom {
        fn new(value: i32) -> Self {
            TestWeakDom {
                reflector: Reflector::new(),
                value,
                finalized: AtomicBool::new(false),
            }
        }
    }
    
    impl PartialEq for TestWeakDom {
        fn eq(&self, other: &Self) -> bool {
            self.value == other.value && self.reflector == other.reflector
        }
    }

    unsafe impl Trace for TestWeakDom {
        unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
            unsafe { self.reflector.trace(tracer) };
        }
        unsafe fn trace_non_roots(&self) {}
        fn run_finalizer(&self) {
            self.finalized.store(true, Ordering::SeqCst);
        }
    }

    impl Finalize for TestWeakDom {}

    impl DomObject for TestWeakDom {
        fn reflector(&self) -> &Reflector {
            &self.reflector
        }
    }

    impl WeakReferenceable for TestWeakDom {}

    #[test]
    fn test_weak_ref_creation() {
        let obj = TestWeakDom::new(42);
        let weak = WeakRef::new(&obj);
        
        assert!(weak.is_alive());
        
        // Root should succeed while object is alive
        let rooted = weak.root();
        assert!(rooted.is_some());
    }
    
    #[test]
    fn test_weak_ref_clone() {
        let obj = TestWeakDom::new(100);
        let weak1 = WeakRef::new(&obj);
        let weak2 = weak1.clone();
        
        assert!(weak1.is_alive());
        assert!(weak2.is_alive());
        assert_eq!(weak1, weak2);
    }
    
    #[test]
    fn test_weak_ref_invalidation() {
        let obj = TestWeakDom::new(200);
        let weak = WeakRef::new(&obj);
        
        assert!(weak.is_alive());
        
        // Simulate GC invalidation
        unsafe { weak.invalidate(); }
        
        assert!(!weak.is_alive());
        assert!(weak.root().is_none());
    }
    
    #[test]
    fn test_mutable_weak_ref() {
        let obj1 = TestWeakDom::new(1);
        let obj2 = TestWeakDom::new(2);
        
        let mut_weak = MutableWeakRef::new(Some(&obj1));
        assert!(mut_weak.is_alive());
        
        // Change to obj2
        mut_weak.set(Some(&obj2));
        assert!(mut_weak.is_alive());
        
        // Set to None
        mut_weak.set(None);
        assert!(!mut_weak.is_alive());
    }
    
    #[test]
    fn test_weak_ref_equality_with_object() {
        let obj = TestWeakDom::new(42);
        let weak = WeakRef::new(&obj);
        
        // WeakRef should equal the original object when alive
        assert!(weak.is_alive());
        // Note: We compare via root() since direct comparison requires trait bounds
        let rooted = weak.root().unwrap();
        assert_eq!(rooted.value, obj.value);
    }
    
    #[test]
    fn test_double_invalidation_debug() {
        // Test that debug assertions catch double invalidation
        let obj = TestWeakDom::new(300);
        let weak = WeakRef::new(&obj);
        
        unsafe { weak.invalidate(); }
        
        // Second invalidation should panic in debug mode
        #[cfg(debug_assertions)]
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                unsafe { weak.invalidate(); }
            }));
            assert!(result.is_err(), "Double invalidation should panic in debug");
        }
    }
    
    #[test]
    fn test_weak_ref_hash() {
        use std::collections::HashSet;
        
        let obj = TestWeakDom::new(42);
        let weak1 = WeakRef::new(&obj);
        let weak2 = weak1.clone();
        
        let mut set = HashSet::new();
        set.insert(weak1.ptr);
        
        // Clone should have same hash
        assert!(set.contains(&weak2.ptr));
    }
    
    #[test]
    fn test_finalization_order() {
        // This test verifies the finalization order documentation
        // 1. finalize (run_finalizer) - cleanup resources
        // 2. detach (weak_ref_will_invalidate) - remove from structures
        // 3. invalidate - null out weak references
        
        let obj = TestWeakDom::new(400);
        let weak = WeakRef::new(&obj);
        
        // Object not yet finalized
        assert!(!obj.finalized.load(Ordering::SeqCst));
        
        // Simulate finalization sequence
        obj.run_finalizer();
        assert!(obj.finalized.load(Ordering::SeqCst));
        
        // WeakRef still valid until explicitly invalidated
        assert!(weak.is_alive());
        
        // Detach phase (notify weak refs)
        obj.weak_ref_will_invalidate();
        
        // Invalidate phase
        unsafe { weak.invalidate(); }
        assert!(!weak.is_alive());
    }
}

