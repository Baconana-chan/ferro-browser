// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Performance API implementation for Boa.
//!
//! Provides:
//! - performance.now(): High-resolution timestamp
//! - performance.timeOrigin: Time origin
//! - performance.mark(): Create a performance mark
//! - performance.measure(): Create a performance measure
//! - performance.getEntries(): Get all performance entries
//! - performance.getEntriesByName(): Get entries by name
//! - performance.getEntriesByType(): Get entries by type
//! - performance.clearMarks(): Clear marks
//! - performance.clearMeasures(): Clear measures
//! - PerformanceObserver: Observe performance entries

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor, object::builtins::JsArray,
};
use std::cell::RefCell;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// Thread-local storage for performance entries
thread_local! {
    static PERFORMANCE_ENTRIES: RefCell<Vec<PerformanceEntry>> = RefCell::new(Vec::new());
    static TIME_ORIGIN: RefCell<Option<(Instant, f64)>> = RefCell::new(None);
}

#[derive(Clone, Debug)]
struct PerformanceEntry {
    name: String,
    entry_type: String,
    start_time: f64,
    duration: f64,
    detail: Option<String>,
}

fn get_time_origin() -> (Instant, f64) {
    TIME_ORIGIN.with(|origin| {
        let mut origin = origin.borrow_mut();
        if origin.is_none() {
            let now = Instant::now();
            let unix_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64() * 1000.0)
                .unwrap_or(0.0);
            *origin = Some((now, unix_time));
        }
        origin.unwrap()
    })
}

fn performance_now() -> f64 {
    let (origin, _) = get_time_origin();
    origin.elapsed().as_secs_f64() * 1000.0
}

/// Register Performance API on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    let performance = create_performance_object(context)?;
    
    context.global_object().define_property_or_throw(
        JsString::from("performance"),
        PropertyDescriptor::builder()
            .value(performance)
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Register PerformanceObserver constructor
    register_performance_observer(context)?;
    
    Ok(())
}

