// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Web Workers API stub for Boa.
//!
//! Provides stub implementations of:
//! - Worker: Dedicated web worker
//! - SharedWorker: Shared web worker
//! - MessageChannel / MessagePort: Inter-worker communication
//!
//! Note: Full worker support requires multi-threaded JS context management.
//! This provides the API surface for compatibility with sites that check for
//! Worker support.

use boa_engine::{
    Context, JsArgs, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor,
};

/// Register Workers API on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    // Register Worker constructor
    register_worker(context)?;
    
    // Register SharedWorker constructor
    register_shared_worker(context)?;
    
    // Register MessageChannel constructor
    register_message_channel(context)?;
    
    Ok(())
}

// ============================================================================
// Worker
// ============================================================================

fn register_worker(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(worker_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("Worker"),
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

fn worker_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    log::debug!("[Worker] Constructor called with URL: {} (stub)", url);
    
    let worker = JsObject::with_null_proto();
    
    // Store the URL
    worker.define_property_or_throw(
        JsString::from("__url"),
        PropertyDescriptor::builder()
            .value(JsString::from(url.as_str()))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // onmessage property (initially null)
    worker.define_property_or_throw(
        JsString::from("onmessage"),
        PropertyDescriptor::builder()
            .value(JsValue::null())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // onerror property (initially null)
    worker.define_property_or_throw(
        JsString::from("onerror"),
        PropertyDescriptor::builder()
            .value(JsValue::null())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // postMessage method (stub)
    let post_message = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let message = args.get_or_undefined(0);
        log::debug!("[Worker] postMessage called: {:?} (stub)", 
            message.to_string(ctx).map(|s| s.to_std_string_escaped()).unwrap_or_default());
        Ok(JsValue::undefined())
    });
    worker.define_property_or_throw(
        JsString::from("postMessage"),
        PropertyDescriptor::builder()
            .value(post_message.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // terminate method (stub)
    let terminate = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[Worker] terminate called (stub)");
        Ok(JsValue::undefined())
    });
    worker.define_property_or_throw(
        JsString::from("terminate"),
        PropertyDescriptor::builder()
            .value(terminate.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // addEventListener (stub - for event listener interface)
    let add_event_listener = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[Worker] addEventListener called (stub)");
        Ok(JsValue::undefined())
    });
    worker.define_property_or_throw(
        JsString::from("addEventListener"),
        PropertyDescriptor::builder()
            .value(add_event_listener.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // removeEventListener (stub)
    let remove_event_listener = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[Worker] removeEventListener called (stub)");
        Ok(JsValue::undefined())
    });
    worker.define_property_or_throw(
        JsString::from("removeEventListener"),
        PropertyDescriptor::builder()
            .value(remove_event_listener.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(worker))
}

// ============================================================================
// SharedWorker
// ============================================================================

fn register_shared_worker(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(shared_worker_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("SharedWorker"),
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

fn shared_worker_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    let name = if !args.get_or_undefined(1).is_undefined() {
        args.get_or_undefined(1).to_string(context)?.to_std_string_escaped()
    } else {
        String::new()
    };
    
    log::debug!("[SharedWorker] Constructor called with URL: {}, name: {} (stub)", url, name);
    
    let shared_worker = JsObject::with_null_proto();
    
    // port property (MessagePort stub)
    let port = create_message_port(context)?;
    shared_worker.define_property_or_throw(
        JsString::from("port"),
        PropertyDescriptor::builder()
            .value(port)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // onerror property
    shared_worker.define_property_or_throw(
        JsString::from("onerror"),
        PropertyDescriptor::builder()
            .value(JsValue::null())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(shared_worker))
}

// ============================================================================
// MessageChannel / MessagePort
// ============================================================================

fn register_message_channel(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(message_channel_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("MessageChannel"),
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

fn message_channel_constructor(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    log::debug!("[MessageChannel] Constructor called (stub)");
    
    let channel = JsObject::with_null_proto();
    
    // port1
    let port1 = create_message_port(context)?;
    channel.define_property_or_throw(
        JsString::from("port1"),
        PropertyDescriptor::builder()
            .value(port1)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // port2
    let port2 = create_message_port(context)?;
    channel.define_property_or_throw(
        JsString::from("port2"),
        PropertyDescriptor::builder()
            .value(port2)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(channel))
}

fn create_message_port(context: &mut Context) -> JsResult<JsValue> {
    let port = JsObject::with_null_proto();
    
    // onmessage
    port.define_property_or_throw(
        JsString::from("onmessage"),
        PropertyDescriptor::builder()
            .value(JsValue::null())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // onmessageerror
    port.define_property_or_throw(
        JsString::from("onmessageerror"),
        PropertyDescriptor::builder()
            .value(JsValue::null())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // postMessage (stub)
    let post_message = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[MessagePort] postMessage called (stub)");
        Ok(JsValue::undefined())
    });
    port.define_property_or_throw(
        JsString::from("postMessage"),
        PropertyDescriptor::builder()
            .value(post_message.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // start (stub)
    let start = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[MessagePort] start called (stub)");
        Ok(JsValue::undefined())
    });
    port.define_property_or_throw(
        JsString::from("start"),
        PropertyDescriptor::builder()
            .value(start.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // close (stub)
    let close = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[MessagePort] close called (stub)");
        Ok(JsValue::undefined())
    });
    port.define_property_or_throw(
        JsString::from("close"),
        PropertyDescriptor::builder()
            .value(close.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // addEventListener
    let add_event_listener = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[MessagePort] addEventListener called (stub)");
        Ok(JsValue::undefined())
    });
    port.define_property_or_throw(
        JsString::from("addEventListener"),
        PropertyDescriptor::builder()
            .value(add_event_listener.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // removeEventListener
    let remove_event_listener = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        log::debug!("[MessagePort] removeEventListener called (stub)");
        Ok(JsValue::undefined())
    });
    port.define_property_or_throw(
        JsString::from("removeEventListener"),
        PropertyDescriptor::builder()
            .value(remove_event_listener.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(port))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;
    
    #[test]
    fn test_worker_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof Worker"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_worker_call() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        // Call as function (not constructor) since NativeFunction doesn't support [[Construct]]
        let result = ctx.eval(Source::from_bytes(
            "Worker('test.js') && typeof Worker('test.js').postMessage"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_worker_has_methods() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            let w = Worker('test.js');
            typeof w.postMessage === 'function' && typeof w.terminate === 'function'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_shared_worker_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof SharedWorker"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_shared_worker_has_port() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            let sw = SharedWorker('test.js');
            typeof sw.port === 'object' && typeof sw.port.postMessage === 'function'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_message_channel_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof MessageChannel"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_message_channel_has_ports() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            let mc = MessageChannel();
            typeof mc.port1 === 'object' && typeof mc.port2 === 'object'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_message_port_methods() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(r#"
            let mc = MessageChannel();
            typeof mc.port1.postMessage === 'function' && 
            typeof mc.port1.start === 'function' &&
            typeof mc.port1.close === 'function'
        "#)).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
}
