// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Storage API implementation for Boa.
//!
//! Provides Web Storage API (localStorage and sessionStorage):
//! - getItem(key): Get item by key
//! - setItem(key, value): Set item
//! - removeItem(key): Remove item by key
//! - clear(): Clear all items
//! - key(index): Get key by index
//! - length: Number of items

use std::cell::RefCell;
use std::collections::HashMap;

use boa_engine::{
    Context, JsArgs, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor,
};

// Thread-local storage for localStorage and sessionStorage
thread_local! {
    static LOCAL_STORAGE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static SESSION_STORAGE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Storage type enum
#[derive(Clone, Copy)]
enum StorageType {
    Local,
    Session,
}

/// Register Storage APIs on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    // Create localStorage
    let local_storage = create_storage_object(StorageType::Local, context)?;
    context.global_object().define_property_or_throw(
        JsString::from("localStorage"),
        PropertyDescriptor::builder()
            .value(local_storage)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;

    // Create sessionStorage
    let session_storage = create_storage_object(StorageType::Session, context)?;
    context.global_object().define_property_or_throw(
        JsString::from("sessionStorage"),
        PropertyDescriptor::builder()
            .value(session_storage)
            .writable(false)
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;

    Ok(())
}

/// Create a Storage object with all methods
fn create_storage_object(storage_type: StorageType, context: &mut Context) -> JsResult<JsObject> {
    let storage = JsObject::with_null_proto();

    // getItem
    let get_item = match storage_type {
        StorageType::Local => create_get_item_local(),
        StorageType::Session => create_get_item_session(),
    };
    storage.define_property_or_throw(
        JsString::from("getItem"),
        PropertyDescriptor::builder()
            .value(get_item.to_js_function(context.realm()))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;

    // setItem
    let set_item = match storage_type {
        StorageType::Local => create_set_item_local(),
        StorageType::Session => create_set_item_session(),
    };
    storage.define_property_or_throw(
        JsString::from("setItem"),
        PropertyDescriptor::builder()
            .value(set_item.to_js_function(context.realm()))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;

    // removeItem
    let remove_item = match storage_type {
        StorageType::Local => create_remove_item_local(),
        StorageType::Session => create_remove_item_session(),
    };
    storage.define_property_or_throw(
        JsString::from("removeItem"),
        PropertyDescriptor::builder()
            .value(remove_item.to_js_function(context.realm()))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;

    // clear
    let clear = match storage_type {
        StorageType::Local => create_clear_local(),
        StorageType::Session => create_clear_session(),
    };
    storage.define_property_or_throw(
        JsString::from("clear"),
        PropertyDescriptor::builder()
            .value(clear.to_js_function(context.realm()))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;

    // key
    let key = match storage_type {
        StorageType::Local => create_key_local(),
        StorageType::Session => create_key_session(),
    };
    storage.define_property_or_throw(
        JsString::from("key"),
        PropertyDescriptor::builder()
            .value(key.to_js_function(context.realm()))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;

    // length property (getter)
    let length_getter = match storage_type {
        StorageType::Local => create_length_local(),
        StorageType::Session => create_length_session(),
    };
    storage.define_property_or_throw(
        JsString::from("length"),
        PropertyDescriptor::builder()
            .get(length_getter.to_js_function(context.realm()))
            .enumerable(true)
            .configurable(false)
            .build(),
        context,
    )?;

    Ok(storage)
}

// ============================================================================
// localStorage functions
// ============================================================================

fn create_get_item_local() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, args, context| {
        let key = args.get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        
        let result = LOCAL_STORAGE.with(|storage| {
            storage.borrow().get(&key).cloned()
        });
        
        Ok(result.map(|s| JsValue::from(JsString::from(s))).unwrap_or(JsValue::null()))
    })
}

fn create_set_item_local() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, args, context| {
        let key = args.get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        let value = args.get_or_undefined(1)
            .to_string(context)?
            .to_std_string_escaped();
        
        LOCAL_STORAGE.with(|storage| {
            storage.borrow_mut().insert(key, value);
        });
        
        Ok(JsValue::undefined())
    })
}

fn create_remove_item_local() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, args, context| {
        let key = args.get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        
        LOCAL_STORAGE.with(|storage| {
            storage.borrow_mut().remove(&key);
        });
        
        Ok(JsValue::undefined())
    })
}

fn create_clear_local() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, _args, _context| {
        LOCAL_STORAGE.with(|storage| {
            storage.borrow_mut().clear();
        });
        
        Ok(JsValue::undefined())
    })
}

