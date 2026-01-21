// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Reflector pattern for DOM objects in Boa.
//!
//! The Reflector is the core abstraction connecting Rust DOM objects
//! to their JavaScript representations. Each DOM object has a Reflector
//! that holds the JsObject wrapper.

use boa_engine::{Context, JsObject, JsResult, JsValue, JsString};
use boa_engine::property::PropertyDescriptor;
use boa_gc::{Finalize, Gc, GcRefCell, Trace};
use std::cell::RefCell;

/// A reference to the JavaScript object wrapper for a DOM object.
#[derive(Default)]
pub struct Reflector {
    object: RefCell<Option<JsObject>>,
}

impl std::fmt::Debug for Reflector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reflector")
            .field("initialized", &self.is_initialized())
            .finish()
    }
}

impl Reflector {
    /// Create a new, uninitialized Reflector.
    pub fn new() -> Self {
        Self {
            object: RefCell::new(None),
        }
    }

    /// Get the JavaScript object for this DOM object.
    pub fn get_jsobject(&self) -> JsObject {
        self.object.borrow().clone().expect("Reflector not initialized")
    }

    /// Get the JavaScript object as a JsValue.
    pub fn get_jsval(&self) -> JsValue {
        JsValue::from(self.get_jsobject())
    }

    /// Check if the reflector has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.object.borrow().is_some()
    }

    /// Set the JavaScript object for this reflector.
    pub fn set_jsobject(&self, obj: JsObject) {
        debug_assert!(!self.is_initialized(), "Reflector already initialized");
        *self.object.borrow_mut() = Some(obj);
    }
}

unsafe impl Trace for Reflector {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {}
    unsafe fn trace_non_roots(&self) {}
    fn run_finalizer(&self) {}
}

impl Finalize for Reflector {}

impl Clone for Reflector {
    fn clone(&self) -> Self {
        Self {
            object: RefCell::new(self.object.borrow().clone()),
        }
    }
}

/// Trait for DOM objects that have a JavaScript representation.
pub trait DomObject: Trace + Finalize + 'static {
    fn reflector(&self) -> &Reflector;

    fn js_object(&self) -> JsObject {
        self.reflector().get_jsobject()
    }

    fn js_value(&self) -> JsValue {
        self.reflector().get_jsval()
    }
}

/// A GC-managed reference to a DOM object.
#[derive(Clone)]
pub struct Dom<T: Trace + Finalize + 'static> {
    inner: Gc<T>,
}

impl<T: Trace + Finalize + 'static> Dom<T> {
    pub fn new(value: T) -> Self {
        Self { inner: Gc::new(value) }
    }

    pub fn get(&self) -> &T {
        &self.inner
    }
}

impl<T: Trace + Finalize + 'static> std::ops::Deref for Dom<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

unsafe impl<T: Trace + Finalize + 'static> Trace for Dom<T> {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        unsafe { self.inner.trace(tracer) };
    }
    
    unsafe fn trace_non_roots(&self) {
        unsafe { self.inner.trace_non_roots() };
    }
    
    fn run_finalizer(&self) {
        self.inner.run_finalizer();
    }
}

impl<T: Trace + Finalize + 'static> Finalize for Dom<T> {}

/// A mutable GC-managed reference to a DOM object.
pub struct DomRefCell<T: Trace + Finalize + 'static> {
    inner: GcRefCell<T>,
}

impl<T: Trace + Finalize + 'static> DomRefCell<T> {
    pub fn new(value: T) -> Self {
        Self { inner: GcRefCell::new(value) }
    }

    pub fn borrow(&self) -> boa_gc::GcRef<'_, T> {
        self.inner.borrow()
    }

    pub fn borrow_mut(&self) -> boa_gc::GcRefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

const DOM_SLOT_SYMBOL: &str = "__ferro_dom_slot";

/// Store a DOM object reference in its JavaScript wrapper.
pub fn set_dom_slot<T: DomObject>(obj: &JsObject, _dom: &T, context: &mut Context) -> JsResult<()> {
    obj.define_property_or_throw(
        JsString::from(DOM_SLOT_SYMBOL),
        PropertyDescriptor::builder()
            .value(JsValue::from(true))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    Ok(())
}

/// Create a JavaScript wrapper object for a DOM interface.
pub fn create_dom_wrapper<T: DomObject>(
    dom: &T,
    _prototype: Option<&JsObject>,
    context: &mut Context,
) -> JsResult<JsObject> {
    let obj = JsObject::with_null_proto();
    set_dom_slot(&obj, dom, context)?;
    dom.reflector().set_jsobject(obj.clone());
    Ok(obj)
}

/// Define a method on a prototype object.
pub fn define_method(
    prototype: &JsObject,
    name: &str,
    _length: usize,
    func: boa_engine::NativeFunction,
    context: &mut Context,
) -> JsResult<()> {
    let js_func = func.to_js_function(context.realm());
    prototype.define_property_or_throw(
        JsString::from(name),
        PropertyDescriptor::builder()
            .value(js_func)
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    Ok(())
}

/// Define a getter on a prototype object.
pub fn define_getter(
    prototype: &JsObject,
    name: &str,
    getter: boa_engine::NativeFunction,
    context: &mut Context,
) -> JsResult<()> {
    let getter_func = getter.to_js_function(context.realm());
    prototype.define_property_or_throw(
        JsString::from(name),
        PropertyDescriptor::builder()
            .get(getter_func)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    Ok(())
}

/// Define a getter and setter on a prototype object.
pub fn define_accessor(
    prototype: &JsObject,
    name: &str,
    getter: boa_engine::NativeFunction,
    setter: boa_engine::NativeFunction,
    context: &mut Context,
) -> JsResult<()> {
    let getter_func = getter.to_js_function(context.realm());
    let setter_func = setter.to_js_function(context.realm());
    prototype.define_property_or_throw(
        JsString::from(name),
        PropertyDescriptor::builder()
            .get(getter_func)
            .set(setter_func)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestDomObject {
        reflector: Reflector,
        value: i32,
    }

    unsafe impl Trace for TestDomObject {
        unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
            unsafe { self.reflector.trace(tracer) };
        }
        unsafe fn trace_non_roots(&self) {}
        fn run_finalizer(&self) {}
    }

    impl Finalize for TestDomObject {}

    impl DomObject for TestDomObject {
        fn reflector(&self) -> &Reflector {
            &self.reflector
        }
    }

    #[test]
    fn test_reflector_lifecycle() {
        let obj = TestDomObject {
            reflector: Reflector::new(),
            value: 42,
        };
        assert!(!obj.reflector().is_initialized());
    }

    #[test]
    fn test_dom_wrapper() {
        let dom = Dom::new(TestDomObject {
            reflector: Reflector::new(),
            value: 123,
        });
        assert_eq!(dom.value, 123);
    }
}
