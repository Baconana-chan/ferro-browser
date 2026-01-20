// MIT License
//
// Copyright (c) 2025-2026 Ferro Browser Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Boa JavaScript Engine bindings for Ferro Browser.
//!
//! This module provides a pure Rust alternative to SpiderMonkey-based
//! script_bindings. Boa is a JavaScript engine written entirely in Rust,
//! offering better debugging, faster builds, and seamless integration.
//!
//! ## Features
//!
//! - **Pure Rust**: No C++ dependencies, no mozjs build complexity
//! - **Boa 0.21**: 94% ECMAScript conformance, NaN-boxing, register-based VM
//! - **Web APIs**: setTimeout, fetch, console via boa_runtime
//!
//! ## Architecture
//!
//! - `runtime` - JavaScript runtime management (Context)
//! - `gc` - Garbage collection integration with DOM
//! - `conversions` - Type conversions between Rust and JavaScript
//! - `error` - JavaScript error handling
//! - `builtins` - Servo-specific Web API extensions

pub mod runtime;
pub mod gc;
pub mod conversions;
pub mod error;
pub mod builtins;

// Re-export key Boa types
pub use boa_engine::{
    Context, JsResult, JsValue, JsError, JsNativeError,
    Source, JsString,
    object::JsObject,
    property::Attribute,
};

pub use boa_gc::{Finalize, Trace, Gc, GcRefCell};

/// Initialize the Boa JavaScript engine.
pub fn init() {
    log::info!("Initializing Boa JavaScript engine v{}", version());
}

/// Get the Boa engine version string.
pub fn version() -> &'static str {
    "0.21.0"
}

#[cfg(test)]
mod tests {
    use super::runtime::JsRuntime;

    #[test]
    fn test_basic_eval() {
        let mut runtime = JsRuntime::new();
        let result = runtime.eval("1 + 1").unwrap();
        assert_eq!(result.as_number(), Some(2.0));
    }

    #[test]
    fn test_string_eval() {
        let mut runtime = JsRuntime::new();
        let result = runtime.eval("'hello' + ' ' + 'world'").unwrap();
        let s = result.as_string().unwrap();
        assert_eq!(s.to_std_string_escaped(), "hello world");
    }

    #[test]
    fn test_function_call() {
        let mut runtime = JsRuntime::new();
        runtime.eval("function add(a, b) { return a + b; }").unwrap();
        let result = runtime.eval("add(2, 3)").unwrap();
        assert_eq!(result.as_number(), Some(5.0));
    }
}
