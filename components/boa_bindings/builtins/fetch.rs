// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Fetch API implementation for Boa.
//!
//! Provides the WHATWG Fetch Standard APIs:
//! - fetch(): Start a request
//! - Request: Represents a resource request
//! - Response: Represents a response to a request
//! - Headers: HTTP headers
//!
//! This implementation uses ureq for synchronous HTTP requests.
//! For now, fetch() executes synchronously but returns a resolved Promise
//! to maintain API compatibility.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor, object::builtins::JsArray,
    object::builtins::JsPromise, object::builtins::JsUint8Array,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;

/// Register Fetch APIs on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    // Register fetch function
    register_fetch(context)?;
    
    // Register Headers constructor
    register_headers(context)?;
    
    // Register Request constructor
    register_request(context)?;
    
    // Register Response constructor
    register_response(context)?;
    
    Ok(())
}

// ============================================================================
// fetch() Function
// ============================================================================

fn register_fetch(context: &mut Context) -> JsResult<()> {
    let fetch_fn = NativeFunction::from_fn_ptr(fetch);
    
    context.global_object().define_property_or_throw(
        JsString::from("fetch"),
        PropertyDescriptor::builder()
            .value(fetch_fn.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

// Thread-local storage for response bodies
thread_local! {
    static RESPONSE_BODIES: RefCell<HashMap<u64, Vec<u8>>> = RefCell::new(HashMap::new());
    static RESPONSE_COUNTER: RefCell<u64> = const { RefCell::new(0) };
}

fn next_response_id() -> u64 {
    RESPONSE_COUNTER.with(|c| {
        let mut counter = c.borrow_mut();
        *counter += 1;
        *counter
    })
}

fn store_response_body(id: u64, body: Vec<u8>) {
    RESPONSE_BODIES.with(|bodies| {
        bodies.borrow_mut().insert(id, body);
    });
}

fn take_response_body(id: u64) -> Option<Vec<u8>> {
    RESPONSE_BODIES.with(|bodies| {
        bodies.borrow_mut().remove(&id)
    })
}

fn fetch(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url = args.get_or_undefined(0);
    let options = args.get_or_undefined(1);
    
    // Extract URL
    let url_string = if url.is_string() {
        url.to_string(context)?.to_std_string_escaped()
    } else if let Some(obj) = url.as_object() {
        // Could be a Request object
        obj.get(JsString::from("url"), context)?
            .to_string(context)?
            .to_std_string_escaped()
    } else {
        return Err(JsNativeError::typ()
            .with_message("First argument must be a URL string or Request object")
            .into());
    };
    
    // Extract method from options
    let method = if let Some(opts) = options.as_object() {
        let method_val = opts.get(JsString::from("method"), context)?;
        if method_val.is_undefined() {
            "GET".to_string()
        } else {
            method_val.to_string(context)?.to_std_string_escaped().to_uppercase()
        }
    } else {
        "GET".to_string()
    };
    
    // Extract headers from options
    let mut request_headers: Vec<(String, String)> = Vec::new();
    if let Some(opts) = options.as_object() {
        let headers_val = opts.get(JsString::from("headers"), context)?;
        if let Some(headers_obj) = headers_val.as_object() {
            // Try to get __ferro_headers internal map
            if let Ok(internal) = headers_obj.get(JsString::from("__ferro_headers"), context) {
                if let Some(arr) = internal.as_object() {
                    // Get length from "length" property
                    if let Ok(len_val) = arr.get(JsString::from("length"), context) {
                        let length = len_val.to_number(context).unwrap_or(0.0) as u64;
                        for i in 0..length {
                            if let Ok(pair) = arr.get(i, context) {
                                if let Some(pair_arr) = pair.as_object() {
                                    let key = pair_arr.get(0, context)?
                                        .to_string(context)?
                                        .to_std_string_escaped();
                                    let val = pair_arr.get(1, context)?
                                        .to_string(context)?
                                        .to_std_string_escaped();
                                    request_headers.push((key, val));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Extract body from options
    let body_data: Option<Vec<u8>> = if let Some(opts) = options.as_object() {
        let body_val = opts.get(JsString::from("body"), context)?;
        if body_val.is_undefined() || body_val.is_null() {
            None
        } else if body_val.is_string() {
            Some(body_val.to_string(context)?.to_std_string_escaped().into_bytes())
        } else {
            None
        }
    } else {
        None
    };
    
    log::debug!("[Fetch] {} request to: {}", method, url_string);
    
    // Perform the actual HTTP request using ureq
    let response = perform_http_request(&url_string, &method, &request_headers, body_data, context)?;
    
    // Wrap in a resolved promise
    let promise = JsPromise::resolve(response, context);
    
    Ok(JsValue::from(promise))
}

fn perform_http_request(
    url: &str,
    method: &str,
    headers: &[(String, String)],
    body: Option<Vec<u8>>,
    context: &mut Context,
) -> JsResult<JsValue> {
    // Build ureq request
    let agent = ureq::Agent::new_with_defaults();
    
    // Build request based on method - ureq v3 has different APIs for GET vs POST
    let result = match method {
        "GET" => {
            let mut req = agent.get(url);
            for (key, value) in headers {
                req = req.header(key.as_str(), value.as_str());
            }
            req = req.header("User-Agent", "Ferro/1.0 (Boa JS Engine)");
            req.call()
        }
        "HEAD" => {
            let mut req = agent.head(url);
            for (key, value) in headers {
                req = req.header(key.as_str(), value.as_str());
            }
            req = req.header("User-Agent", "Ferro/1.0 (Boa JS Engine)");
            req.call()
        }
        "DELETE" => {
            let mut req = agent.delete(url);
            for (key, value) in headers {
                req = req.header(key.as_str(), value.as_str());
            }
            req = req.header("User-Agent", "Ferro/1.0 (Boa JS Engine)");
            req.call()
        }
        "POST" => {
            let mut req = agent.post(url);
            for (key, value) in headers {
                req = req.header(key.as_str(), value.as_str());
            }
            req = req.header("User-Agent", "Ferro/1.0 (Boa JS Engine)");
            if let Some(body_data) = body {
                req.send(&body_data[..])
            } else {
                req.send(&[][..])
            }
        }
        "PUT" => {
            let mut req = agent.put(url);
            for (key, value) in headers {
                req = req.header(key.as_str(), value.as_str());
            }
            req = req.header("User-Agent", "Ferro/1.0 (Boa JS Engine)");
            if let Some(body_data) = body {
                req.send(&body_data[..])
            } else {
                req.send(&[][..])
            }
        }
        "PATCH" => {
            let mut req = agent.patch(url);
            for (key, value) in headers {
                req = req.header(key.as_str(), value.as_str());
            }
            req = req.header("User-Agent", "Ferro/1.0 (Boa JS Engine)");
            if let Some(body_data) = body {
                req.send(&body_data[..])
            } else {
                req.send(&[][..])
            }
        }
        _ => {
            // Default to GET for unknown methods
            let mut req = agent.get(url);
            for (key, value) in headers {
                req = req.header(key.as_str(), value.as_str());
            }
            req = req.header("User-Agent", "Ferro/1.0 (Boa JS Engine)");
            req.call()
        }
    };
    
    match result {
        Ok(response) => {
            create_response_from_ureq(response, url, context)
        }
        Err(e) => {
            log::warn!("[Fetch] Network error: {}", e);
            // Return a network error response
            create_network_error_response(&e.to_string(), url, context)
        }
    }
}

fn create_response_from_ureq(
    response: ureq::http::Response<ureq::Body>,
    url: &str,
    context: &mut Context,
) -> JsResult<JsValue> {
    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("OK").to_string();
    
    // Extract headers
    let response_headers: Vec<(String, String)> = response.headers()
        .iter()
        .map(|(name, value)| {
            (name.to_string(), value.to_str().unwrap_or("").to_string())
        })
        .collect();
    
    // Read body
    let mut body_bytes = Vec::new();
    if let Err(e) = response.into_body().into_reader().read_to_end(&mut body_bytes) {
        log::warn!("[Fetch] Error reading response body: {}", e);
    }
    
    create_response_object(
        status,
        &status_text,
        &response_headers,
        body_bytes,
        url,
        context,
    )
}

#[allow(dead_code)]
fn create_error_response(status: u16, url: &str, context: &mut Context) -> JsResult<JsValue> {
    let status_text = match status {
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Error",
    };
    
    create_response_object(status, status_text, &[], Vec::new(), url, context)
}

fn create_network_error_response(message: &str, url: &str, context: &mut Context) -> JsResult<JsValue> {
    let response = JsObject::with_null_proto();
    let response_id = next_response_id();
    
    // Store empty body
    store_response_body(response_id, Vec::new());
    
    // Set properties
    define_response_property(&response, "ok", JsValue::from(false), context)?;
    define_response_property(&response, "status", JsValue::from(0), context)?;
    define_response_property(&response, "statusText", JsString::from(""), context)?;
    define_response_property(&response, "url", JsString::from(url), context)?;
    define_response_property(&response, "type", JsString::from("error"), context)?;
    define_response_property(&response, "redirected", JsValue::from(false), context)?;
    define_response_property(&response, "bodyUsed", JsValue::from(false), context)?;
    define_response_property(&response, "__ferro_response_id", JsValue::from(response_id), context)?;
    define_response_property(&response, "__ferro_error", JsString::from(message), context)?;
    
    // Empty headers
    let headers = create_headers_object(context)?;
    define_response_property(&response, "headers", headers, context)?;
    
    // Body methods
    register_method(&response, "text", response_text, context)?;
    register_method(&response, "json", response_json, context)?;
    register_method(&response, "blob", response_blob, context)?;
    register_method(&response, "arrayBuffer", response_array_buffer, context)?;
    register_method(&response, "formData", response_form_data, context)?;
    register_method(&response, "clone", response_clone, context)?;
    
    Ok(JsValue::from(response))
}

fn create_response_object(
    status: u16,
    status_text: &str,
    headers: &[(String, String)],
    body: Vec<u8>,
    url: &str,
    context: &mut Context,
) -> JsResult<JsValue> {
    let response = JsObject::with_null_proto();
    let response_id = next_response_id();
    
    // Store body for later retrieval
    store_response_body(response_id, body);
    
    // Basic properties
    define_response_property(&response, "ok", JsValue::from(status >= 200 && status < 300), context)?;
    define_response_property(&response, "status", JsValue::from(status), context)?;
    define_response_property(&response, "statusText", JsString::from(status_text), context)?;
    define_response_property(&response, "url", JsString::from(url), context)?;
    define_response_property(&response, "type", JsString::from("basic"), context)?;
    define_response_property(&response, "redirected", JsValue::from(false), context)?;
    define_response_property(&response, "bodyUsed", JsValue::from(false), context)?;
    define_response_property(&response, "__ferro_response_id", JsValue::from(response_id), context)?;
    
    // Headers object
    let headers_obj = create_headers_with_entries(headers, context)?;
    define_response_property(&response, "headers", headers_obj, context)?;
    
    // Body methods
    register_method(&response, "text", response_text, context)?;
    register_method(&response, "json", response_json, context)?;
    register_method(&response, "blob", response_blob, context)?;
    register_method(&response, "arrayBuffer", response_array_buffer, context)?;
    register_method(&response, "formData", response_form_data, context)?;
    register_method(&response, "clone", response_clone, context)?;
    
    Ok(JsValue::from(response))
}

fn define_response_property(obj: &JsObject, name: &str, value: impl Into<JsValue>, context: &mut Context) -> JsResult<()> {
    obj.define_property_or_throw(
        JsString::from(name),
        PropertyDescriptor::builder()
            .value(value)
            .writable(name == "bodyUsed") // Only bodyUsed is writable
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    Ok(())
}

fn create_headers_with_entries(entries: &[(String, String)], context: &mut Context) -> JsResult<JsValue> {
    let headers = JsObject::with_null_proto();
    
    // Internal storage array
    let internal = JsArray::new(context);
    for (key, value) in entries {
        let pair = JsArray::new(context);
        pair.set(0, JsString::from(key.as_str()), true, context)?;
        pair.set(1, JsString::from(value.as_str()), true, context)?;
        
        let len = internal.length(context)?;
        internal.set(len, pair, true, context)?;
    }
    
    headers.define_property_or_throw(
        JsString::from("__ferro_headers"),
        PropertyDescriptor::builder()
            .value(internal)
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Add methods
    register_method(&headers, "get", headers_get, context)?;
    register_method(&headers, "set", headers_set, context)?;
    register_method(&headers, "has", headers_has, context)?;
    register_method(&headers, "delete", headers_delete, context)?;
    register_method(&headers, "append", headers_append, context)?;
    register_method(&headers, "entries", headers_entries, context)?;
    register_method(&headers, "keys", headers_keys, context)?;
    register_method(&headers, "values", headers_values, context)?;
    register_method(&headers, "forEach", headers_for_each, context)?;
    
    Ok(JsValue::from(headers))
}

#[allow(dead_code)]
fn create_stub_response(url: &str, context: &mut Context) -> JsResult<JsValue> {
    let response = JsObject::with_null_proto();
    
    // Basic properties
    response.define_property_or_throw(
        JsString::from("ok"),
        PropertyDescriptor::builder()
            .value(JsValue::from(true))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("status"),
        PropertyDescriptor::builder()
            .value(JsValue::from(200))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("statusText"),
        PropertyDescriptor::builder()
            .value(JsString::from("OK"))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("url"),
        PropertyDescriptor::builder()
            .value(JsString::from(url))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("type"),
        PropertyDescriptor::builder()
            .value(JsString::from("basic"))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("redirected"),
        PropertyDescriptor::builder()
            .value(JsValue::from(false))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("bodyUsed"),
        PropertyDescriptor::builder()
            .value(JsValue::from(false))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Headers
    let headers = create_headers_object(context)?;
    response.define_property_or_throw(
        JsString::from("headers"),
        PropertyDescriptor::builder()
            .value(headers)
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Body methods
    register_method(&response, "text", response_text, context)?;
    register_method(&response, "json", response_json, context)?;
    register_method(&response, "blob", response_blob, context)?;
    register_method(&response, "arrayBuffer", response_array_buffer, context)?;
    register_method(&response, "formData", response_form_data, context)?;
    register_method(&response, "clone", response_clone, context)?;
    
    Ok(JsValue::from(response))
}

fn get_response_body(this: &JsValue, context: &mut Context) -> JsResult<Option<Vec<u8>>> {
    if let Some(obj) = this.as_object() {
        // Check if body was already used
        let body_used = obj.get(JsString::from("bodyUsed"), context)?;
        if body_used.to_boolean() {
            return Err(JsNativeError::typ()
                .with_message("Body has already been consumed")
                .into());
        }
        
        // Mark body as used
        obj.set(JsString::from("bodyUsed"), JsValue::from(true), true, context)?;
        
        // Get response ID and retrieve body
        let id_val = obj.get(JsString::from("__ferro_response_id"), context)?;
        if let Some(id) = id_val.as_number() {
            return Ok(take_response_body(id as u64));
        }
    }
    Ok(None)
}

fn response_text(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let body = get_response_body(this, context)?;
    
    let text = match body {
        Some(bytes) => {
            String::from_utf8_lossy(&bytes).into_owned()
        }
        None => String::new(),
    };
    
    let promise = JsPromise::resolve(JsString::from(text.as_str()), context);
    Ok(JsValue::from(promise))
}

fn response_json(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let body = get_response_body(this, context)?;
    
    let text = match body {
        Some(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        None => "null".to_string(),
    };
    
    // Parse JSON using Boa's JSON.parse
    let global = context.global_object();
    let json_obj = global.get(JsString::from("JSON"), context)?;
    
    if let Some(json) = json_obj.as_object() {
        let parse_fn = json.get(JsString::from("parse"), context)?;
        if let Some(parse) = parse_fn.as_callable() {
            let parsed = parse.call(&json_obj, &[JsString::from(text.as_str()).into()], context)?;
            let promise = JsPromise::resolve(parsed, context);
            return Ok(JsValue::from(promise));
        }
    }
    
    // Fallback: return null
    let promise = JsPromise::resolve(JsValue::null(), context);
    Ok(JsValue::from(promise))
}

fn response_blob(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let body = get_response_body(this, context)?;
    let bytes = body.unwrap_or_default();
    
    // Create Blob-like object
    let blob = JsObject::with_null_proto();
    
    // Store bytes internally
    let byte_array = JsArray::new(context);
    for (i, &byte) in bytes.iter().enumerate() {
        byte_array.set(i as u64, JsValue::from(byte), true, context)?;
    }
    
    blob.define_property_or_throw(
        JsString::from("size"),
        PropertyDescriptor::builder()
            .value(JsValue::from(bytes.len()))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    blob.define_property_or_throw(
        JsString::from("type"),
        PropertyDescriptor::builder()
            .value(JsString::from("application/octet-stream"))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    blob.define_property_or_throw(
        JsString::from("__ferro_bytes"),
        PropertyDescriptor::builder()
            .value(byte_array)
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Add slice method
    register_method(&blob, "slice", blob_slice, context)?;
    register_method(&blob, "text", blob_text, context)?;
    register_method(&blob, "arrayBuffer", blob_array_buffer, context)?;
    
    let promise = JsPromise::resolve(JsValue::from(blob), context);
    Ok(JsValue::from(promise))
}

fn blob_slice(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Stub slice - returns empty blob
    let blob = JsObject::with_null_proto();
    blob.define_property_or_throw(
        JsString::from("size"),
        PropertyDescriptor::builder()
            .value(JsValue::from(0))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    Ok(JsValue::from(blob))
}

fn blob_text(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        let bytes_val = obj.get(JsString::from("__ferro_bytes"), context)?;
        if let Some(arr) = bytes_val.as_object() {
            let len_val = arr.get(JsString::from("length"), context)?;
            let len = len_val.to_number(context).unwrap_or(0.0) as u64;
            let mut bytes = Vec::with_capacity(len as usize);
            for i in 0..len {
                let val = arr.get(i, context)?;
                if let Some(n) = val.as_number() {
                    bytes.push(n as u8);
                }
            }
            let text = String::from_utf8_lossy(&bytes);
            let promise = JsPromise::resolve(JsString::from(text.as_ref()), context);
            return Ok(JsValue::from(promise));
        }
    }
    let promise = JsPromise::resolve(JsString::from(""), context);
    Ok(JsValue::from(promise))
}

fn blob_array_buffer(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        let bytes_val = obj.get(JsString::from("__ferro_bytes"), context)?;
        if let Some(arr) = bytes_val.as_object() {
            let len_val = arr.get(JsString::from("length"), context)?;
            let len = len_val.to_number(context).unwrap_or(0.0) as u64;
            let mut bytes = Vec::with_capacity(len as usize);
            for i in 0..len {
                let val = arr.get(i, context)?;
                if let Some(n) = val.as_number() {
                    bytes.push(n as u8);
                }
            }
            // Create ArrayBuffer using Uint8Array
            let uint8_arr = JsUint8Array::from_iter(bytes.into_iter(), context)?;
            let promise = JsPromise::resolve(uint8_arr.buffer(context)?, context);
            return Ok(JsValue::from(promise));
        }
    }
    let uint8_arr = JsUint8Array::from_iter(std::iter::empty(), context)?;
    let promise = JsPromise::resolve(uint8_arr.buffer(context)?, context);
    Ok(JsValue::from(promise))
}

fn response_array_buffer(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let body = get_response_body(this, context)?;
    let bytes = body.unwrap_or_default();
    
    // Create ArrayBuffer using Uint8Array
    let uint8_arr = JsUint8Array::from_iter(bytes.into_iter(), context)?;
    let promise = JsPromise::resolve(uint8_arr.buffer(context)?, context);
    Ok(JsValue::from(promise))
}

fn response_form_data(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // FormData parsing is complex - return empty FormData-like object for now
    let form_data = JsObject::with_null_proto();
    
    // Add basic methods
    register_method(&form_data, "get", formdata_get, context)?;
    register_method(&form_data, "set", formdata_set, context)?;
    register_method(&form_data, "has", formdata_has, context)?;
    register_method(&form_data, "append", formdata_append, context)?;
    register_method(&form_data, "delete", formdata_delete, context)?;
    
    let promise = JsPromise::resolve(JsValue::from(form_data), context);
    Ok(JsValue::from(promise))
}

fn formdata_get(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::null())
}

fn formdata_set(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn formdata_has(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(false))
}

fn formdata_append(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn formdata_delete(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn response_clone(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        // Check if body was already used
        let body_used = obj.get(JsString::from("bodyUsed"), context)?;
        if body_used.to_boolean() {
            return Err(JsNativeError::typ()
                .with_message("Cannot clone a Response whose body has already been used")
                .into());
        }
        
        // Get properties
        let status = obj.get(JsString::from("status"), context)?
            .as_number().unwrap_or(200.0) as u16;
        let status_text = obj.get(JsString::from("statusText"), context)?
            .to_string(context)?
            .to_std_string_escaped();
        let url = obj.get(JsString::from("url"), context)?
            .to_string(context)?
            .to_std_string_escaped();
        
        // Get response ID to copy body
        let id_val = obj.get(JsString::from("__ferro_response_id"), context)?;
        let body = if let Some(id) = id_val.as_number() {
            RESPONSE_BODIES.with(|bodies| {
                bodies.borrow().get(&(id as u64)).cloned()
            })
        } else {
            None
        };
        
        // Create new response with copied body
        return create_response_object(
            status,
            &status_text,
            &[], // Headers would need to be copied too
            body.unwrap_or_default(),
            &url,
            context,
        );
    }
    
    create_response_object(200, "OK", &[], Vec::new(), "", context)
}

// ============================================================================
// Headers Implementation
// ============================================================================

fn register_headers(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(headers_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("Headers"),
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

fn headers_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let headers = create_headers_object(context)?;
    
    // Initialize with provided headers if any
    if !args.is_empty() {
        let init = args.get_or_undefined(0);
        if let Some(init_obj) = init.as_object() {
            // Copy headers from init object
            if let Ok(keys) = init_obj.own_property_keys(context) {
                for key in keys {
                    if let Ok(value) = init_obj.get(key.clone(), context) {
                        let key_str = key.to_string();
                        let value_str = value.to_string(context)?.to_std_string_escaped();
                        headers_set_internal(&headers.as_object().unwrap(), &key_str, &value_str, context)?;
                    }
                }
            }
        }
    }
    
    Ok(headers)
}

fn create_headers_object(context: &mut Context) -> JsResult<JsValue> {
    let headers = JsObject::with_null_proto();
    
    // Internal storage for headers
    let entries = JsArray::new(context);
    headers.define_property_or_throw(
        JsString::from("__entries__"),
        PropertyDescriptor::builder()
            .value(entries)
            .writable(true)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;
    
    // Methods
    register_method(&headers, "append", headers_append, context)?;
    register_method(&headers, "delete", headers_delete, context)?;
    register_method(&headers, "get", headers_get, context)?;
    register_method(&headers, "has", headers_has, context)?;
    register_method(&headers, "set", headers_set, context)?;
    register_method(&headers, "entries", headers_entries, context)?;
    register_method(&headers, "keys", headers_keys, context)?;
    register_method(&headers, "values", headers_values, context)?;
    register_method(&headers, "forEach", headers_for_each, context)?;
    
    Ok(JsValue::from(headers))
}

fn headers_set_internal(headers: &JsObject, key: &str, value: &str, context: &mut Context) -> JsResult<()> {
    let entries = headers.get(JsString::from("__entries__"), context)?;
    if let Some(arr) = entries.as_object() {
        let len = arr.get(JsString::from("length"), context)?
            .to_length(context)? as u32;
        
        // Check if key already exists
        let key_lower = key.to_lowercase();
        for i in 0..len {
            if let Ok(entry) = arr.get(i, context) {
                if let Some(entry_arr) = entry.as_object() {
                    let existing_key = entry_arr.get(0_u32, context)?
                        .to_string(context)?
                        .to_std_string_escaped()
                        .to_lowercase();
                    if existing_key == key_lower {
                        // Update existing
                        let pair = JsArray::new(context);
                        pair.push(JsString::from(key), context)?;
                        pair.push(JsString::from(value), context)?;
                        arr.set(i, JsValue::from(pair), true, context)?;
                        return Ok(());
                    }
                }
            }
        }
        
        // Add new
        let pair = JsArray::new(context);
        pair.push(JsString::from(key), context)?;
        pair.push(JsString::from(value), context)?;
        arr.set(len, JsValue::from(pair), true, context)?;
    }
    
    Ok(())
}

fn headers_append(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
    
    if let Some(obj) = this.as_object() {
        let entries = obj.get(JsString::from("__entries__"), context)?;
        if let Some(arr) = entries.as_object() {
            let len = arr.get(JsString::from("length"), context)?
                .to_length(context)? as u32;
            
            let pair = JsArray::new(context);
            pair.push(JsString::from(key.as_str()), context)?;
            pair.push(JsString::from(value.as_str()), context)?;
            arr.set(len, JsValue::from(pair), true, context)?;
        }
    }
    
    Ok(JsValue::undefined())
}

fn headers_delete(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_lowercase();
    
    if let Some(obj) = this.as_object() {
        let entries = get_header_entries(&obj, context)?;
        let filtered: Vec<_> = entries.into_iter()
            .filter(|(k, _)| k.to_lowercase() != key)
            .collect();
        set_header_entries(&obj, filtered, context)?;
    }
    
    Ok(JsValue::undefined())
}

fn headers_get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_lowercase();
    
    if let Some(obj) = this.as_object() {
        let entries = get_header_entries(&obj, context)?;
        let values: Vec<_> = entries.iter()
            .filter(|(k, _)| k.to_lowercase() == key)
            .map(|(_, v)| v.clone())
            .collect();
        
        if !values.is_empty() {
            return Ok(JsValue::from(JsString::from(values.join(", "))));
        }
    }
    
    Ok(JsValue::null())
}

fn headers_has(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_lowercase();
    
    if let Some(obj) = this.as_object() {
        let entries = get_header_entries(&obj, context)?;
        for (k, _) in entries {
            if k.to_lowercase() == key {
                return Ok(JsValue::from(true));
            }
        }
    }
    
    Ok(JsValue::from(false))
}

fn headers_set(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
    
    if let Some(obj) = this.as_object() {
        headers_set_internal(&obj, &key, &value, context)?;
    }
    
    Ok(JsValue::undefined())
}

fn headers_entries(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let result = JsArray::new(context);
    
    if let Some(obj) = this.as_object() {
        let entries = get_header_entries(&obj, context)?;
        for (i, (k, v)) in entries.iter().enumerate() {
            let pair = JsArray::new(context);
            pair.push(JsString::from(k.as_str()), context)?;
            pair.push(JsString::from(v.as_str()), context)?;
            result.set(i, JsValue::from(pair), true, context)?;
        }
    }
    
    Ok(JsValue::from(result))
}

fn headers_keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let result = JsArray::new(context);
    
    if let Some(obj) = this.as_object() {
        let entries = get_header_entries(&obj, context)?;
        for (i, (k, _)) in entries.iter().enumerate() {
            result.set(i, JsValue::from(JsString::from(k.as_str())), true, context)?;
        }
    }
    
    Ok(JsValue::from(result))
}

fn headers_values(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let result = JsArray::new(context);
    
    if let Some(obj) = this.as_object() {
        let entries = get_header_entries(&obj, context)?;
        for (i, (_, v)) in entries.iter().enumerate() {
            result.set(i, JsValue::from(JsString::from(v.as_str())), true, context)?;
        }
    }
    
    Ok(JsValue::from(result))
}

fn headers_for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);
    
    if !callback.is_callable() {
        return Err(JsNativeError::typ().with_message("Callback is not a function").into());
    }
    
    if let Some(obj) = this.as_object() {
        let entries = get_header_entries(&obj, context)?;
        let callback_obj = callback.as_object().unwrap();
        
        for (k, v) in entries {
            callback_obj.call(
                &JsValue::undefined(),
                &[
                    JsValue::from(JsString::from(v)),
                    JsValue::from(JsString::from(k)),
                    this.clone(),
                ],
                context,
            )?;
        }
    }
    
    Ok(JsValue::undefined())
}

// ============================================================================
// Request Implementation
// ============================================================================

fn register_request(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(request_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("Request"),
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

fn request_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let options = args.get_or_undefined(1);
    
    let request = JsObject::with_null_proto();
    
    // URL
    request.define_property_or_throw(
        JsString::from("url"),
        PropertyDescriptor::builder()
            .value(JsString::from(url.as_str()))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Method (default GET)
    let method = if let Some(opts) = options.as_object() {
        opts.get(JsString::from("method"), context)?
            .to_string(context)?
            .to_std_string_escaped()
            .to_uppercase()
    } else {
        "GET".to_string()
    };
    
    request.define_property_or_throw(
        JsString::from("method"),
        PropertyDescriptor::builder()
            .value(JsString::from(method.as_str()))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Headers
    let headers = create_headers_object(context)?;
    if let Some(opts) = options.as_object() {
        if let Ok(init_headers) = opts.get(JsString::from("headers"), context) {
            if let Some(init_obj) = init_headers.as_object() {
                if let Ok(keys) = init_obj.own_property_keys(context) {
                    for key in keys {
                        if let Ok(value) = init_obj.get(key.clone(), context) {
                            let key_str = key.to_string();
                            let value_str = value.to_string(context)?.to_std_string_escaped();
                            headers_set_internal(&headers.as_object().unwrap(), &key_str, &value_str, context)?;
                        }
                    }
                }
            }
        }
    }
    
    request.define_property_or_throw(
        JsString::from("headers"),
        PropertyDescriptor::builder()
            .value(headers)
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Body
    request.define_property_or_throw(
        JsString::from("body"),
        PropertyDescriptor::builder()
            .value(JsValue::null())
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    request.define_property_or_throw(
        JsString::from("bodyUsed"),
        PropertyDescriptor::builder()
            .value(JsValue::from(false))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Mode, credentials, cache, redirect
    let defaults = [
        ("mode", "cors"),
        ("credentials", "same-origin"),
        ("cache", "default"),
        ("redirect", "follow"),
        ("referrer", "about:client"),
        ("referrerPolicy", ""),
        ("integrity", ""),
    ];
    
    for (prop, default_value) in defaults {
        let value = if let Some(opts) = options.as_object() {
            opts.get(JsString::from(prop), context)?
                .to_string(context)?
                .to_std_string_escaped()
        } else {
            default_value.to_string()
        };
        
        request.define_property_or_throw(
            JsString::from(prop),
            PropertyDescriptor::builder()
                .value(JsString::from(value.as_str()))
                .writable(false)
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )?;
    }
    
    // Clone method
    register_method(&request, "clone", request_clone, context)?;
    
    Ok(JsValue::from(request))
}

fn request_clone(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        let url = obj.get(JsString::from("url"), context)?;
        return request_constructor(&JsValue::undefined(), &[url], context);
    }
    
    Err(JsNativeError::typ().with_message("Invalid Request object").into())
}

// ============================================================================
// Response Implementation
// ============================================================================

fn register_response(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(response_constructor);
    let constructor_obj = constructor.to_js_function(context.realm());
    
    // Static methods
    let error_fn = NativeFunction::from_fn_ptr(response_error);
    constructor_obj.define_property_or_throw(
        JsString::from("error"),
        PropertyDescriptor::builder()
            .value(error_fn.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    let redirect_fn = NativeFunction::from_fn_ptr(response_redirect);
    constructor_obj.define_property_or_throw(
        JsString::from("redirect"),
        PropertyDescriptor::builder()
            .value(redirect_fn.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    let json_fn = NativeFunction::from_fn_ptr(response_json_static);
    constructor_obj.define_property_or_throw(
        JsString::from("json"),
        PropertyDescriptor::builder()
            .value(json_fn.to_js_function(context.realm()))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    context.global_object().define_property_or_throw(
        JsString::from("Response"),
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

fn response_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _body = args.get_or_undefined(0);
    let init = args.get_or_undefined(1);
    
    let response = JsObject::with_null_proto();
    
    // Status
    let status = if let Some(opts) = init.as_object() {
        opts.get(JsString::from("status"), context)?
            .to_number(context)
            .unwrap_or(200.0) as u16
    } else {
        200
    };
    
    response.define_property_or_throw(
        JsString::from("status"),
        PropertyDescriptor::builder()
            .value(JsValue::from(status))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // OK
    response.define_property_or_throw(
        JsString::from("ok"),
        PropertyDescriptor::builder()
            .value(JsValue::from(status >= 200 && status < 300))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // StatusText
    let status_text = if let Some(opts) = init.as_object() {
        opts.get(JsString::from("statusText"), context)?
            .to_string(context)?
            .to_std_string_escaped()
    } else {
        "".to_string()
    };
    
    response.define_property_or_throw(
        JsString::from("statusText"),
        PropertyDescriptor::builder()
            .value(JsString::from(status_text.as_str()))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Headers
    let headers = create_headers_object(context)?;
    response.define_property_or_throw(
        JsString::from("headers"),
        PropertyDescriptor::builder()
            .value(headers)
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Other properties
    response.define_property_or_throw(
        JsString::from("type"),
        PropertyDescriptor::builder()
            .value(JsString::from("default"))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("url"),
        PropertyDescriptor::builder()
            .value(JsString::from(""))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("redirected"),
        PropertyDescriptor::builder()
            .value(JsValue::from(false))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("bodyUsed"),
        PropertyDescriptor::builder()
            .value(JsValue::from(false))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Body methods
    register_method(&response, "text", response_text, context)?;
    register_method(&response, "json", response_json, context)?;
    register_method(&response, "blob", response_blob, context)?;
    register_method(&response, "arrayBuffer", response_array_buffer, context)?;
    register_method(&response, "formData", response_form_data, context)?;
    register_method(&response, "clone", response_clone, context)?;
    
    Ok(JsValue::from(response))
}

fn response_error(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let response = JsObject::with_null_proto();
    
    response.define_property_or_throw(
        JsString::from("type"),
        PropertyDescriptor::builder()
            .value(JsString::from("error"))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("status"),
        PropertyDescriptor::builder()
            .value(JsValue::from(0))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("ok"),
        PropertyDescriptor::builder()
            .value(JsValue::from(false))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(response))
}

fn response_redirect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let status = args.get_or_undefined(1)
        .to_number(context)
        .unwrap_or(302.0) as u16;
    
    if ![301, 302, 303, 307, 308].contains(&status) {
        return Err(JsNativeError::range()
            .with_message("Invalid redirect status")
            .into());
    }
    
    let response = JsObject::with_null_proto();
    
    response.define_property_or_throw(
        JsString::from("status"),
        PropertyDescriptor::builder()
            .value(JsValue::from(status))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("ok"),
        PropertyDescriptor::builder()
            .value(JsValue::from(false))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    response.define_property_or_throw(
        JsString::from("redirected"),
        PropertyDescriptor::builder()
            .value(JsValue::from(true))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // Add Location header
    let headers = create_headers_object(context)?;
    headers_set_internal(&headers.as_object().unwrap(), "Location", &url, context)?;
    
    response.define_property_or_throw(
        JsString::from("headers"),
        PropertyDescriptor::builder()
            .value(headers)
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(response))
}

fn response_json_static(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let data = args.get_or_undefined(0);
    let init = args.get_or_undefined(1);
    
    // Create response with JSON body
    let response = response_constructor(&JsValue::undefined(), &[JsValue::undefined(), init.clone()], context)?;
    
    if let Some(obj) = response.as_object() {
        // Set Content-Type header
        if let Ok(headers_val) = obj.get(JsString::from("headers"), context) {
            if let Some(headers) = headers_val.as_object() {
                headers_set_internal(&headers, "Content-Type", "application/json", context)?;
            }
        }
        
        // Store body data for later retrieval
        obj.define_property_or_throw(
            JsString::from("__body__"),
            PropertyDescriptor::builder()
                .value(data.clone())
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;
    }
    
    Ok(response)
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

fn get_header_entries(obj: &JsObject, context: &mut Context) -> JsResult<Vec<(String, String)>> {
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

fn set_header_entries(obj: &JsObject, entries: Vec<(String, String)>, context: &mut Context) -> JsResult<()> {
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
    fn test_fetch_registration() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let fetch = context.global_object()
            .get(JsString::from("fetch"), &mut context)
            .unwrap();
        assert!(fetch.is_callable());
    }

    #[test]
    fn test_headers_registration() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let headers = context.global_object()
            .get(JsString::from("Headers"), &mut context)
            .unwrap();
        assert!(headers.is_callable());
    }

    #[test]
    fn test_request_registration() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let request = context.global_object()
            .get(JsString::from("Request"), &mut context)
            .unwrap();
        assert!(request.is_callable());
    }

    #[test]
    fn test_response_registration() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let response = context.global_object()
            .get(JsString::from("Response"), &mut context)
            .unwrap();
        assert!(response.is_callable());
    }

    #[test]
    fn test_headers_set_get() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        // Call Headers() as factory function (not constructor)
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"
            const h = Headers();
            h.set('Content-Type', 'application/json');
            h.get('Content-Type')
            "#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "application/json");
    }

    #[test]
    fn test_request_creation() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"
            const req = Request('https://example.com', { method: 'POST' });
            req.method
            "#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "POST");
    }

    #[test]
    fn test_response_creation() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"
            const res = Response(null, { status: 404, statusText: 'Not Found' });
            res.status
            "#
        )).unwrap();
        
        assert_eq!(result.as_number().unwrap(), 404.0);
    }
}


