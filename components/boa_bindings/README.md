# boa_bindings

**Pure Rust JavaScript engine bindings for Ferro Browser**

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)

This module provides bindings for the [Boa JavaScript engine](https://github.com/boa-dev/boa), a pure Rust implementation of ECMAScript. It serves as a future replacement for SpiderMonkey (mozjs) in Ferro Browser.

## Why Boa?

| Feature | SpiderMonkey | Boa |
|---------|--------------|-----|
| Language | C++ with Rust FFI | Pure Rust |
| Build time | 10-15 minutes | ~2 minutes |
| ECMAScript compliance | ~100% | 94% (v0.21) |
| Debugging | Complex C++/Rust boundary | Native Rust |
| Memory model | Custom GC | boa_gc (Rust) |
| Dependencies | Clang, LLVM, libmozjs | None (pure Rust) |

## Features (Boa 0.21)

- **94.12% ECMAScript conformance** (Test262)
- NaN-boxing for efficient JsValue representation
- Register-based VM for faster execution
- Built-in Web APIs via `boa_runtime`:
  - `console.log/error/warn`
  - `setTimeout/setInterval`
  - `fetch`
  - `queueMicrotask`

## Module Structure

```
boa_bindings/
├── lib.rs          # Main module, re-exports
├── runtime.rs      # JsRuntime wrapper for Context
├── gc.rs           # GC integration (DomRef, DomCell)
├── conversions.rs  # Rust ↔ JavaScript type conversions
├── error.rs        # Error handling utilities
└── builtins/       # Ferro-specific Web API extensions
    ├── mod.rs
    ├── console.rs  # Console extensions
    └── events.rs   # Event/CustomEvent classes
```

## Usage

```rust
use boa_bindings::runtime::JsRuntime;

fn main() {
    let mut runtime = JsRuntime::new();
    
    // Evaluate JavaScript
    let result = runtime.eval("1 + 1").unwrap();
    assert_eq!(result.as_number(), Some(2.0));
    
    // Set global variables
    runtime.set_global("greeting", "Hello, Ferro!".into()).unwrap();
    
    // Call functions
    runtime.eval("function add(a, b) { return a + b; }").unwrap();
    let sum = runtime.eval("add(2, 3)").unwrap();
    assert_eq!(sum.as_number(), Some(5.0));
}
```

## License

MIT License - Copyright (c) 2025-2026 Ferro Browser Contributors

This module is independently developed and licensed under MIT.
Files derived from Servo remain under MPL-2.0.
