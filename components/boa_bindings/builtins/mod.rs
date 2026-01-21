// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Built-in Web API implementations for Boa.
//!
//! Provides complete Web API implementations:
//! - Console API (console.log, console.error, console.group, console.time, etc.)
//! - Timer API (setTimeout, setInterval, clearTimeout, clearInterval)
//! - Fetch API (fetch, Request, Response, Headers)
//! - Storage API (localStorage, sessionStorage)
//! - URL API (URL, URLSearchParams)
//! - Encoding API (TextEncoder, TextDecoder)
//! - Event system (addEventListener, dispatchEvent, removeEventListener)
//!
//! This module provides implementations that integrate with Servo's architecture.

pub mod console;
pub mod crypto;
pub mod encoding;
pub mod events;
pub mod fetch;
pub mod media_source;
pub mod modules;
pub mod navigator;
pub mod observers;
pub mod performance;
pub mod promises;
pub mod storage;
pub mod timers;
pub mod url;
pub mod web_audio;
pub mod webgl;
pub mod workers;

use boa_engine::{Context, JsResult};

/// Register all builtins on a context.
pub fn register_builtins(context: &mut Context) -> JsResult<()> {
    // Register Console API with Ferro extensions
    console::register(context)?;
    
    // Register event system
    events::register(context)?;
    
    // Register timer functions (setTimeout, setInterval, etc.)
    timers::register(context)?;
    
    // Register Storage APIs (localStorage, sessionStorage)
    storage::register(context)?;
    
    // Register URL APIs (URL, URLSearchParams)
    url::register(context)?;
    
    // Register Encoding APIs (TextEncoder, TextDecoder)
    encoding::register(context)?;
    
    // Register Fetch APIs (fetch, Request, Response, Headers)
    fetch::register(context)?;
    
    // Register Promise utilities (queueMicrotask, Promise.withResolvers)
    promises::register(context)?;
    
    // Register Crypto API (crypto.getRandomValues, randomUUID)
    crypto::register(context)?;
    
    // Register Performance API (performance.now, mark, measure)
    performance::register(context)?;
    
    // Register Workers API stubs (Worker, SharedWorker, MessageChannel)
    workers::register(context)?;
    
    // Register ES Modules support (import.meta, dynamic import)
    modules::register(context)?;
    
    // Register Observer APIs (IntersectionObserver, ResizeObserver, MutationObserver)
    observers::register(context)?;
    
    // Register Navigator API (navigator.*)
    navigator::register(context)?;

    // Register WebGL APIs (WebGLRenderingContext, WebGL2RenderingContext)
    webgl::register(context)?;

    // Register MediaSource Extensions API (MediaSource, SourceBuffer)
    media_source::register(context)?;

    // Register Web Audio API (AudioContext, GainNode, OscillatorNode, etc.)
    web_audio::register(context)?;

    log::info!("Registered Boa builtins (Console, Timers, Storage, URL, Encoding, Fetch, Promises, Crypto, Performance, Workers, Modules, Observers, Navigator, WebGL, MediaSource, WebAudio)");
    Ok(())
}