fn create_key_local() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, args, context| {
        let index = args.get_or_undefined(0)
            .to_number(context)? as usize;
        
        let result = LOCAL_STORAGE.with(|storage| {
            storage.borrow()
                .keys()
                .nth(index)
                .cloned()
        });
        
        Ok(result.map(|s| JsValue::from(JsString::from(s))).unwrap_or(JsValue::null()))
    })
}

fn create_length_local() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, _args, _context| {
        let len = LOCAL_STORAGE.with(|storage| {
            storage.borrow().len()
        });
        
        Ok(JsValue::from(len as f64))
    })
}

// ============================================================================
// sessionStorage functions
// ============================================================================

fn create_get_item_session() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, args, context| {
        let key = args.get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        
        let result = SESSION_STORAGE.with(|storage| {
            storage.borrow().get(&key).cloned()
        });
        
        Ok(result.map(|s| JsValue::from(JsString::from(s))).unwrap_or(JsValue::null()))
    })
}

fn create_set_item_session() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, args, context| {
        let key = args.get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        let value = args.get_or_undefined(1)
            .to_string(context)?
            .to_std_string_escaped();
        
        SESSION_STORAGE.with(|storage| {
            storage.borrow_mut().insert(key, value);
        });
        
        Ok(JsValue::undefined())
    })
}

fn create_remove_item_session() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, args, context| {
        let key = args.get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        
        SESSION_STORAGE.with(|storage| {
            storage.borrow_mut().remove(&key);
        });
        
        Ok(JsValue::undefined())
    })
}

fn create_clear_session() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, _args, _context| {
        SESSION_STORAGE.with(|storage| {
            storage.borrow_mut().clear();
        });
        
        Ok(JsValue::undefined())
    })
}

fn create_key_session() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, args, context| {
        let index = args.get_or_undefined(0)
            .to_number(context)? as usize;
        
        let result = SESSION_STORAGE.with(|storage| {
            storage.borrow()
                .keys()
                .nth(index)
                .cloned()
        });
        
        Ok(result.map(|s| JsValue::from(JsString::from(s))).unwrap_or(JsValue::null()))
    })
}

fn create_length_session() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_this, _args, _context| {
        let len = SESSION_STORAGE.with(|storage| {
            storage.borrow().len()
        });
        
        Ok(JsValue::from(len as f64))
    })
}

// ============================================================================
// Utility functions for tests
// ============================================================================

/// Clear all storage (for testing)
pub fn clear_all_storage() {
    LOCAL_STORAGE.with(|storage| storage.borrow_mut().clear());
    SESSION_STORAGE.with(|storage| storage.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_storage_basic() {
        clear_all_storage();
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        // Test setItem and getItem
        context.eval(boa_engine::Source::from_bytes(
            r#"localStorage.setItem('test', 'value')"#
        )).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"localStorage.getItem('test')"#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "value");
    }

    #[test]
    fn test_session_storage_basic() {
        clear_all_storage();
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        context.eval(boa_engine::Source::from_bytes(
            r#"sessionStorage.setItem('session_test', 'session_value')"#
        )).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"sessionStorage.getItem('session_test')"#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "session_value");
    }

    #[test]
    fn test_storage_get_missing_key() {
        clear_all_storage();
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"localStorage.getItem('nonexistent')"#
        )).unwrap();
        
        assert!(result.is_null());
    }

    #[test]
    fn test_storage_remove_item() {
        clear_all_storage();
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        context.eval(boa_engine::Source::from_bytes(
            r#"
            localStorage.setItem('toRemove', 'value');
            localStorage.removeItem('toRemove');
            "#
        )).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"localStorage.getItem('toRemove')"#
        )).unwrap();
        
        assert!(result.is_null());
    }

    #[test]
    fn test_storage_clear() {
        clear_all_storage();
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        context.eval(boa_engine::Source::from_bytes(
            r#"
            localStorage.setItem('key1', 'value1');
            localStorage.setItem('key2', 'value2');
            localStorage.clear();
            "#
        )).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"localStorage.length"#
        )).unwrap();
        
        assert_eq!(result.as_number().unwrap(), 0.0);
    }

    #[test]
    fn test_storage_length() {
        clear_all_storage();
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        context.eval(boa_engine::Source::from_bytes(
            r#"
            localStorage.setItem('a', '1');
            localStorage.setItem('b', '2');
            localStorage.setItem('c', '3');
            "#
        )).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"localStorage.length"#
        )).unwrap();
        
        assert_eq!(result.as_number().unwrap(), 3.0);
    }
}
