// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Event-related builtins for Boa JavaScript engine.
//!
//! This module provides the Event and CustomEvent classes
//! fundamental to DOM event handling.

use boa_engine::{Context, JsResult, JsValue};

/// Event phase constants as defined in the DOM spec.
pub mod event_phase {
    pub const NONE: u16 = 0;
    pub const CAPTURING_PHASE: u16 = 1;
    pub const AT_TARGET: u16 = 2;
    pub const BUBBLING_PHASE: u16 = 3;
}

/// Internal data for an Event object.
#[derive(Debug, Clone)]
pub struct EventData {
    pub event_type: String,
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub timestamp: f64,
    pub propagation_stopped: bool,
    pub immediate_propagation_stopped: bool,
    pub default_prevented: bool,
    pub event_phase: u16,
    pub is_trusted: bool,
}

impl EventData {
    /// Create a new Event with the given type.
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            bubbles: false,
            cancelable: false,
            composed: false,
            timestamp: 0.0,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            default_prevented: false,
            event_phase: event_phase::NONE,
            is_trusted: false,
        }
    }

    /// Create from an event init dictionary.
    pub fn from_init(event_type: impl Into<String>, init: EventInit) -> Self {
        Self {
            event_type: event_type.into(),
            bubbles: init.bubbles,
            cancelable: init.cancelable,
            composed: init.composed,
            timestamp: 0.0,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            default_prevented: false,
            event_phase: event_phase::NONE,
            is_trusted: false,
        }
    }
}

/// Options for creating an Event.
#[derive(Debug, Default, Clone)]
pub struct EventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
}

/// Internal data for a CustomEvent.
#[derive(Debug, Clone)]
pub struct CustomEventData {
    pub event: EventData,
    pub detail: Option<JsValue>,
}

impl CustomEventData {
    /// Create a new CustomEvent.
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event: EventData::new(event_type),
            detail: None,
        }
    }
}

/// Register all event-related builtins.
pub fn register(_context: &mut Context) -> JsResult<()> {
    // TODO: Implement full Event and CustomEvent classes
    // For now, these are placeholders for future DOM integration
    log::debug!("Event builtins placeholder registered");
    Ok(())
}
