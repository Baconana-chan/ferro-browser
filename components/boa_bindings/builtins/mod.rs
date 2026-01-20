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
//! This module provides Ferro-specific extensions.

pub mod console;
pub mod events;

use boa_engine::{Context, JsResult};

/// Register Ferro-specific builtins on a context.
pub fn register_builtins(context: &mut Context) -> JsResult<()> {
    // boa_runtime already provides console, setTimeout, fetch, etc.
    // We only register Ferro-specific extensions here

    console::register(context)?;
    events::register(context)?;

    log::info!("Registered Ferro-specific Boa builtins");
    Ok(())
}
