//! EventTarget Interface Binding for Boa
//!
//! The EventTarget interface is the base interface for all DOM objects
//! that can receive events and have listeners for them.

use std::cell::RefCell;
use std::collections::HashMap;

use boa_engine::{
    Context, JsArgs, JsObject, JsResult, JsValue, js_string,
};

use crate::codegen::types::{FromJsValueBoa, ToJsValueBoa};
use crate::codegen::traits::{InterfaceBuilder};
use crate::codegen::interface::check_args_length;

/// Event listener callback type
pub type EventCallback = JsObject;

/// Event listener entry
#[derive(Clone)]
pub struct EventListener {
    pub callback: EventCallback,
    pub capture: bool,
    pub passive: bool,
    pub once: bool,
}

/// EventTarget - base interface for DOM event handling
#[derive(Default)]
pub struct EventTarget {
    /// Map of event type to list of listeners
    listeners: RefCell<HashMap<String, Vec<EventListener>>>,
}

impl EventTarget {
    /// Create a new EventTarget
    pub fn new() -> Self {
        EventTarget {
            listeners: RefCell::new(HashMap::new()),
        }
    }

    /// Add an event listener
    pub fn add_event_listener(
        &self,
        event_type: &str,
        callback: EventCallback,
        options: ListenerOptions,
    ) {
        let listener = EventListener {
            callback,
            capture: options.capture,
            passive: options.passive,
            once: options.once,
        };

        let mut listeners = self.listeners.borrow_mut();
        listeners
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(listener);
    }

    /// Remove an event listener
    pub fn remove_event_listener(
        &self,
        event_type: &str,
        _callback: &EventCallback,
        _capture: bool,
    ) {
        let mut listeners = self.listeners.borrow_mut();
        if let Some(list) = listeners.get_mut(event_type) {
            // Note: In a full implementation, we would compare callbacks
            list.clear();
        }
    }

    /// Dispatch an event to all listeners
    pub fn dispatch_event(&self, event: &Event, ctx: &mut Context) -> JsResult<bool> {
        let listeners = self.listeners.borrow();
        
        if let Some(list) = listeners.get(&event.event_type) {
            for listener in list.iter() {
                let this = JsValue::undefined();
                let event_value = event.to_js_value(ctx)?;
                
                listener.callback.call(&this, &[event_value], ctx)?;
            }
        }
        
        Ok(!event.default_prevented)
    }

    /// Check if there are listeners for an event type
    pub fn has_event_listeners(&self, event_type: &str) -> bool {
        let listeners = self.listeners.borrow();
        listeners.get(event_type).is_some_and(|l| !l.is_empty())
    }
}

/// Options for addEventListener
#[derive(Default, Clone)]
pub struct ListenerOptions {
    pub capture: bool,
    pub passive: bool,
    pub once: bool,
}

impl FromJsValueBoa for ListenerOptions {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        if value.is_boolean() {
            Ok(ListenerOptions {
                capture: value.to_boolean(),
                passive: false,
                once: false,
            })
        } else if value.is_object() {
            let obj = value.to_object(ctx)?;
            let capture = obj.get(js_string!("capture"), ctx)?.to_boolean();
            let passive = obj.get(js_string!("passive"), ctx)?.to_boolean();
            let once = obj.get(js_string!("once"), ctx)?.to_boolean();

            Ok(ListenerOptions { capture, passive, once })
        } else {
            Ok(ListenerOptions::default())
        }
    }
}

/// Simplified Event structure
#[derive(Default, Clone)]
pub struct Event {
    pub event_type: String,
    pub bubbles: bool,
    pub cancelable: bool,
    pub default_prevented: bool,
    pub is_trusted: bool,
    pub timestamp: f64,
}

impl Event {
    pub fn new(event_type: &str) -> Self {
        Event {
            event_type: event_type.to_string(),
            bubbles: false,
            cancelable: false,
            default_prevented: false,
            is_trusted: false,
            timestamp: 0.0,
        }
    }

    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.default_prevented = true;
        }
    }
}

impl ToJsValueBoa for Event {
    fn to_js_value(&self, ctx: &mut Context) -> JsResult<JsValue> {
        let obj = JsObject::with_null_proto();
        
        obj.set(js_string!("type"), JsValue::from(js_string!(self.event_type.as_str())), false, ctx)?;
        obj.set(js_string!("bubbles"), JsValue::from(self.bubbles), false, ctx)?;
        obj.set(js_string!("cancelable"), JsValue::from(self.cancelable), false, ctx)?;
        obj.set(js_string!("defaultPrevented"), JsValue::from(self.default_prevented), false, ctx)?;
        obj.set(js_string!("isTrusted"), JsValue::from(self.is_trusted), false, ctx)?;
        obj.set(js_string!("timeStamp"), JsValue::from(self.timestamp), false, ctx)?;

        Ok(obj.into())
    }
}

// ============================================================================
// JavaScript Binding Functions
// ============================================================================

fn event_target_constructor(
    _this: &JsValue,
    _args: &[JsValue],
    _ctx: &mut Context,
) -> JsResult<JsValue> {
    // Return an empty object for now
    Ok(JsObject::with_null_proto().into())
}

fn add_event_listener_fn(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {
    check_args_length(args, 2, "addEventListener")?;
    // Stub implementation
    let _event_type: String = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
    Ok(JsValue::undefined())
}

fn remove_event_listener_fn(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {
    check_args_length(args, 2, "removeEventListener")?;
    let _event_type: String = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
    Ok(JsValue::undefined())
}

fn dispatch_event_fn(
    _this: &JsValue,
    args: &[JsValue],
    _ctx: &mut Context,
) -> JsResult<JsValue> {
    check_args_length(args, 1, "dispatchEvent")?;
    Ok(JsValue::from(true))
}

/// Register the EventTarget interface on the global object
pub fn register_event_target(ctx: &mut Context) -> JsResult<JsObject> {
    InterfaceBuilder::new(ctx, "EventTarget")
        .constructor(event_target_constructor, 0)
        .method("addEventListener", add_event_listener_fn, 2)
        .method("removeEventListener", remove_event_listener_fn, 2)
        .method("dispatchEvent", dispatch_event_fn, 1)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_target_new() {
        let event_target = EventTarget::new();
        assert!(!event_target.has_event_listeners("click"));
    }

    #[test]
    fn test_listener_options_default() {
        let options = ListenerOptions::default();
        assert!(!options.capture);
        assert!(!options.passive);
        assert!(!options.once);
    }

    #[test]
    fn test_event_new() {
        let event = Event::new("click");
        assert_eq!(event.event_type, "click");
        assert!(!event.bubbles);
        assert!(!event.cancelable);
    }

    #[test]
    fn test_register_event_target() {
        let mut ctx = Context::default();
        register_event_target(&mut ctx).unwrap();
        
        let global = ctx.global_object();
        let ctor = global.get(js_string!("EventTarget"), &mut ctx).unwrap();
        assert!(ctor.is_object());
    }
}
