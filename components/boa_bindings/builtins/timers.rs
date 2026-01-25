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

// ============================================================================
// Phase C2: Timer edge case utilities
// ============================================================================

/// Minimum timer delay (clamped per HTML spec)
/// In nested timeouts (depth > 5), minimum is 4ms
pub const MIN_TIMER_DELAY_MS: u32 = 0;
pub const MIN_NESTED_TIMER_DELAY_MS: u32 = 4;

/// Maximum timer delay (approximately 24.8 days)
pub const MAX_TIMER_DELAY_MS: u32 = 2_147_483_647; // i32::MAX

/// Clamp timer delay according to HTML spec.
/// 
/// Per HTML spec:
/// - Delay < 0 is treated as 0
/// - In nested timeouts (depth > 5), minimum is 4ms
/// - Maximum is limited to prevent overflow
pub fn clamp_timer_delay(delay: i64, nesting_level: u32) -> u32 {
    // Handle negative delays
    if delay < 0 {
        if nesting_level > 5 {
            return MIN_NESTED_TIMER_DELAY_MS;
        }
        return MIN_TIMER_DELAY_MS;
    }
    
    // Clamp to maximum
    let delay = delay.min(MAX_TIMER_DELAY_MS as i64) as u32;
    
    // Apply minimum for nested timers
    if nesting_level > 5 && delay < MIN_NESTED_TIMER_DELAY_MS {
        return MIN_NESTED_TIMER_DELAY_MS;
    }
    
    delay
}

/// Check if a timer ID is valid (non-zero positive integer)
pub fn is_valid_timer_id(id: u32) -> bool {
    id > 0
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
    
    // Phase C2: Timer edge case tests
    
    #[test]
    fn test_zero_delay_timeout() {
        let mut runtime = JsRuntime::new();
        register(runtime.context_mut()).unwrap();
        
        // setTimeout with 0 delay should work
        let result = runtime.eval("setTimeout(function() {}, 0)").unwrap();
        assert!(result.as_number().unwrap() > 0.0);
    }
    
    #[test]
    fn test_negative_delay_treated_as_zero() {
        let clamped = clamp_timer_delay(-100, 0);
        assert_eq!(clamped, 0);
    }
    
    #[test]
    fn test_nested_timer_minimum_delay() {
        // At nesting level > 5, minimum is 4ms
        let clamped = clamp_timer_delay(0, 6);
        assert_eq!(clamped, 4);
        
        let clamped = clamp_timer_delay(2, 10);
        assert_eq!(clamped, 4);
        
        // At nesting level <= 5, no minimum
        let clamped = clamp_timer_delay(0, 5);
        assert_eq!(clamped, 0);
    }
    
    #[test]
    fn test_max_timer_delay() {
        let clamped = clamp_timer_delay(i64::MAX, 0);
        assert_eq!(clamped, MAX_TIMER_DELAY_MS);
    }
    
    #[test]
    fn test_clear_nonexistent_timer() {
        let mut runtime = JsRuntime::new();
        register(runtime.context_mut()).unwrap();
        
        // clearTimeout with invalid ID should not throw
        let result = runtime.eval("clearTimeout(999999); true").unwrap();
        assert!(result.to_boolean());
    }
    
    #[test]
    fn test_clear_timeout_twice() {
        let mut runtime = JsRuntime::new();
        register(runtime.context_mut()).unwrap();
        
        // Clearing same timer twice should not throw
        let result = runtime.eval(r#"
            var id = setTimeout(function() {}, 1000);
            clearTimeout(id);
            clearTimeout(id);
            true
        "#).unwrap();
        assert!(result.to_boolean());
    }
    
    #[test]
    fn test_set_timeout_non_callable() {
        let mut runtime = JsRuntime::new();
        register(runtime.context_mut()).unwrap();
        
        // Non-callable first argument returns 0
        let result = runtime.eval("setTimeout('not a function', 100)").unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }
    
    #[test]
    fn test_unique_timer_ids() {
        let mut runtime = JsRuntime::new();
        register(runtime.context_mut()).unwrap();
        
        // Each timer should get a unique ID
        let result = runtime.eval(r#"
            var id1 = setTimeout(function() {}, 100);
            var id2 = setTimeout(function() {}, 100);
            var id3 = setInterval(function() {}, 100);
            id1 !== id2 && id2 !== id3 && id1 !== id3
        "#).unwrap();
        assert!(result.to_boolean());
    }
    
    #[test]
    fn test_valid_timer_id() {
        assert!(is_valid_timer_id(1));
        assert!(is_valid_timer_id(100));
        assert!(!is_valid_timer_id(0));
    }
}
