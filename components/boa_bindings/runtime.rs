// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! JavaScript Runtime management for Boa 0.21.
//!
//! This module provides the main entry point for JavaScript execution.

use boa_engine::{Context, JsResult, JsValue, Source};

/// A JavaScript runtime environment wrapping Boa Context.
pub struct JsRuntime {
    context: Context,
}

impl JsRuntime {
    /// Create a new JavaScript runtime with default settings.
    pub fn new() -> Self {
        let context = Context::default();
        log::info!("Created new Boa JsRuntime (v0.21)");
        Self { context }
    }

    /// Evaluate a JavaScript source string.
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut runtime = JsRuntime::new();
    /// let result = runtime.eval("2 + 2").unwrap();
    /// assert_eq!(result.as_number(), Some(4.0));
    /// ```
    pub fn eval(&mut self, source: &str) -> JsResult<JsValue> {
        let source = Source::from_bytes(source.as_bytes());
        self.context.eval(source)
    }

    /// Evaluate JavaScript with a filename for error messages.
    pub fn eval_with_filename(&mut self, source: &str, filename: &str) -> JsResult<JsValue> {
        let source = Source::from_bytes(source.as_bytes())
            .with_path(std::path::Path::new(filename));
        self.context.eval(source)
    }

    /// Get a reference to the underlying Boa context.
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Get a mutable reference to the underlying Boa context.
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    /// Register a global variable.
    pub fn set_global(&mut self, name: &str, value: JsValue) -> JsResult<()> {
        self.context.register_global_property(
            boa_engine::JsString::from(name),
            value,
            boa_engine::property::Attribute::WRITABLE |
            boa_engine::property::Attribute::ENUMERABLE |
            boa_engine::property::Attribute::CONFIGURABLE,
        )
    }

    /// Get a global variable's value.
    pub fn get_global(&mut self, name: &str) -> JsResult<JsValue> {
        let global = self.context.global_object();
        global.get(boa_engine::JsString::from(name), &mut self.context)
    }

    /// Run garbage collection.
    pub fn gc(&mut self) {
        boa_gc::force_collect();
    }
}

impl Default for JsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for JsRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsRuntime")
            .field("context", &"<Boa Context>")
            .finish()
    }
}

/// A handle to a JavaScript realm (global environment).
/// Each Window/Worker has its own realm.
pub struct JsRealm {
    _private: (),
}

impl JsRealm {
    /// Create a new realm within an existing runtime.
    pub fn new(_runtime: &mut JsRuntime) -> Self {
        // TODO: Implement proper realm creation for multi-window
        Self { _private: () }
    }
}
