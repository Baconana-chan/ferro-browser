// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Garbage collection integration for Boa with DOM.
//!
//! Boa 0.21 uses boa_gc for automatic garbage collection.

pub use boa_gc::{Finalize, Gc, GcRefCell, Trace, force_collect};

/// Marker trait for DOM objects traceable by Boa's GC.
pub trait DomTraceable: Trace + Finalize {}

/// A GC-managed pointer to a DOM object.
#[derive(Clone)]
pub struct DomRef<T: Trace + 'static> {
    inner: Gc<T>,
}

impl<T: Trace + 'static> DomRef<T> {
    /// Create a new GC-managed reference.
    pub fn new(value: T) -> Self {
        Self { inner: Gc::new(value) }
    }

    /// Get a reference to the inner value.
    pub fn get(&self) -> &T {
        &self.inner
    }
}

impl<T: Trace + 'static> std::ops::Deref for DomRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A mutable GC-managed cell for DOM objects.
pub struct DomCell<T: Trace + 'static> {
    inner: GcRefCell<T>,
}

impl<T: Trace + 'static> DomCell<T> {
    /// Create a new mutable GC-managed cell.
    pub fn new(value: T) -> Self {
        Self { inner: GcRefCell::new(value) }
    }

    /// Borrow the inner value immutably.
    pub fn borrow(&self) -> boa_gc::GcRef<'_, T> {
        self.inner.borrow()
    }

    /// Borrow the inner value mutably.
    pub fn borrow_mut(&self) -> boa_gc::GcRefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

/// Run a garbage collection cycle.
pub fn collect_garbage() {
    force_collect();
}

/// GC statistics.
#[derive(Debug, Clone, Default)]
pub struct GcStats {
    pub object_count: usize,
    pub memory_used: usize,
}

/// Get current GC statistics.
pub fn gc_stats() -> GcStats {
    // TODO: Get actual stats from Boa when API is available
    GcStats::default()
}
