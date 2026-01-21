// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Web Crypto API implementation for Boa.
//!
//! Provides:
//! - crypto.getRandomValues(): Generate random values
//! - crypto.randomUUID(): Generate a random UUID
//! - crypto.subtle: SubtleCrypto interface (stub)
//!
//! Uses the `getrandom` crate for cryptographically secure randomness.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor, object::builtins::JsUint8Array,
};

/// Register Crypto API on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    let crypto = create_crypto_object(context)?;
    
    context.global_object().define_property_or_throw(
        JsString::from("crypto"),
        PropertyDescriptor::builder()
            .value(crypto)
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(())
}

fn create_crypto_object(context: &mut Context) -> JsResult<JsValue> {
    let crypto = JsObject::with_null_proto();
    
    // getRandomValues
    let get_random_values = NativeFunction::from_fn_ptr(crypto_get_random_values);
    crypto.define_property_or_throw(
        JsString::from("getRandomValues"),
        PropertyDescriptor::builder()
            .value(get_random_values.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // randomUUID
    let random_uuid = NativeFunction::from_fn_ptr(crypto_random_uuid);
    crypto.define_property_or_throw(
        JsString::from("randomUUID"),
        PropertyDescriptor::builder()
            .value(random_uuid.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // subtle (SubtleCrypto stub)
    let subtle = create_subtle_crypto(context)?;
    crypto.define_property_or_throw(
        JsString::from("subtle"),
        PropertyDescriptor::builder()
            .value(subtle)
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(crypto))
}

fn crypto_get_random_values(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let array = args.get_or_undefined(0);
    
    if let Some(obj) = array.as_object() {
        // Check if it's a TypedArray
        if let Ok(typed_array) = JsUint8Array::from_object(obj.clone()) {
            let len = typed_array.length(context)?;
            
            // Check quota (max 65536 bytes per call)
            if len > 65536 {
                return Err(JsNativeError::range()
                    .with_message("getRandomValues: cannot generate more than 65536 bytes")
                    .into());
            }
            
            // Generate random bytes
            let mut bytes = vec![0u8; len as usize];
            getrandom::fill(&mut bytes).map_err(|e| {
                JsNativeError::error()
                    .with_message(format!("Failed to generate random values: {}", e))
            })?;
            
            // Fill the typed array
            for (i, &byte) in bytes.iter().enumerate() {
                typed_array.set(i as u64, JsValue::from(byte), true, context)?;
            }
            
            return Ok(array.clone());
        }
        
        // Check for other integer TypedArrays (Int8Array, Uint16Array, etc.)
        // For now, we support Uint8Array primarily
        if let Ok(length_val) = obj.get(JsString::from("length"), context) {
            if let Some(length) = length_val.as_number() {
                let len = length as usize;
                
                if len > 65536 {
                    return Err(JsNativeError::range()
                        .with_message("getRandomValues: cannot generate more than 65536 bytes")
                        .into());
                }
                
                let mut bytes = vec![0u8; len];
                getrandom::fill(&mut bytes).map_err(|e| {
                    JsNativeError::error()
                        .with_message(format!("Failed to generate random values: {}", e))
                })?;
                
                for (i, &byte) in bytes.iter().enumerate() {
                    obj.set(i as u64, JsValue::from(byte), true, context)?;
                }
                
                return Ok(array.clone());
            }
        }
    }
    
    Err(JsNativeError::typ()
        .with_message("getRandomValues: argument must be a TypedArray")
        .into())
}

fn crypto_random_uuid(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // Generate 16 random bytes
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| {
        JsNativeError::error()
            .with_message(format!("Failed to generate UUID: {}", e))
    })?;
    
    // Set version (4) and variant (RFC 4122)
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant RFC 4122
    
    // Format as UUID string
    let uuid = format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    );
    
    Ok(JsValue::from(JsString::from(uuid.as_str())))
}

// ============================================================================
// SubtleCrypto (Stub implementation)
// ============================================================================

fn create_subtle_crypto(context: &mut Context) -> JsResult<JsValue> {
    let subtle = JsObject::with_null_proto();
    
    // digest
    let digest = NativeFunction::from_fn_ptr(subtle_digest);
    subtle.define_property_or_throw(
        JsString::from("digest"),
        PropertyDescriptor::builder()
            .value(digest.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // generateKey (stub)
    let generate_key = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("generateKey"),
        PropertyDescriptor::builder()
            .value(generate_key.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // encrypt (stub)
    let encrypt = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("encrypt"),
        PropertyDescriptor::builder()
            .value(encrypt.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // decrypt (stub)
    let decrypt = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("decrypt"),
        PropertyDescriptor::builder()
            .value(decrypt.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // sign (stub)
    let sign = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("sign"),
        PropertyDescriptor::builder()
            .value(sign.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // verify (stub)
    let verify = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("verify"),
        PropertyDescriptor::builder()
            .value(verify.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // importKey (stub)
    let import_key = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("importKey"),
        PropertyDescriptor::builder()
            .value(import_key.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // exportKey (stub)
    let export_key = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("exportKey"),
        PropertyDescriptor::builder()
            .value(export_key.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // deriveBits (stub)
    let derive_bits = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("deriveBits"),
        PropertyDescriptor::builder()
            .value(derive_bits.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // deriveKey (stub)
    let derive_key = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("deriveKey"),
        PropertyDescriptor::builder()
            .value(derive_key.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // wrapKey (stub)
    let wrap_key = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("wrapKey"),
        PropertyDescriptor::builder()
            .value(wrap_key.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // unwrapKey (stub)
    let unwrap_key = NativeFunction::from_fn_ptr(subtle_not_implemented);
    subtle.define_property_or_throw(
        JsString::from("unwrapKey"),
        PropertyDescriptor::builder()
            .value(unwrap_key.to_js_function(context.realm()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(subtle))
}

fn subtle_not_implemented(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Err(JsNativeError::error()
        .with_message("SubtleCrypto method not yet implemented")
        .into())
}

fn subtle_digest(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    
    let algorithm = args.get_or_undefined(0);
    let data = args.get_or_undefined(1);
    
    // Get algorithm name
    let algo_name = if algorithm.is_string() {
        algorithm.to_string(context)?.to_std_string_escaped().to_uppercase()
    } else if let Some(obj) = algorithm.as_object() {
        obj.get(JsString::from("name"), context)?
            .to_string(context)?
            .to_std_string_escaped()
            .to_uppercase()
    } else {
        return Err(JsNativeError::typ()
            .with_message("digest: algorithm must be a string or object with 'name' property")
            .into());
    };
    
    // Get data bytes
    let bytes: Vec<u8> = if let Some(obj) = data.as_object() {
        if let Ok(typed_array) = JsUint8Array::from_object(obj.clone()) {
            let len = typed_array.length(context)?;
            let mut result = Vec::with_capacity(len as usize);
            for i in 0..len {
                let val = typed_array.get(i, context)?;
                result.push(val.to_number(context)? as u8);
            }
            result
        } else {
            // Try to get from ArrayBuffer or array-like
            let len_val = obj.get(JsString::from("length"), context)?;
            let len = len_val.to_number(context).unwrap_or(0.0) as u64;
            let mut result = Vec::with_capacity(len as usize);
            for i in 0..len {
                let val = obj.get(i, context)?;
                result.push(val.to_number(context)? as u8);
            }
            result
        }
    } else {
        Vec::new()
    };
    
    // Compute hash based on algorithm
    let hash_result = match algo_name.as_str() {
        "SHA-256" => compute_sha256(&bytes),
        "SHA-384" => compute_sha384(&bytes),
        "SHA-512" => compute_sha512(&bytes),
        "SHA-1" => compute_sha1(&bytes),
        _ => {
            return Err(JsNativeError::error()
                .with_message(format!("digest: unsupported algorithm '{}'", algo_name))
                .into());
        }
    };
    
    // Create ArrayBuffer with result
    let result_array = JsUint8Array::from_iter(hash_result.into_iter(), context)?;
    let buffer = result_array.buffer(context)?;
    
    // Return resolved promise
    let promise = JsPromise::resolve(buffer, context);
    Ok(JsValue::from(promise))
}

// Simple hash implementations using basic algorithms
// In production, these should use proper crypto libraries

fn compute_sha256(data: &[u8]) -> Vec<u8> {
    // Simplified SHA-256 stub - returns deterministic hash based on data
    // TODO: Use proper crypto library (ring, sha2, etc.)
    let mut hash = [0u8; 32];
    
    // Simple hash mixing (NOT cryptographically secure - placeholder only)
    let mut state: u64 = 0x6a09e667bb67ae85;
    for &byte in data {
        state = state.wrapping_mul(0x5851f42d4c957f2d).wrapping_add(byte as u64);
    }
    
    for i in 0..4 {
        let chunk = state.wrapping_mul((i + 1) as u64);
        hash[i * 8..(i + 1) * 8].copy_from_slice(&chunk.to_le_bytes());
    }
    
    hash.to_vec()
}

fn compute_sha384(data: &[u8]) -> Vec<u8> {
    // Placeholder - returns 48-byte hash
    let mut hash = [0u8; 48];
    let mut state: u64 = 0xcbbb9d5dc1059ed8;
    for &byte in data {
        state = state.wrapping_mul(0x629a292a367cd507).wrapping_add(byte as u64);
    }
    for i in 0..6 {
        let chunk = state.wrapping_mul((i + 1) as u64);
        hash[i * 8..(i + 1) * 8].copy_from_slice(&chunk.to_le_bytes());
    }
    hash.to_vec()
}

fn compute_sha512(data: &[u8]) -> Vec<u8> {
    // Placeholder - returns 64-byte hash
    let mut hash = [0u8; 64];
    let mut state: u64 = 0x6a09e667f3bcc908;
    for &byte in data {
        state = state.wrapping_mul(0xbb67ae8584caa73b).wrapping_add(byte as u64);
    }
    for i in 0..8 {
        let chunk = state.wrapping_mul((i + 1) as u64);
        hash[i * 8..(i + 1) * 8].copy_from_slice(&chunk.to_le_bytes());
    }
    hash.to_vec()
}

fn compute_sha1(data: &[u8]) -> Vec<u8> {
    // Placeholder - returns 20-byte hash
    let mut hash = [0u8; 20];
    let mut state: u32 = 0x67452301;
    for &byte in data {
        state = state.wrapping_mul(0xefcdab89).wrapping_add(byte as u32);
    }
    for i in 0..5 {
        let chunk = state.wrapping_mul((i + 1) as u32);
        hash[i * 4..(i + 1) * 4].copy_from_slice(&chunk.to_le_bytes());
    }
    hash.to_vec()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;
    
    #[test]
    fn test_crypto_registration() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof crypto"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "object");
    }
    
    #[test]
    fn test_crypto_get_random_values() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof crypto.getRandomValues"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
    
    #[test]
    fn test_crypto_random_uuid() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "crypto.randomUUID()"
        )).unwrap();
        
        let uuid = result.to_string(&mut ctx).unwrap().to_std_string_escaped();
        
        // UUID format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[8..9], "-");
        assert_eq!(&uuid[13..14], "-");
        assert_eq!(&uuid[14..15], "4"); // Version 4
        assert_eq!(&uuid[18..19], "-");
        assert_eq!(&uuid[23..24], "-");
    }
    
    #[test]
    fn test_crypto_random_uuid_unique() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let uuid1 = ctx.eval(Source::from_bytes("crypto.randomUUID()")).unwrap()
            .to_string(&mut ctx).unwrap().to_std_string_escaped();
        let uuid2 = ctx.eval(Source::from_bytes("crypto.randomUUID()")).unwrap()
            .to_string(&mut ctx).unwrap().to_std_string_escaped();
        
        assert_ne!(uuid1, uuid2);
    }
    
    #[test]
    fn test_subtle_crypto_exists() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof crypto.subtle"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "object");
    }
    
    #[test]
    fn test_subtle_digest_exists() {
        let mut ctx = Context::default();
        register(&mut ctx).unwrap();
        
        let result = ctx.eval(Source::from_bytes(
            "typeof crypto.subtle.digest"
        )).unwrap();
        
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "function");
    }
}
