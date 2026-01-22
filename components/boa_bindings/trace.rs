// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// Tracing support for Boa GC integration
// Provides traits and utilities for tracing DOM objects

use boa_gc::{Trace, Tracer, Finalize};

use super::reflector::Reflector;

/// Trait for objects that can be traced by Boa's GC
/// 
/// This is the Boa equivalent of js::gc::Traceable
pub trait BoaTraceable: 'static {
    /// Trace the object for garbage collection
    /// 
    /// # Safety
    /// Must only be called from GC tracer
    unsafe fn trace_boa(&self, tracer: &mut Tracer);
}

// Blanket implementation for types that implement boa_gc::Trace
impl<T: Trace + 'static> BoaTraceable for T {
    unsafe fn trace_boa(&self, tracer: &mut Tracer) {
        unsafe { self.trace(tracer) };
    }
}

/// Trace a reflector
/// 
/// # Safety
/// tracer must be valid, reflector must be rooted
pub unsafe fn trace_reflector(_tracer: &mut Tracer, _info: &str, _reflector: &Reflector) {
    // In Boa, the GC handles tracing automatically through the Trace trait
    // This function is kept for API compatibility with SpiderMonkey
}

/// Trace a value that might be null
/// 
/// # Safety
/// If value is not null, it must be valid
pub unsafe fn trace_optional<T: Trace>(tracer: &mut Tracer, value: &Option<T>) {
    if let Some(v) = value {
        unsafe { v.trace(tracer) };
    }
}

/// Trace a raw pointer if not null
/// 
/// # Safety
/// If ptr is not null, it must point to valid data
pub unsafe fn trace_ptr<T: Trace>(tracer: &mut Tracer, ptr: *const T) {
    if !ptr.is_null() {
        unsafe { (*ptr).trace(tracer) };
    }
}

/// Trace a vector of traceable items
/// 
/// # Safety
/// All items must be valid
pub unsafe fn trace_vec<T: Trace>(tracer: &mut Tracer, vec: &[T]) {
    for item in vec {
        unsafe { item.trace(tracer) };
    }
}

/// Custom trace derive helper
/// 
/// This struct helps with implementing Trace for complex types
pub struct TraceHelper<'a> {
    tracer: &'a mut Tracer,
}

impl<'a> TraceHelper<'a> {
    /// Create a new trace helper
    pub fn new(tracer: &'a mut Tracer) -> Self {
        Self { tracer }
    }
    
    /// Trace a value
    /// 
    /// # Safety
    /// Value must be valid
    pub unsafe fn trace<T: Trace>(&mut self, value: &T) {
        unsafe { value.trace(self.tracer) };
    }
    
    /// Trace an optional value
    /// 
    /// # Safety
    /// If Some, value must be valid
    pub unsafe fn trace_opt<T: Trace>(&mut self, value: &Option<T>) {
        if let Some(v) = value {
            unsafe { v.trace(self.tracer) };
        }
    }
}

/// Macro to help implement Trace for DOM structs
#[macro_export]
macro_rules! impl_trace_for_dom {
    ($ty:ty { $($field:ident),* $(,)? }) => {
        unsafe impl boa_gc::Trace for $ty {
            unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
                $(
                    unsafe { self.$field.trace(tracer); }
                )*
            }
            
            unsafe fn trace_non_roots(&self) {
                $(
                    self.$field.trace_non_roots();
                )*
            }
            
            fn run_pointerifier(&self) {
                $(
                    self.$field.run_pointerifier();
                )*
            }
        }
    };
}

/// Macro to implement Finalize with custom cleanup
#[macro_export]
macro_rules! impl_finalize_for_dom {
    ($ty:ty) => {
        impl boa_gc::Finalize for $ty {
            fn finalize(&self) {
                // Default: no custom cleanup needed
            }
        }
    };
    ($ty:ty, $cleanup:expr) => {
        impl boa_gc::Finalize for $ty {
            fn finalize(&self) {
                $cleanup(self);
            }
        }
    };
}
