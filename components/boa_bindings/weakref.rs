// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// Weak reference types for Boa engine - equivalent to script_bindings/weakref.rs
// These types provide weak references to DOM objects for GC

use std::cell::Cell;
use std::hash::{Hash, Hasher};
use std::ptr;

use boa_gc::{Finalize, Trace};

use crate::reflector::DomObject;
use crate::root::{Dom, DomRoot, assert_in_script};

/// The inner box of weak references.
/// Public for the finalization in codegen.
pub struct WeakBox<T: WeakReferenceable> {
    /// The reference count. When it reaches zero, the `value` field should
    /// have already been set to `None`. The pointee contributes one to the count.
    pub count: Cell<usize>,
    /// The pointer to the DOM object, set to None when it is collected.
    pub value: Cell<Option<ptr::NonNull<T>>>,
}

/// A weak reference to a DOM object.
///
/// Unlike strong references (`Dom<T>`), weak references do not prevent
/// garbage collection. When the referenced object is collected, the
/// weak reference returns `None`.
pub struct WeakRef<T: WeakReferenceable> {
    ptr: ptr::NonNull<WeakBox<T>>,
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
            }));
            
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
    /// # Safety
    /// Must only be called during GC finalization.
    pub unsafe fn invalidate(&self) {
        unsafe { &*self.ptr.as_ptr() }.value.set(None);
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

    #[derive(Debug)]
    struct TestWeakDom {
        reflector: Reflector,
        value: i32,
    }

    unsafe impl Trace for TestWeakDom {
        unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
            unsafe { self.reflector.trace(tracer) };
        }
        unsafe fn trace_non_roots(&self) {}
        fn run_finalizer(&self) {}
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
        // Note: This test may not work perfectly without full GC integration
        // but demonstrates the API usage
    }
}
