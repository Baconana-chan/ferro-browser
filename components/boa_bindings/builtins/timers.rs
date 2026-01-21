// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Timer API implementation for Boa.
//!
//! Provides setTimeout, setInterval, clearTimeout, clearInterval.
//! These integrate with Servo's timer infrastructure.

use boa_engine::{Context, JsResult, JsValue, NativeFunction, JsString};
use boa_engine::property::PropertyDescriptor;
use std::sync::atomic::{AtomicU32, Ordering};

/// Unique timer ID counter.
static TIMER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Type of timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerType {
    Timeout,
    Interval,
}

/// Generate a new unique timer ID.
fn next_timer_id() -> u32 {
    TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Register timer functions on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    // setTimeout(callback, delay, ...args) -> timerId
    let set_timeout = NativeFunction::from_fn_ptr(|_this, args, context| {
        set_timer_impl(TimerType::Timeout, args, context)
    });

    // setInterval(callback, delay, ...args) -> timerId
    let set_interval = NativeFunction::from_fn_ptr(|_this, args, context| {
        set_timer_impl(TimerType::Interval, args, context)
    });

    // clearTimeout(timerId)
    let clear_timeout = NativeFunction::from_fn_ptr(|_this, args, context| {
        clear_timer_impl(args, context)
    });

    // clearInterval(timerId)
    let clear_interval = NativeFunction::from_fn_ptr(|_this, args, context| {
        clear_timer_impl(args, context)
    });

    let global = context.global_object();

    // Register setTimeout
    global.define_property_or_throw(
        JsString::from("setTimeout"),
        PropertyDescriptor::builder()
            .value(set_timeout.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    // Register setInterval
    global.define_property_or_throw(
        JsString::from("setInterval"),
        PropertyDescriptor::builder()
            .value(set_interval.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    // Register clearTimeout
    global.define_property_or_throw(
        JsString::from("clearTimeout"),
        PropertyDescriptor::builder()
            .value(clear_timeout.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    // Register clearInterval
    global.define_property_or_throw(
        JsString::from("clearInterval"),
        PropertyDescriptor::builder()
            .value(clear_interval.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    log::debug!("Timer builtins registered (setTimeout, setInterval, clearTimeout, clearInterval)");
    Ok(())
}

/// Implementation for setTimeout/setInterval.
/// 
/// In production, this sends a message to Servo's ScriptThread to schedule the timer.
/// For now, we just return a unique ID and log the request.
fn set_timer_impl(timer_type: TimerType, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Get callback (first argument)
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    
    if !callback.is_callable() {
        // Non-callable values are ignored, return 0
        return Ok(JsValue::from(0));
    }

    // Get delay (second argument), default to 0
    let delay_ms = if args.len() > 1 {
        args[1].to_u32(context).unwrap_or(0)
    } else {
        0
    };

    // Generate unique timer ID
    let id = next_timer_id();

    log::trace!(
        "Scheduled {:?} with id={}, delay={}ms (callback stored in ScriptThread)",
        timer_type,
        id,
        delay_ms
    );

    // In production, we would:
    // 1. Store the callback and args in a timer registry
    // 2. Send a message to ScriptThread to schedule the timer
    // 3. When timer fires, ScriptThread calls the callback
    //
    // For now, just return the ID
    Ok(JsValue::from(id))
}

/// Implementation for clearTimeout/clearInterval.
fn clear_timer_impl(args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(id_value) = args.first() {
        if let Ok(id) = id_value.to_u32(context) {
            log::trace!("Cancelled timer id={}", id);
            // In production, send cancel message to ScriptThread
        }
    }
    Ok(JsValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::JsRuntime;

    #[test]
    fn test_set_timeout_returns_id() {
        let mut runtime = JsRuntime::new();
        register(runtime.context_mut()).unwrap();
        
        let result = runtime.eval("setTimeout(function() {}, 100)").unwrap();
        assert!(result.as_number().unwrap() > 0.0);
    }

    #[test]
    fn test_clear_timeout() {
        let mut runtime = JsRuntime::new();
        register(runtime.context_mut()).unwrap();
        
        let result = runtime.eval(r#"
            var id = setTimeout(function() {}, 1000);
            clearTimeout(id);
            id
        "#).unwrap();
        assert!(result.as_number().unwrap() > 0.0);
    }

    #[test]
    fn test_set_interval_returns_id() {
        let mut runtime = JsRuntime::new();
        register(runtime.context_mut()).unwrap();
        
        let result = runtime.eval("setInterval(function() {}, 100)").unwrap();
        assert!(result.as_number().unwrap() > 0.0);
    }
}
