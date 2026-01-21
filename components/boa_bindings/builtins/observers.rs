// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Observer APIs for Boa.
//!
//! Provides stub implementations of:
//! - IntersectionObserver: Observe element visibility
//! - ResizeObserver: Observe element size changes
//! - MutationObserver: Observe DOM mutations
//!
//! Note: Full observer functionality requires integration with the layout engine.
//! These provide the API surface for compatibility.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor,
    object::builtins::JsArray,
};
use std::cell::RefCell;

// Thread-local observer tracking
thread_local! {
    static OBSERVER_ID_COUNTER: RefCell<u64> = RefCell::new(0);
}

fn next_observer_id() -> u64 {
    OBSERVER_ID_COUNTER.with(|counter| {
        let mut c = counter.borrow_mut();
        *c += 1;
        *c
    })
}

/// Register Observer APIs on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    // Register IntersectionObserver
    register_intersection_observer(context)?;
    
    // Register ResizeObserver
    register_resize_observer(context)?;
    
    // Register MutationObserver
    register_mutation_observer(context)?;
    
    Ok(())
}

// ============================================================================
// IntersectionObserver
// ============================================================================

fn register_intersection_observer(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(intersection_observer_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("IntersectionObserver"),
        PropertyDescriptor::builder()
            .value(constructor.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn intersection_observer_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);
    
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("IntersectionObserver: callback must be a function")
            .into());
    }
    
    let options = args.get_or_undefined(1);
    
    log::debug!("[IntersectionObserver] Constructor called (stub)");
    
    let observer = JsObject::with_null_proto();
    let observer_id = next_observer_id();
    
    // Store callback and options
    observer.define_property_or_throw(
        JsString::from("__id"),
        PropertyDescriptor::builder()
            .value(JsValue::from(observer_id as f64))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    observer.define_property_or_throw(
        JsString::from("__callback"),
        PropertyDescriptor::builder()
            .value(callback.clone())
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // Parse options
    let root = if let Some(obj) = options.as_object() {
        obj.get(JsString::from("root"), context).unwrap_or(JsValue::null())
    } else {
        JsValue::null()
    };
    
    let root_margin = if let Some(obj) = options.as_object() {
        obj.get(JsString::from("rootMargin"), context)
            .unwrap_or(JsValue::from(JsString::from("0px")))
    } else {
        JsValue::from(JsString::from("0px"))
    };
    
    let threshold = if let Some(obj) = options.as_object() {
        obj.get(JsString::from("threshold"), context)
            .unwrap_or(JsValue::from(0.0))
    } else {
        JsValue::from(0.0)
    };
    
    // root property (readonly)
    observer.define_property_or_throw(
        JsString::from("root"),
        PropertyDescriptor::builder()
            .value(root)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // rootMargin property (readonly)
    observer.define_property_or_throw(
        JsString::from("rootMargin"),
        PropertyDescriptor::builder()
            .value(root_margin)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // thresholds property (readonly array)
    let thresholds = {
        let arr = JsArray::new(context);
        // If threshold is array-like, use first element, otherwise use threshold value
        arr.set(0, threshold, true, context)?;
        JsValue::from(arr)
    };
    observer.define_property_or_throw(
        JsString::from("thresholds"),
        PropertyDescriptor::builder()
            .value(thresholds)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // observe(target)
    let observe = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let _target = args.get_or_undefined(0);
        log::debug!("[IntersectionObserver] observe called (stub)");
        Ok(JsValue::undefined())
    });
    observer.define_property_or_throw(
        JsString::from("observe"),
        PropertyDescriptor::builder()
            .value(observe.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // unobserve(target)
    let unobserve = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let _target = args.get_or_undefined(0);
        log::debug!("[IntersectionObserver] unobserve called (stub)");
        Ok(JsValue::undefined())
    });
    observer.define_property_or_throw(
        JsString::from("unobserve"),
        PropertyDescriptor::builder()
            .value(unobserve.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // disconnect()
    let disconnect = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[IntersectionObserver] disconnect called (stub)");
        Ok(JsValue::undefined())
    });
    observer.define_property_or_throw(
        JsString::from("disconnect"),
        PropertyDescriptor::builder()
            .value(disconnect.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // takeRecords()
    let take_records = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[IntersectionObserver] takeRecords called (stub)");
        Ok(JsValue::from(JsArray::new(ctx)))
    });
    observer.define_property_or_throw(
        JsString::from("takeRecords"),
        PropertyDescriptor::builder()
            .value(take_records.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(observer))
}

// ============================================================================
// ResizeObserver
// ============================================================================

fn register_resize_observer(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(resize_observer_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("ResizeObserver"),
        PropertyDescriptor::builder()
            .value(constructor.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn resize_observer_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);
    
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("ResizeObserver: callback must be a function")
            .into());
    }
    
    log::debug!("[ResizeObserver] Constructor called (stub)");
    
    let observer = JsObject::with_null_proto();
    let observer_id = next_observer_id();
    
    observer.define_property_or_throw(
        JsString::from("__id"),
        PropertyDescriptor::builder()
            .value(JsValue::from(observer_id as f64))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    observer.define_property_or_throw(
        JsString::from("__callback"),
        PropertyDescriptor::builder()
            .value(callback.clone())
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // observe(target, options?)
    let observe = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let _target = args.get_or_undefined(0);
        let _options = args.get_or_undefined(1);
        log::debug!("[ResizeObserver] observe called (stub)");
        Ok(JsValue::undefined())
    });
    observer.define_property_or_throw(
        JsString::from("observe"),
        PropertyDescriptor::builder()
            .value(observe.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // unobserve(target)
    let unobserve = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let _target = args.get_or_undefined(0);
        log::debug!("[ResizeObserver] unobserve called (stub)");
        Ok(JsValue::undefined())
    });
    observer.define_property_or_throw(
        JsString::from("unobserve"),
        PropertyDescriptor::builder()
            .value(unobserve.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // disconnect()
    let disconnect = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[ResizeObserver] disconnect called (stub)");
        Ok(JsValue::undefined())
    });
    observer.define_property_or_throw(
        JsString::from("disconnect"),
        PropertyDescriptor::builder()
            .value(disconnect.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(observer))
}

// ============================================================================
// MutationObserver
// ============================================================================

fn register_mutation_observer(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(mutation_observer_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("MutationObserver"),
        PropertyDescriptor::builder()
            .value(constructor.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn mutation_observer_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);
    
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("MutationObserver: callback must be a function")
            .into());
    }
    
    log::debug!("[MutationObserver] Constructor called (stub)");
    
    let observer = JsObject::with_null_proto();
    let observer_id = next_observer_id();
    
    observer.define_property_or_throw(
        JsString::from("__id"),
        PropertyDescriptor::builder()
            .value(JsValue::from(observer_id as f64))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    observer.define_property_or_throw(
        JsString::from("__callback"),
        PropertyDescriptor::builder()
            .value(callback.clone())
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // observe(target, options)
    let observe = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let _target = args.get_or_undefined(0);
        let _options = args.get_or_undefined(1);
        log::debug!("[MutationObserver] observe called (stub)");
        Ok(JsValue::undefined())
    });
    observer.define_property_or_throw(
        JsString::from("observe"),
        PropertyDescriptor::builder()
            .value(observe.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // disconnect()
    let disconnect = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[MutationObserver] disconnect called (stub)");
        Ok(JsValue::undefined())
    });
    observer.define_property_or_throw(
        JsString::from("disconnect"),
        PropertyDescriptor::builder()
            .value(disconnect.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // takeRecords()
    let take_records = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
        log::debug!("[MutationObserver] takeRecords called (stub)");
        Ok(JsValue::from(JsArray::new(ctx)))
    });
    observer.define_property_or_throw(
        JsString::from("takeRecords"),
        PropertyDescriptor::builder()
            .value(take_records.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(observer))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;
    
    #[test]
    fn test_intersection_observer_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof IntersectionObserver"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_intersection_observer_creation() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            let observer = IntersectionObserver(() => {});
            typeof observer.observe === 'function' && 
            typeof observer.disconnect === 'function'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_intersection_observer_with_options() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            let observer = IntersectionObserver(() => {}, { 
                threshold: 0.5, 
                rootMargin: '10px' 
            });
            observer.rootMargin === '10px'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_resize_observer_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof ResizeObserver"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_resize_observer_creation() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            let observer = ResizeObserver(() => {});
            typeof observer.observe === 'function' && 
            typeof observer.unobserve === 'function' &&
            typeof observer.disconnect === 'function'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_mutation_observer_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof MutationObserver"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_mutation_observer_creation() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            let observer = MutationObserver(() => {});
            typeof observer.observe === 'function' && 
            typeof observer.takeRecords === 'function' &&
            typeof observer.disconnect === 'function'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_observer_requires_callback() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            try {
                IntersectionObserver('not a function');
                false;
            } catch(e) {
                true;
            }
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
}