fn create_performance_object(context: &mut Context) -> JsResult<JsValue> {
    let perf = JsObject::with_null_proto();
    
    // timeOrigin (readonly)
    let (_, time_origin) = get_time_origin();
    perf.define_property_or_throw(
        JsString::from("timeOrigin"),
        PropertyDescriptor::builder()
            .value(JsValue::from(time_origin))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // now()
    register_method(&perf, "now", perf_now, context)?;
    
    // mark()
    register_method(&perf, "mark", perf_mark, context)?;
    
    // measure()
    register_method(&perf, "measure", perf_measure, context)?;
    
    // getEntries()
    register_method(&perf, "getEntries", perf_get_entries, context)?;
    
    // getEntriesByName()
    register_method(&perf, "getEntriesByName", perf_get_entries_by_name, context)?;
    
    // getEntriesByType()
    register_method(&perf, "getEntriesByType", perf_get_entries_by_type, context)?;
    
    // clearMarks()
    register_method(&perf, "clearMarks", perf_clear_marks, context)?;
    
    // clearMeasures()
    register_method(&perf, "clearMeasures", perf_clear_measures, context)?;
    
    // clearResourceTimings()
    register_method(&perf, "clearResourceTimings", perf_clear_resource_timings, context)?;
    
    // toJSON()
    register_method(&perf, "toJSON", perf_to_json, context)?;
    
    Ok(JsValue::from(perf))
}

fn register_method(obj: &JsObject, name: &str, func: fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>, context: &mut Context) -> JsResult<()> {
    let js_func = NativeFunction::from_fn_ptr(func)
        .to_js_function(context.realm());
    
    obj.define_property_or_throw(
        JsString::from(name),
        PropertyDescriptor::builder()
            .value(js_func)
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn perf_now(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(performance_now()))
}

fn perf_mark(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let name = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    let options = args.get_or_undefined(1);
    
    // Get start time (default to now)
    let start_time = if let Some(obj) = options.as_object() {
        let start_val = obj.get(JsString::from("startTime"), context)?;
        if !start_val.is_undefined() {
            start_val.to_number(context)?
        } else {
            performance_now()
        }
    } else {
        performance_now()
    };
    
    // Get detail
    let detail = if let Some(obj) = options.as_object() {
        let detail_val = obj.get(JsString::from("detail"), context)?;
        if !detail_val.is_undefined() {
            Some(detail_val.to_string(context)?.to_std_string_escaped())
        } else {
            None
        }
    } else {
        None
    };
    
    let entry = PerformanceEntry {
        name: name.clone(),
        entry_type: "mark".to_string(),
        start_time,
        duration: 0.0,
        detail,
    };
    
    PERFORMANCE_ENTRIES.with(|entries| {
        entries.borrow_mut().push(entry.clone());
    });
    
    // Return PerformanceMark object
    create_performance_entry_object(&entry, context)
}

fn perf_measure(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let name = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    let start_or_options = args.get_or_undefined(1);
    let end_mark = args.get_or_undefined(2);
    
    let (start_time, end_time, detail) = if let Some(obj) = start_or_options.as_object() {
        // Options object
        let start_val = obj.get(JsString::from("start"), context)?;
        let end_val = obj.get(JsString::from("end"), context)?;
        let duration_val = obj.get(JsString::from("duration"), context)?;
        let detail_val = obj.get(JsString::from("detail"), context)?;
        
        let start = if start_val.is_string() {
            // Find mark by name
            find_mark_time(&start_val.to_string(context)?.to_std_string_escaped())
                .unwrap_or(0.0)
        } else if !start_val.is_undefined() {
            start_val.to_number(context)?
        } else {
            0.0
        };
        
        let end = if end_val.is_string() {
            find_mark_time(&end_val.to_string(context)?.to_std_string_escaped())
                .unwrap_or(performance_now())
        } else if !end_val.is_undefined() {
            end_val.to_number(context)?
        } else if !duration_val.is_undefined() {
            start + duration_val.to_number(context)?
        } else {
            performance_now()
        };
        
        let detail = if !detail_val.is_undefined() {
            Some(detail_val.to_string(context)?.to_std_string_escaped())
        } else {
            None
        };
        
        (start, end, detail)
    } else if start_or_options.is_string() {
        // Start mark name
        let start = find_mark_time(&start_or_options.to_string(context)?.to_std_string_escaped())
            .unwrap_or(0.0);
        let end = if end_mark.is_string() {
            find_mark_time(&end_mark.to_string(context)?.to_std_string_escaped())
                .unwrap_or(performance_now())
        } else {
            performance_now()
        };
        (start, end, None)
    } else {
        (0.0, performance_now(), None)
    };
    
    let entry = PerformanceEntry {
        name: name.clone(),
        entry_type: "measure".to_string(),
        start_time,
        duration: end_time - start_time,
        detail,
    };
    
    PERFORMANCE_ENTRIES.with(|entries| {
        entries.borrow_mut().push(entry.clone());
    });
    
    create_performance_entry_object(&entry, context)
}

fn find_mark_time(name: &str) -> Option<f64> {
    PERFORMANCE_ENTRIES.with(|entries| {
        entries.borrow().iter()
            .rev()
            .find(|e| e.entry_type == "mark" && e.name == name)
            .map(|e| e.start_time)
    })
}

fn create_performance_entry_object(entry: &PerformanceEntry, context: &mut Context) -> JsResult<JsValue> {
    let obj = JsObject::with_null_proto();
    
    obj.define_property_or_throw(
        JsString::from("name"),
        PropertyDescriptor::builder()
            .value(JsString::from(entry.name.as_str()))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    obj.define_property_or_throw(
        JsString::from("entryType"),
        PropertyDescriptor::builder()
            .value(JsString::from(entry.entry_type.as_str()))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    obj.define_property_or_throw(
        JsString::from("startTime"),
        PropertyDescriptor::builder()
            .value(JsValue::from(entry.start_time))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    obj.define_property_or_throw(
        JsString::from("duration"),
        PropertyDescriptor::builder()
            .value(JsValue::from(entry.duration))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    if let Some(detail) = &entry.detail {
        obj.define_property_or_throw(
            JsString::from("detail"),
            PropertyDescriptor::builder()
                .value(JsString::from(detail.as_str()))
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
            context,
        )?;
    }
    
    // toJSON method
    let to_json = NativeFunction::from_fn_ptr(|this, _, _ctx| {
        // Just return this for simplicity
        Ok(this.clone())
    });
    obj.define_property_or_throw(
        JsString::from("toJSON"),
        PropertyDescriptor::builder()
            .value(to_json.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(obj))
}

fn perf_get_entries(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let result = JsArray::new(context);
    
    PERFORMANCE_ENTRIES.with(|entries| {
        for (i, entry) in entries.borrow().iter().enumerate() {
            let obj = create_performance_entry_object(entry, context)?;
            result.set(i as u64, obj, true, context)?;
        }
        Ok::<_, boa_engine::JsError>(())
    })?;
    
    Ok(JsValue::from(result))
}

fn perf_get_entries_by_name(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let name = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    let entry_type = if !args.get_or_undefined(1).is_undefined() {
        Some(args.get_or_undefined(1).to_string(context)?.to_std_string_escaped())
    } else {
        None
    };
    
    let result = JsArray::new(context);
    let mut idx = 0u64;
    
    PERFORMANCE_ENTRIES.with(|entries| {
        for entry in entries.borrow().iter() {
            if entry.name == name {
                if let Some(ref et) = entry_type {
                    if &entry.entry_type != et {
                        continue;
                    }
                }
                let obj = create_performance_entry_object(entry, context)?;
                result.set(idx, obj, true, context)?;
                idx += 1;
            }
        }
        Ok::<_, boa_engine::JsError>(())
    })?;
    
    Ok(JsValue::from(result))
}

fn perf_get_entries_by_type(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let entry_type = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    let result = JsArray::new(context);
    let mut idx = 0u64;
    
    PERFORMANCE_ENTRIES.with(|entries| {
        for entry in entries.borrow().iter() {
            if entry.entry_type == entry_type {
                let obj = create_performance_entry_object(entry, context)?;
                result.set(idx, obj, true, context)?;
                idx += 1;
            }
        }
        Ok::<_, boa_engine::JsError>(())
    })?;
    
    Ok(JsValue::from(result))
}

fn perf_clear_marks(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let name = if !args.get_or_undefined(0).is_undefined() {
        Some(args.get_or_undefined(0).to_string(context)?.to_std_string_escaped())
    } else {
        None
    };
    
    PERFORMANCE_ENTRIES.with(|entries| {
        let mut entries = entries.borrow_mut();
        entries.retain(|e| {
            if e.entry_type != "mark" {
                return true;
            }
            if let Some(ref n) = name {
                e.name != *n
            } else {
                false
            }
        });
    });
    
    Ok(JsValue::undefined())
}

fn perf_clear_measures(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let name = if !args.get_or_undefined(0).is_undefined() {
        Some(args.get_or_undefined(0).to_string(context)?.to_std_string_escaped())
    } else {
        None
    };
    
    PERFORMANCE_ENTRIES.with(|entries| {
        let mut entries = entries.borrow_mut();
        entries.retain(|e| {
            if e.entry_type != "measure" {
                return true;
            }
            if let Some(ref n) = name {
                e.name != *n
            } else {
                false
            }
        });
    });
    
    Ok(JsValue::undefined())
}

fn perf_clear_resource_timings(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    PERFORMANCE_ENTRIES.with(|entries| {
        let mut entries = entries.borrow_mut();
        entries.retain(|e| e.entry_type != "resource");
    });
    
    Ok(JsValue::undefined())
}

fn perf_to_json(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = JsObject::with_null_proto();
    
    let (_, time_origin) = get_time_origin();
    
    obj.define_property_or_throw(
        JsString::from("timeOrigin"),
        PropertyDescriptor::builder()
            .value(JsValue::from(time_origin))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(obj))
}

// ============================================================================
// PerformanceObserver
// ============================================================================

fn register_performance_observer(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(performance_observer_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("PerformanceObserver"),
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

fn performance_observer_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);
    
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("PerformanceObserver: callback must be a function")
            .into());
    }
    
    let observer = JsObject::with_null_proto();
    
    // Store callback
    observer.define_property_or_throw(
        JsString::from("__callback"),
        PropertyDescriptor::builder()
            .value(callback.clone())
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // observe method
    let observe = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        // Stub - in real implementation, this would register the observer
        log::debug!("[PerformanceObserver] observe called (stub)");
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
    
    // disconnect method
    let disconnect = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[PerformanceObserver] disconnect called (stub)");
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
    
    // takeRecords method
    let take_records = NativeFunction::from_fn_ptr(|_this, _args, ctx| {
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
    fn test_performance_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof performance"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "object");
    }
    
    #[test]
    fn test_performance_now() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof performance.now()"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "number");
    }
    
    #[test]
    fn test_performance_now_increases() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let t1 = ctx.eval(Source::from_bytes("performance.now()")).unwrap()
            .to_number(&mut ctx).unwrap();
        
        // Small delay
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let t2 = ctx.eval(Source::from_bytes("performance.now()")).unwrap()
            .to_number(&mut ctx).unwrap();
        
        assert!(t2 > t1);
    }
    
    #[test]
    fn test_performance_time_origin() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "performance.timeOrigin"
        )).unwrap();
        
        let time_origin = result.to_number(&mut ctx).unwrap();
        // Should be a reasonable Unix timestamp in milliseconds
        assert!(time_origin > 1_600_000_000_000.0); // After 2020
    }
    
    #[test]
    fn test_performance_mark() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        // Clear any existing marks first
        PERFORMANCE_ENTRIES.with(|e| e.borrow_mut().clear());
        
        let result = ctx.eval(Source::from_bytes(
            "performance.mark('test-mark')"
        )).unwrap();
        
        let obj = result.as_object().unwrap();
        let name = obj.get(JsString::from("name"), &mut ctx).unwrap()
            .to_string(&mut ctx).unwrap().to_std_string_escaped();
        
        assert_eq!(name, "test-mark");
    }
    
    #[test]
    fn test_performance_measure() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        PERFORMANCE_ENTRIES.with(|e| e.borrow_mut().clear());
        
        ctx.eval(Source::from_bytes("performance.mark('start')")).unwrap();
        ctx.eval(Source::from_bytes("performance.mark('end')")).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "performance.measure('test-measure', 'start', 'end')"
        )).unwrap();
        
        let obj = result.as_object().unwrap();
        let name = obj.get(JsString::from("name"), &mut ctx).unwrap()
            .to_string(&mut ctx).unwrap().to_std_string_escaped();
        
        assert_eq!(name, "test-measure");
    }
    
    #[test]
    fn test_performance_get_entries() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        PERFORMANCE_ENTRIES.with(|e| e.borrow_mut().clear());
        
        ctx.eval(Source::from_bytes("performance.mark('mark1')")).unwrap();
        ctx.eval(Source::from_bytes("performance.mark('mark2')")).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "performance.getEntries().length"
        )).unwrap();
        
        assert_eq!(result.to_number(&mut ctx).unwrap(), 2.0);
    }
    
    #[test]
    fn test_performance_observer_constructor() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof PerformanceObserver"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
}
