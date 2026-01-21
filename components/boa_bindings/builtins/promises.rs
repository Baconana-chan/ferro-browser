// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Promise utilities and microtask queue for Boa.
//!
//! Provides:
//! - queueMicrotask(): Queue a microtask
//! - Promise utilities for async/await support
//!
//! Note: Boa already has native Promise support. This module adds
//! Web platform specific utilities.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor, object::builtins::JsPromise,
};
use std::cell::RefCell;
use std::collections::VecDeque;

// Microtask queue
thread_local! {
    static MICROTASK_QUEUE: RefCell<VecDeque<JsObject>> = RefCell::new(VecDeque::new());
}

/// Register Promise utilities on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    // Register queueMicrotask
    register_queue_microtask(context)?;
    
    // Register Promise.withResolvers (ES2024)
    register_promise_with_resolvers(context)?;
    
    Ok(())
}

// ============================================================================
// queueMicrotask
// ============================================================================

fn register_queue_microtask(context: &mut Context) -> JsResult<()> {
    let func = NativeFunction::from_fn_ptr(queue_microtask);
    
    context.global_object().define_property_or_throw(
        JsString::from("queueMicrotask"),
        PropertyDescriptor::builder()
            .value(func.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn queue_microtask(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);
    
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("queueMicrotask: callback must be a function")
            .into());
    }
    
    let callback_obj = callback.as_object().unwrap().clone();
    
    // Queue the microtask
    MICROTASK_QUEUE.with(|queue| {
        queue.borrow_mut().push_back(callback_obj);
    });
    
    // Run microtasks immediately (simplified implementation)
    run_microtasks(context)?;
    
    Ok(JsValue::undefined())
}

/// Run all queued microtasks.
pub fn run_microtasks(context: &mut Context) -> JsResult<()> {
    loop {
        let task = MICROTASK_QUEUE.with(|queue| {
            queue.borrow_mut().pop_front()
        });
        
        match task {
            Some(callback) => {
                // Execute the callback - JsObject::call works for callable objects
                if callback.is_callable() {
                    if let Err(e) = callback.call(&JsValue::undefined(), &[], context) {
                        log::warn!("[Microtask] Error executing microtask: {:?}", e);
                    }
                }
            }
            None => break,
        }
    }
    
    Ok(())
}

// ============================================================================
// Promise.withResolvers (ES2024)
// ============================================================================

fn register_promise_with_resolvers(context: &mut Context) -> JsResult<()> {
    // Get the Promise constructor
    let global = context.global_object();
    let promise_val = global.get(JsString::from("Promise"), context)?;
    
    if let Some(promise_constructor) = promise_val.as_object() {
        let func = NativeFunction::from_fn_ptr(promise_with_resolvers);
        
        promise_constructor.define_property_or_throw(
            JsString::from("withResolvers"),
            PropertyDescriptor::builder()
                .value(func.to_js_function(context.realm()))
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
    }
    
    Ok(())
}

fn promise_with_resolvers(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create the promise with pending resolvers using Boa's built-in mechanism
    let (promise, resolvers) = JsPromise::new_pending(context);
    
    let result = JsObject::with_null_proto();
    
    result.define_property_or_throw(
        JsString::from("promise"),
        PropertyDescriptor::builder()
            .value(promise)
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Pass the resolve function directly from resolvers
    result.define_property_or_throw(
        JsString::from("resolve"),
        PropertyDescriptor::builder()
            .value(resolvers.resolve)
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Pass the reject function directly from resolvers
    result.define_property_or_throw(
        JsString::from("reject"),
        PropertyDescriptor::builder()
            .value(resolvers.reject)
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(result))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;
    
    #[test]
    fn test_queue_microtask_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof queueMicrotask"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_queue_microtask_execution() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        // Set up a global variable to track execution
        ctx.eval(Source::from_bytes("var executed = false")).unwrap();
        
        ctx.eval(Source::from_bytes(
            "queueMicrotask(function() { executed = true; })"
        )).unwrap();
        
        let result = ctx.eval(Source::from_bytes("executed")).unwrap();
        assert!(result.to_boolean());
    }
    
    #[test]
    fn test_queue_microtask_order() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        ctx.eval(Source::from_bytes("var order = []")).unwrap();
        
        ctx.eval(Source::from_bytes(r#"
            queueMicrotask(function() { order.push(1); });
            queueMicrotask(function() { order.push(2); });
            queueMicrotask(function() { order.push(3); });
        "#)).unwrap();
        
        let result = ctx.eval(Source::from_bytes("order.join(',')")).unwrap();
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "1,2,3");
    }
    
    #[test]
    fn test_queue_microtask_requires_function() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes("queueMicrotask('not a function')"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_promise_native_support() {
        let mut ctx = Context::default();
        
        // Test that Boa has native Promise support
        let result = ctx.eval(Source::from_bytes(
            "typeof Promise"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_promise_resolve() {
        let mut ctx = Context::default();
        
        ctx.eval(Source::from_bytes("var resolved = false")).unwrap();
        
        ctx.eval(Source::from_bytes(r#"
            Promise.resolve(42).then(function(v) {
                resolved = v === 42;
            });
        "#)).unwrap();
        
        // Run the job queue
        ctx.run_jobs();
        
        let result = ctx.eval(Source::from_bytes("resolved")).unwrap();
        assert!(result.to_boolean());
    }
    
    #[test]
    fn test_promise_async_await() {
        let mut ctx = Context::default();
        
        ctx.eval(Source::from_bytes("var result = 0")).unwrap();
        
        ctx.eval(Source::from_bytes(r#"
            async function test() {
                const value = await Promise.resolve(42);
                result = value;
                return value;
            }
            test();
        "#)).unwrap();
        
        // Run the job queue to execute async code
        ctx.run_jobs();
        
        let result = ctx.eval(Source::from_bytes("result")).unwrap();
        assert_eq!(result.to_number(&mut ctx).unwrap(), 42.0);
    }
    
    #[test]
    fn test_async_await_sequential() {
        let mut ctx = Context::default();
        
        ctx.eval(Source::from_bytes("var steps = []")).unwrap();
        
        ctx.eval(Source::from_bytes(r#"
            async function test() {
                steps.push(1);
                await Promise.resolve();
                steps.push(2);
                await Promise.resolve();
                steps.push(3);
            }
            test();
        "#)).unwrap();
        
        ctx.run_jobs();
        
        let result = ctx.eval(Source::from_bytes("steps.join(',')")).unwrap();
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "1,2,3");
    }
}
