// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! URL API implementation for Boa.
//!
//! Provides WHATWG URL Standard APIs:
//! - URL: Parse and manipulate URLs
//! - URLSearchParams: Work with query strings

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor, object::builtins::JsArray,
};

/// Register URL APIs on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    // Register URL constructor
    register_url_constructor(context)?;
    
    // Register URLSearchParams constructor
    register_url_search_params_constructor(context)?;
    
    Ok(())
}

// ============================================================================
// URL Implementation
// ============================================================================

fn register_url_constructor(context: &mut Context) -> JsResult<()> {
    let url_constructor = NativeFunction::from_fn_ptr(url_constructor_fn);
    
    // Create prototype
    let url_prototype = JsObject::with_null_proto();
    
    // Instance methods
    register_method(&url_prototype, "toString", url_to_string, context)?;
    register_method(&url_prototype, "toJSON", url_to_json, context)?;
    
    // Static methods on constructor
    let constructor_obj = url_constructor.to_js_function(context.realm());
    
    let create_object_url = NativeFunction::from_fn_ptr(|_this, _args, _context| {
        // Stub - requires Blob support
        Ok(JsValue::from(JsString::from("blob:null/stub-uuid")))
    });
    
    constructor_obj.define_property_or_throw(
        JsString::from("createObjectURL"),
        PropertyDescriptor::builder()
            .value(create_object_url.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    let revoke_object_url = NativeFunction::from_fn_ptr(|_this, _args, _context| {
        // Stub
        Ok(JsValue::undefined())
    });
    
    constructor_obj.define_property_or_throw(
        JsString::from("revokeObjectURL"),
        PropertyDescriptor::builder()
            .value(revoke_object_url.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Set prototype
    constructor_obj.define_property_or_throw(
        JsString::from("prototype"),
        PropertyDescriptor::builder()
            .value(url_prototype)
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    context.global_object().define_property_or_throw(
        JsString::from("URL"),
        PropertyDescriptor::builder()
            .value(constructor_obj)
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn url_constructor_fn(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url_string = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    let base = if args.len() > 1 {
        Some(args.get_or_undefined(1).to_string(context)?.to_std_string_escaped())
    } else {
        None
    };
    
    // Parse URL
    let parsed = if let Some(base_str) = base {
        match url::Url::parse(&base_str) {
            Ok(base_url) => base_url.join(&url_string).map_err(|e| {
                JsNativeError::typ().with_message(format!("Invalid URL: {}", e))
            })?,
            Err(e) => return Err(JsNativeError::typ().with_message(format!("Invalid base URL: {}", e)).into()),
        }
    } else {
        url::Url::parse(&url_string).map_err(|e| {
            JsNativeError::typ().with_message(format!("Invalid URL: {}", e))
        })?
    };
    
    // Create URL object
    let url_obj = JsObject::with_null_proto();
    
    // Set properties
    set_url_properties(&url_obj, &parsed, context)?;
    
    // Store the parsed URL for later modification
    url_obj.define_property_or_throw(
        JsString::from("__url__"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.to_string()))
            .writable(true)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // Add methods
    register_method(&url_obj, "toString", url_to_string, context)?;
    register_method(&url_obj, "toJSON", url_to_json, context)?;
    
    Ok(JsValue::from(url_obj))
}

fn set_url_properties(url_obj: &JsObject, parsed: &url::Url, context: &mut Context) -> JsResult<()> {
    // href
    url_obj.define_property_or_throw(
        JsString::from("href"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.to_string()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // origin
    url_obj.define_property_or_throw(
        JsString::from("origin"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.origin().ascii_serialization()))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // protocol
    url_obj.define_property_or_throw(
        JsString::from("protocol"),
        PropertyDescriptor::builder()
            .value(JsString::from(format!("{}:", parsed.scheme())))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // username
    url_obj.define_property_or_throw(
        JsString::from("username"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.username()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // password
    url_obj.define_property_or_throw(
        JsString::from("password"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.password().unwrap_or("")))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // host
    url_obj.define_property_or_throw(
        JsString::from("host"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.host_str().unwrap_or("")))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // hostname
    url_obj.define_property_or_throw(
        JsString::from("hostname"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.host_str().unwrap_or("")))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // port
    url_obj.define_property_or_throw(
        JsString::from("port"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.port().map(|p| p.to_string()).unwrap_or_default()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // pathname
    url_obj.define_property_or_throw(
        JsString::from("pathname"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.path()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // search
    url_obj.define_property_or_throw(
        JsString::from("search"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.query().map(|q| format!("?{}", q)).unwrap_or_default()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // hash
    url_obj.define_property_or_throw(
        JsString::from("hash"),
        PropertyDescriptor::builder()
            .value(JsString::from(parsed.fragment().map(|f| format!("#{}", f)).unwrap_or_default()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // searchParams
    let search_params = create_url_search_params(parsed.query().unwrap_or(""), context)?;
    url_obj.define_property_or_throw(
        JsString::from("searchParams"),
        PropertyDescriptor::builder()
            .value(search_params)
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn url_to_string(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        let href = obj.get(JsString::from("href"), context)?;
        return Ok(href);
    }
    Ok(JsValue::from(JsString::from("")))
}

fn url_to_json(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    url_to_string(this, args, context)
}

// ============================================================================
// URLSearchParams Implementation
// ============================================================================

fn register_url_search_params_constructor(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(url_search_params_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("URLSearchParams"),
        PropertyDescriptor::builder()
            .value(constructor.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn url_search_params_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let init = if args.is_empty() {
        String::new()
    } else {
        args.get_or_undefined(0).to_string(context)?.to_std_string_escaped()
    };
    
    create_url_search_params(&init, context)
}

fn create_url_search_params(init: &str, context: &mut Context) -> JsResult<JsValue> {
    let params_obj = JsObject::with_null_proto();
    
    // Parse initial query string and store entries
    let init_str = init.strip_prefix('?').unwrap_or(init);
    let entries: Vec<(String, String)> = form_urlencoded::parse(init_str.as_bytes())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    
    // Store entries as internal array
    let entries_array = JsArray::new(context);
    for (i, (key, value)) in entries.iter().enumerate() {
        let pair = JsArray::new(context);
        pair.push(JsString::from(key.as_str()), context)?;
        pair.push(JsString::from(value.as_str()), context)?;
        entries_array.set(i, JsValue::from(pair), true, context)?;
    }
    
    params_obj.define_property_or_throw(
        JsString::from("__entries__"),
        PropertyDescriptor::builder()
            .value(entries_array)
            .writable(true)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // Methods
    register_method(&params_obj, "append", search_params_append, context)?;
    register_method(&params_obj, "delete", search_params_delete, context)?;
    register_method(&params_obj, "get", search_params_get, context)?;
    register_method(&params_obj, "getAll", search_params_get_all, context)?;
    register_method(&params_obj, "has", search_params_has, context)?;
    register_method(&params_obj, "set", search_params_set, context)?;
    register_method(&params_obj, "sort", search_params_sort, context)?;
    register_method(&params_obj, "toString", search_params_to_string, context)?;
    register_method(&params_obj, "entries", search_params_entries, context)?;
    register_method(&params_obj, "keys", search_params_keys, context)?;
    register_method(&params_obj, "values", search_params_values, context)?;
    register_method(&params_obj, "forEach", search_params_for_each, context)?;
    
    Ok(JsValue::from(params_obj))
}

fn search_params_append(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
    
    if let Some(obj) = this.as_object() {
        if let Ok(entries) = obj.get(JsString::from("__entries__"), context) {
            if let Some(arr) = entries.as_object() {
                let len = arr.get(JsString::from("length"), context)?
                    .to_length(context)? as u32;
                
                let pair = JsArray::new(context);
                pair.push(JsString::from(key.as_str()), context)?;
                pair.push(JsString::from(value.as_str()), context)?;
                arr.set(JsString::from(len.to_string()), JsValue::from(pair), true, context)?;
            }
        }
    }
    
    Ok(JsValue::undefined())
}

fn search_params_delete(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        let filtered: Vec<_> = entries.into_iter()
            .filter(|(k, _)| k != &key)
            .collect();
        set_entries(&obj, filtered, context)?;
    }
    
    Ok(JsValue::undefined())
}

fn search_params_get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        for (k, v) in entries {
            if k == key {
                return Ok(JsValue::from(JsString::from(v)));
            }
        }
    }
    
    Ok(JsValue::null())
}

fn search_params_get_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    
    let result = JsArray::new(context);
    
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        let mut i = 0u32;
        for (k, v) in entries {
            if k == key {
                result.set(i, JsValue::from(JsString::from(v)), true, context)?;
                i += 1;
            }
        }
    }
    
    Ok(JsValue::from(result))
}

fn search_params_has(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        for (k, _) in entries {
            if k == key {
                return Ok(JsValue::from(true));
            }
        }
    }
    
    Ok(JsValue::from(false))
}

fn search_params_set(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
    
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        let mut found = false;
        let mut new_entries: Vec<(String, String)> = Vec::new();
        
        for (k, v) in entries {
            if k == key {
                if !found {
                    new_entries.push((key.clone(), value.clone()));
                    found = true;
                }
                // Skip other entries with same key
            } else {
                new_entries.push((k, v));
            }
        }
        
        if !found {
            new_entries.push((key, value));
        }
        
        set_entries(&obj, new_entries, context)?;
    }
    
    Ok(JsValue::undefined())
}

fn search_params_sort(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        let mut entries = get_entries(&obj, context)?;
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        set_entries(&obj, entries, context)?;
    }
    
    Ok(JsValue::undefined())
}

fn search_params_to_string(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        let result = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(entries)
            .finish();
        return Ok(JsValue::from(JsString::from(result)));
    }
    
    Ok(JsValue::from(JsString::from("")))
}

fn search_params_entries(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Return array of [key, value] pairs
    let result = JsArray::new(context);
    
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        for (i, (k, v)) in entries.iter().enumerate() {
            let pair = JsArray::new(context);
            pair.push(JsString::from(k.as_str()), context)?;
            pair.push(JsString::from(v.as_str()), context)?;
            result.set(i, JsValue::from(pair), true, context)?;
        }
    }
    
    Ok(JsValue::from(result))
}

fn search_params_keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let result = JsArray::new(context);
    
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        for (i, (k, _)) in entries.iter().enumerate() {
            result.set(i, JsValue::from(JsString::from(k.as_str())), true, context)?;
        }
    }
    
    Ok(JsValue::from(result))
}

fn search_params_values(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let result = JsArray::new(context);
    
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        for (i, (_, v)) in entries.iter().enumerate() {
            result.set(i, JsValue::from(JsString::from(v.as_str())), true, context)?;
        }
    }
    
    Ok(JsValue::from(result))
}

fn search_params_for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);
    
    if !callback.is_callable() {
        return Err(JsNativeError::typ().with_message("Callback is not a function").into());
    }
    
    if let Some(obj) = this.as_object() {
        let entries = get_entries(&obj, context)?;
        let callback_obj = callback.as_object().unwrap();
        
        for (k, v) in entries {
            let value = JsValue::from(JsString::from(v));
            let key = JsValue::from(JsString::from(k));
            callback_obj.call(&JsValue::undefined(), &[value, key, this.clone()], context)?;
        }
    }
    
    Ok(JsValue::undefined())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn register_method(
    obj: &JsObject,
    name: &str,
    func: fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>,
    context: &mut Context,
) -> JsResult<()> {
    let native = NativeFunction::from_fn_ptr(func);
    obj.define_property_or_throw(
        JsString::from(name),
        PropertyDescriptor::builder()
            .value(native.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    Ok(())
}

fn get_entries(obj: &JsObject, context: &mut Context) -> JsResult<Vec<(String, String)>> {
    let entries_val = obj.get(JsString::from("__entries__"), context)?;
    let mut result = Vec::new();
    
    if let Some(arr) = entries_val.as_object() {
        let len = arr.get(JsString::from("length"), context)?
            .to_length(context)? as u32;
        
        for i in 0..len {
            if let Ok(pair_val) = arr.get(i, context) {
                if let Some(pair) = pair_val.as_object() {
                    let key = pair.get(0_u32, context)?
                        .to_string(context)?
                        .to_std_string_escaped();
                    let value = pair.get(1_u32, context)?
                        .to_string(context)?
                        .to_std_string_escaped();
                    result.push((key, value));
                }
            }
        }
    }
    
    Ok(result)
}

fn set_entries(obj: &JsObject, entries: Vec<(String, String)>, context: &mut Context) -> JsResult<()> {
    let entries_array = JsArray::new(context);
    
    for (i, (key, value)) in entries.iter().enumerate() {
        let pair = JsArray::new(context);
        pair.push(JsString::from(key.as_str()), context)?;
        pair.push(JsString::from(value.as_str()), context)?;
        entries_array.set(i, JsValue::from(pair), true, context)?;
    }
    
    obj.define_property_or_throw(
        JsString::from("__entries__"),
        PropertyDescriptor::builder()
            .value(entries_array)
            .writable(true)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        // Call URL() as factory function
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"URL('https://example.com:8080/path?query=1#hash').href"#
        )).unwrap();
        
        assert_eq!(
            result.as_string().unwrap().to_std_string_escaped(),
            "https://example.com:8080/path?query=1#hash"
        );
    }

    #[test]
    fn test_url_properties() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"
            const url = URL('https://user:pass@example.com:8080/path?q=1#hash');
            [url.protocol, url.hostname, url.port, url.pathname, url.search, url.hash].join('|')
            "#
        )).unwrap();
        
        assert_eq!(
            result.as_string().unwrap().to_std_string_escaped(),
            "https:|example.com|8080|/path|?q=1|#hash"
        );
    }

    #[test]
    fn test_url_with_base() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"URL('/path', 'https://example.com').href"#
        )).unwrap();
        
        assert_eq!(
            result.as_string().unwrap().to_std_string_escaped(),
            "https://example.com/path"
        );
    }

    #[test]
    fn test_url_search_params() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"
            const params = URLSearchParams('a=1&b=2');
            params.get('a')
            "#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "1");
    }

    #[test]
    fn test_url_search_params_set() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"
            const params = URLSearchParams('a=1&b=2');
            params.set('a', '99');
            params.get('a')
            "#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "99");
    }

    #[test]
    fn test_url_search_params_append() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"
            const params = URLSearchParams('a=1');
            params.append('a', '2');
            params.getAll('a').join(',')
            "#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "1,2");
    }

    #[test]
    fn test_url_search_params_to_string() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"
            const params = URLSearchParams();
            params.append('name', 'test');
            params.append('value', 'hello world');
            params.toString()
            "#
        )).unwrap();
        
        assert_eq!(
            result.as_string().unwrap().to_std_string_escaped(),
            "name=test&value=hello+world"
        );
    }
}

