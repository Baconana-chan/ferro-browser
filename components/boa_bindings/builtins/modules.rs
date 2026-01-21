// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! ES Modules support for Boa.
//!
//! Provides:
//! - import.meta: Module metadata object
//! - Dynamic import(): Load modules dynamically
//! - Module registry for caching loaded modules
//!
//! Note: Full ES Modules requires integration with the module loader.
//! This provides the API surface and basic infrastructure.

use boa_engine::{
    Context, JsArgs, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor,
    object::builtins::JsPromise,
};
use std::cell::RefCell;
use std::collections::HashMap;

// Thread-local module registry
thread_local! {
    static MODULE_REGISTRY: RefCell<HashMap<String, JsObject>> = RefCell::new(HashMap::new());
    static MODULE_BASE_URL: RefCell<String> = RefCell::new(String::from("file:///"));
}

/// Register ES Modules support on the context.
pub fn register(context: &mut Context) -> JsResult<()> {
    // Register dynamic import function
    register_dynamic_import(context)?;
    
    // Register import.meta object
    register_import_meta(context)?;
    
    Ok(())
}

/// Set the base URL for module resolution.
pub fn set_base_url(url: &str) {
    MODULE_BASE_URL.with(|base| {
        *base.borrow_mut() = url.to_string();
    });
}

/// Get the base URL for module resolution.
pub fn get_base_url() -> String {
    MODULE_BASE_URL.with(|base| base.borrow().clone())
}

// ============================================================================
// Dynamic import()
// ============================================================================

fn register_dynamic_import(context: &mut Context) -> JsResult<()> {
    // Note: In Boa, dynamic import is handled internally,
    // but we can provide a polyfill for environments without native support
    let import_func = NativeFunction::from_fn_ptr(dynamic_import);
    
    context.global_object().define_property_or_throw(
        JsString::from("__dynamicImport"),
        PropertyDescriptor::builder()
            .value(import_func.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn dynamic_import(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let specifier = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    log::debug!("[ESModules] Dynamic import requested: {}", specifier);
    
    // Check if module is already in registry
    let cached = MODULE_REGISTRY.with(|registry| {
        registry.borrow().get(&specifier).cloned()
    });
    
    if let Some(module_ns) = cached {
        // Return cached module as resolved promise
        let promise = JsPromise::resolve(JsValue::from(module_ns), context);
        return Ok(promise.into());
    }
    
    // For now, return a promise that rejects with "Module not found"
    // In a full implementation, this would trigger the module loader
    let (promise, resolvers) = JsPromise::new_pending(context);
    
    // Create error object
    let error_msg = format!("Module '{}' not found. Module loading requires integration with the resource loader.", specifier);
    let error = JsValue::from(JsString::from(error_msg.as_str()));
    
    // Reject the promise
    resolvers.reject.call(&JsValue::undefined(), &[error], context)?;
    
    Ok(promise.into())
}

// ============================================================================
// import.meta
// ============================================================================

fn register_import_meta(context: &mut Context) -> JsResult<()> {
    // Create import.meta object
    let import_meta = create_import_meta_object(context)?;
    
    // Register as __importMeta (actual import.meta is syntax-level)
    context.global_object().define_property_or_throw(
        JsString::from("__importMeta"),
        PropertyDescriptor::builder()
            .value(import_meta)
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn create_import_meta_object(context: &mut Context) -> JsResult<JsValue> {
    let meta = JsObject::with_null_proto();
    
    // import.meta.url - The URL of the current module
    let base_url = get_base_url();
    meta.define_property_or_throw(
        JsString::from("url"),
        PropertyDescriptor::builder()
            .value(JsString::from(base_url.as_str()))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // import.meta.resolve(specifier) - Resolve a module specifier
    let resolve_fn = NativeFunction::from_fn_ptr(import_meta_resolve);
    meta.define_property_or_throw(
        JsString::from("resolve"),
        PropertyDescriptor::builder()
            .value(resolve_fn.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // import.meta.dirname - Directory of the current module (Node.js compat)
    meta.define_property_or_throw(
        JsString::from("dirname"),
        PropertyDescriptor::builder()
            .value(JsString::from("/"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // import.meta.filename - Filename of the current module (Node.js compat)
    meta.define_property_or_throw(
        JsString::from("filename"),
        PropertyDescriptor::builder()
            .value(JsString::from("module.js"))
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(meta))
}

fn import_meta_resolve(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let specifier = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    let base_url = get_base_url();
    
    // Simple URL resolution
    let resolved = if specifier.starts_with("./") || specifier.starts_with("../") {
        // Relative URL
        format!("{}{}", base_url, specifier.trim_start_matches("./"))
    } else if specifier.starts_with('/') {
        // Absolute path
        format!("file://{}", specifier)
    } else if specifier.starts_with("http://") || specifier.starts_with("https://") {
        // Already absolute URL
        specifier
    } else {
        // Bare specifier (package name)
        format!("{}node_modules/{}/index.js", base_url, specifier)
    };
    
    Ok(JsValue::from(JsString::from(resolved.as_str())))
}

// ============================================================================
// Module Registry Management
// ============================================================================

/// Register a module in the registry.
pub fn register_module(specifier: &str, namespace: JsObject) {
    MODULE_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(specifier.to_string(), namespace);
    });
}

/// Check if a module is registered.
pub fn is_module_registered(specifier: &str) -> bool {
    MODULE_REGISTRY.with(|registry| {
        registry.borrow().contains_key(specifier)
    })
}

/// Clear the module registry.
pub fn clear_module_registry() {
    MODULE_REGISTRY.with(|registry| {
        registry.borrow_mut().clear();
    });
}

/// Get all registered module specifiers.
pub fn get_registered_modules() -> Vec<String> {
    MODULE_REGISTRY.with(|registry| {
        registry.borrow().keys().cloned().collect()
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;
    
    #[test]
    fn test_modules_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof __dynamicImport"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_import_meta_exists() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof __importMeta"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "object");
    }
    
    #[test]
    fn test_import_meta_url() {
        let mut ctx = Context::default();
        set_base_url("https://example.com/scripts/");
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "__importMeta.url"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "https://example.com/scripts/");
    }
    
    #[test]
    fn test_import_meta_resolve() {
        let mut ctx = Context::default();
        set_base_url("https://example.com/scripts/");
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "__importMeta.resolve('./utils.js')"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "https://example.com/scripts/utils.js");
    }
    
    #[test]
    fn test_import_meta_resolve_absolute() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "__importMeta.resolve('https://cdn.example.com/lib.js')"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "https://cdn.example.com/lib.js");
    }
    
    #[test]
    fn test_dynamic_import_returns_promise() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "__dynamicImport('./module.js') instanceof Promise"
        )).unwrap();
        
        assert_eq!(result.to_boolean(), true);
    }
    
    #[test]
    fn test_module_registry() {
        clear_module_registry();
        
        assert!(!is_module_registered("test-module"));
        
        let mut ctx = Context::default();
        let obj = JsObject::with_null_proto();
        register_module("test-module", obj);
        
        assert!(is_module_registered("test-module"));
        
        let modules = get_registered_modules();
        assert!(modules.contains(&"test-module".to_string()));
        
        clear_module_registry();
        assert!(!is_module_registered("test-module"));
    }
    
    #[test]
    fn test_base_url() {
        set_base_url("https://ferro-browser.dev/");
        assert_eq!(get_base_url(), "https://ferro-browser.dev/");
        
        set_base_url("file:///home/user/");
        assert_eq!(get_base_url(), "file:///home/user/");
    }
}
