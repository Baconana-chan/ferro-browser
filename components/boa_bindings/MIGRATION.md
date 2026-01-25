# Boa Engine Migration Guide

## Overview

Ferro Browser uses **Boa** as its default JavaScript engine instead of SpiderMonkey.
This guide documents the migration and provides guidance for developers.

## Why Boa?

| Aspect | SpiderMonkey | Boa |
|--------|--------------|-----|
| Language | C++ with Rust bindings | Pure Rust |
| Build time | 10-15 minutes | 1-2 minutes |
| Dependencies | clang, llvm, python, etc. | Just Rust |
| Platforms | Limited (x86_64, aarch64) | Any Rust target |
| ES Conformance | 100% | 94% |
| Debugging | gdb/lldb | rust-gdb, IDE native |

## Feature Flags

Select the JavaScript engine via Cargo features:

```toml
# Use Boa (default, pure Rust)
cargo build --features js-boa

# Use SpiderMonkey (if you need 100% ES conformance)
cargo build --features js-spidermonkey --no-default-features
```

## API Compatibility

boa_bindings provides compatibility shims for most SpiderMonkey APIs:

### Smart Pointers
- `Dom<T>` - Non-GC pointer to DOM object
- `DomRoot<T>` - Rooted DOM reference
- `Root<T>` - RAII root guard

### Traits
- `DomObject` - Base trait for DOM objects
- `Reflector` - JS wrapper management
- `Trace` / `Finalize` - GC integration

### Conversion
- `ToJSValConvertible` - Rust → JS
- `FromJSValConvertible` - JS → Rust
- `NativeFromObjectExt` - Extract DOM from JsObject

## Module Structure

```
boa_bindings/
├── runtime.rs       # JsRuntime (Context wrapper)
├── reflector.rs     # DOM reflection
├── root.rs          # Smart pointers
├── weakref.rs       # Weak references
├── gc_safety.rs     # GC debug assertions
├── settings_stack.rs # HTML settings stack
├── event_loop.rs    # Task/microtask queue
├── builtins/        # Web APIs
│   ├── console.rs
│   ├── fetch.rs     # + AbortController
│   ├── timers.rs
│   ├── storage.rs
│   ├── performance.rs
│   ├── crypto.rs
│   ├── url.rs
│   ├── webgl.rs
│   ├── web_audio.rs
│   └── ...
└── codegen/         # WebIDL bindings
    ├── types.rs
    ├── traits.rs
    └── prototype_list.rs
```

## Testing

Run all boa_bindings tests:
```bash
cd components/boa_bindings
cargo test
```

Current status: **267 tests passing**

## Migration Checklist

For new code using boa_bindings:

- [ ] Use `JsRuntime::new()` instead of `JSRuntime`
- [ ] Use `Dom<T>` / `DomRoot<T>` from `boa_bindings::root`
- [ ] Implement `DomObject` + `Trace` for DOM types
- [ ] Use `NativeFromObjectExt` for type-safe extraction
- [ ] Use `event_loop::schedule_task` for async operations

## Debugging

Enable GC debug mode:
```bash
GC_DEBUG=1 cargo test
```

This enables:
- Allocation tracking
- Finalization order verification
- Root depth assertions

## Known Limitations

1. **ES Modules**: Basic support, no dynamic `import()` yet
2. **WeakRef**: Uses custom implementation, not Boa's
3. **Stack traces**: Basic formatting, no source maps
4. **Workers**: Stub implementation (no true parallelism)

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for general guidelines.

For boa_bindings specifically:
1. Add tests for any new functionality
2. Document public APIs
3. Ensure SpiderMonkey compatibility where needed
