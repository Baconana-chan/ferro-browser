// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// Mutable DOM cell types for Boa engine
// These types provide interior mutability for GC-managed DOM objects

use std::cell::UnsafeCell;

use boa_gc::{Finalize, Trace};

use crate::reflector::DomObject;
use crate::root::{Dom, DomRoot, assert_in_script};

/// Check if we're in the layout thread
fn is_layout_thread() -> bool {
    // TODO: integrate with style::thread_state when available
    true
}

/// Assert that we are in the layout thread
pub fn assert_in_layout() {
    debug_assert!(is_layout_thread());
}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`. Essentially a `Cell<Dom<T>>`, but safer.
///
/// This should only be used as a field in other DOM objects.
pub struct MutDom<T: DomObject> {
    val: UnsafeCell<Dom<T>>,
}

impl<T: DomObject> MutDom<T> {
    /// Create a new `MutDom`.
    pub fn new(initial: &T) -> Self {
        assert_in_script();
        MutDom {
            val: UnsafeCell::new(Dom::from_ref(initial)),
        }
    }

    /// Set this `MutDom` to the given value.
    pub fn set(&self, val: &T) {
        assert_in_script();
        unsafe {
            *self.val.get() = Dom::from_ref(val);
        }
    }

    /// Get the value out of this object.
    pub fn get(&self) -> DomRoot<T> {
        assert_in_script();
        DomRoot::from_ref(unsafe { &**self.val.get() })
    }
}

impl<T: DomObject> PartialEq for MutDom<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.val.get() == *other.val.get() }
    }
}

impl<T: DomObject + PartialEq> PartialEq<T> for MutDom<T> {
    fn eq(&self, other: &T) -> bool {
        unsafe { **self.val.get() == *other }
    }
}

// Boa GC integration
unsafe impl<T: DomObject + Trace> Trace for MutDom<T> {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        unsafe { (*self.val.get()).trace(tracer) };
    }
    
    unsafe fn trace_non_roots(&self) {
        unsafe { (*self.val.get()).trace_non_roots() };
    }
    
    fn run_finalizer(&self) {
        unsafe { (*self.val.get()).run_finalizer() };
    }
}

impl<T: DomObject + Finalize> Finalize for MutDom<T> {}

/// A holder that provides interior mutability for GC-managed values such as
/// `Dom<T>`, with nullability represented by an enclosing Option wrapper.
/// Essentially a `Cell<Option<Dom<T>>>`, but safer.
///
/// This should only be used as a field in other DOM objects.
pub struct MutNullableDom<T: DomObject> {
    ptr: UnsafeCell<Option<Dom<T>>>,
}

impl<T: DomObject> MutNullableDom<T> {
    /// Create a new `MutNullableDom`.
    pub fn new(initial: Option<&T>) -> Self {
        assert_in_script();
        MutNullableDom {
            ptr: UnsafeCell::new(initial.map(Dom::from_ref)),
        }
    }

    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    pub fn or_init<F>(&self, cb: F) -> DomRoot<T>
    where
        F: FnOnce() -> DomRoot<T>,
    {
        assert_in_script();
        match self.get() {
            Some(inner) => inner,
            None => {
                let inner = cb();
                self.set(Some(&inner));
                inner
            },
        }
    }

    /// Retrieve a copy of the inner optional `Dom<T>` as `LayoutDom<T>`.
    /// For use by layout, which can't use safe types like DomRoot.
    ///
    /// # Safety
    /// Must only be called from the layout thread.
    pub unsafe fn get_inner_as_layout(&self) -> Option<LayoutDom<'_, T>> {
        assert_in_layout();
        let ptr = self.ptr.get();
        unsafe {
            (*ptr).as_ref().map(|dom| LayoutDom {
                value: &**dom,
            })
        }
    }

    /// Get a rooted value out of this object.
    pub fn get(&self) -> Option<DomRoot<T>> {
        assert_in_script();
        unsafe { (*self.ptr.get()).as_ref().map(|dom| DomRoot::from_ref(&**dom)) }
    }

    /// Set this `MutNullableDom` to the given value.
    pub fn set(&self, val: Option<&T>) {
        assert_in_script();
        unsafe {
            *self.ptr.get() = val.map(Dom::from_ref);
        }
    }

    /// Gets the current value out of this object and sets it to `None`.
    pub fn take(&self) -> Option<DomRoot<T>> {
        let value = self.get();
        self.set(None);
        value
    }

    /// Runs the given callback on the object if it's not null.
    pub fn if_is_some<F, R>(&self, cb: F) -> Option<&R>
    where
        F: FnOnce(&T) -> &R,
    {
        unsafe {
            if let Some(ref value) = *self.ptr.get() {
                Some(cb(value))
            } else {
                None
            }
        }
    }
}

impl<T: DomObject> PartialEq for MutNullableDom<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.ptr.get() == *other.ptr.get() }
    }
}

impl<T: DomObject> PartialEq<Option<&T>> for MutNullableDom<T> {
    fn eq(&self, other: &Option<&T>) -> bool {
        unsafe { *self.ptr.get() == other.map(Dom::from_ref) }
    }
}

impl<T: DomObject> Default for MutNullableDom<T> {
    fn default() -> Self {
        assert_in_script();
        MutNullableDom {
            ptr: UnsafeCell::new(None),
        }
    }
}

// Boa GC integration
unsafe impl<T: DomObject + Trace> Trace for MutNullableDom<T> {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        if let Some(dom) = unsafe { &*self.ptr.get() } {
            unsafe { dom.trace(tracer) };
        }
    }
    
    unsafe fn trace_non_roots(&self) {
        if let Some(dom) = unsafe { &*self.ptr.get() } {
            unsafe { dom.trace_non_roots() };
        }
    }
    
    fn run_finalizer(&self) {
        if let Some(dom) = unsafe { &*self.ptr.get() } {
            dom.run_finalizer();
        }
    }
}

impl<T: DomObject + Finalize> Finalize for MutNullableDom<T> {}

/// A holder that allows to lazily initialize the value only once.
/// Essentially a `OnceCell<Dom<T>>`.
///
/// This should only be used as a field in other DOM objects.
pub struct DomOnceCell<T: DomObject> {
    ptr: UnsafeCell<Option<Dom<T>>>,
}

impl<T: DomObject> DomOnceCell<T> {
    /// Create a new uninitialized `DomOnceCell`.
    pub const fn empty() -> Self {
        DomOnceCell {
            ptr: UnsafeCell::new(None),
        }
    }

    /// Initialize this cell with the given value.
    /// Panics if already initialized.
    pub fn init(&self, val: &T) {
        assert_in_script();
        unsafe {
            assert!((*self.ptr.get()).is_none(), "DomOnceCell already initialized");
            *self.ptr.get() = Some(Dom::from_ref(val));
        }
    }

    /// Get the value if initialized.
    pub fn get(&self) -> Option<DomRoot<T>> {
        assert_in_script();
        unsafe { (*self.ptr.get()).as_ref().map(|dom| DomRoot::from_ref(&**dom)) }
    }

    /// Check if this cell has been initialized.
    pub fn is_initialized(&self) -> bool {
        unsafe { (*self.ptr.get()).is_some() }
    }
}

impl<T: DomObject> Default for DomOnceCell<T> {
    fn default() -> Self {
        Self::empty()
    }
}

// Boa GC integration
unsafe impl<T: DomObject + Trace> Trace for DomOnceCell<T> {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        if let Some(dom) = unsafe { &*self.ptr.get() } {
            unsafe { dom.trace(tracer) };
        }
    }
    
    unsafe fn trace_non_roots(&self) {
        if let Some(dom) = unsafe { &*self.ptr.get() } {
            unsafe { dom.trace_non_roots() };
        }
    }
    
    fn run_finalizer(&self) {
        if let Some(dom) = unsafe { &*self.ptr.get() } {
            dom.run_finalizer();
        }
    }
}

impl<T: DomObject + Finalize> Finalize for DomOnceCell<T> {}

/// A reference to a DOM object that is safe to access from the layout thread.
///
/// This type does not prevent garbage collection, so it should only be used
/// during layout when we know the script thread is blocked.
pub struct LayoutDom<'a, T: DomObject> {
    value: &'a T,
}

impl<'a, T: DomObject> LayoutDom<'a, T> {
    /// Create a new LayoutDom from a reference.
    ///
    /// # Safety
    /// Must only be called from the layout thread when script is blocked.
    pub unsafe fn from_ref(value: &'a T) -> Self {
        assert_in_layout();
        LayoutDom { value }
    }

    /// Get a reference to the underlying DOM object.
    pub fn get(&self) -> &'a T {
        self.value
    }
}

impl<'a, T: DomObject> std::ops::Deref for LayoutDom<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: DomObject> Clone for LayoutDom<'a, T> {
    fn clone(&self) -> Self {
        LayoutDom { value: self.value }
    }
}

impl<'a, T: DomObject> Copy for LayoutDom<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reflector::Reflector;

    #[derive(Debug, PartialEq)]
    struct TestDomNode {
        reflector: Reflector,
        value: i32,
    }

    unsafe impl Trace for TestDomNode {
        unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
            unsafe { self.reflector.trace(tracer) };
        }
        unsafe fn trace_non_roots(&self) {}
        fn run_finalizer(&self) {}
    }

    impl Finalize for TestDomNode {}

    impl DomObject for TestDomNode {
        fn reflector(&self) -> &Reflector {
            &self.reflector
        }
    }

    #[test]
    fn test_mut_nullable_dom_default() {
        let cell: MutNullableDom<TestDomNode> = Default::default();
        assert!(cell.get().is_none());
    }

    #[test]
    fn test_dom_once_cell() {
        let cell: DomOnceCell<TestDomNode> = DomOnceCell::empty();
        assert!(!cell.is_initialized());
    }
}
