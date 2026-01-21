// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Built-in Web API implementations for Boa.
//!
//! Note: boa_runtime 0.21 already provides:
//! - Console API (console.log, console.error, etc.)
//! - Timer API (setTimeout, setInterval, clearTimeout, clearInterval)
//! - fetch API
//! - queueMicrotask
//!
//! This module provides Ferro-specific extensions and implementations
//! that integrate with Servo's architecture.

pub mod console;
pub mod events;
pub mod timers;

use boa_engine::{Context, JsResult};

/// Register Ferro-specific builtins on a context.
pub fn register_builtins(context: &mut Context) -> JsResult<()> {
    // Register Ferro-specific console extensions
    console::register(context)?;
    
    // Register event system placeholders
    events::register(context)?;
    
    // Register timer functions (setTimeout, setInterval, etc.)
    timers::register(context)?;

    log::info!("Registered Ferro-specific Boa builtins");
    Ok(())
}
