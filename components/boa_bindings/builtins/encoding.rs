// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Encoding API implementation for Boa.
//!
//! Provides the WHATWG Encoding Standard APIs:
//! - TextEncoder: Encode strings to UTF-8 bytes
//! - TextDecoder: Decode bytes to strings

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue,
    NativeFunction, property::PropertyDescriptor,
};

/// Register Encoding APIs on the global object.
pub fn register(context: &mut Context) -> JsResult<()> {
    register_text_encoder(context)?;
    register_text_decoder(context)?;
    Ok(())
}

// ============================================================================
// TextEncoder Implementation
// ============================================================================

fn register_text_encoder(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(text_encoder_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("TextEncoder"),
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

fn text_encoder_constructor(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let encoder = JsObject::with_null_proto();
    
    // encoding property (always utf-8)
    encoder.define_property_or_throw(
        JsString::from("encoding"),
        PropertyDescriptor::builder()
            .value(JsString::from("utf-8"))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // encode method
    register_method(&encoder, "encode", text_encoder_encode, context)?;
    
    // encodeInto method
    register_method(&encoder, "encodeInto", text_encoder_encode_into, context)?;
    
    Ok(JsValue::from(encoder))
}

fn text_encoder_encode(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let input = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    let bytes = input.as_bytes();
    
    // Create Uint8Array with the encoded bytes
    let typed_array = boa_engine::object::builtins::JsUint8Array::from_iter(
        bytes.iter().copied(),
        context,
    )?;
    
    Ok(JsValue::from(typed_array))
}

fn text_encoder_encode_into(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let input = args.get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    
    let destination = args.get_or_undefined(1);
    
    if destination.is_undefined() {
        return Err(JsNativeError::typ()
            .with_message("encodeInto requires a Uint8Array destination")
            .into());
    }
    
    let bytes = input.as_bytes();
    
    // Get destination array length
    let dest_obj = destination.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Destination must be a Uint8Array"))?;
    
    let dest_len = dest_obj.get(JsString::from("length"), context)?
        .to_length(context)? as usize;
    
    // Calculate how many bytes we can write
    let written = bytes.len().min(dest_len);
    
    // Write bytes to destination
    for (i, &byte) in bytes.iter().take(written).enumerate() {
        dest_obj.set(i as u32, JsValue::from(byte as f64), true, context)?;
    }
    
    // Return result object
    let result = JsObject::with_null_proto();
    
    result.define_property_or_throw(
        JsString::from("read"),
        PropertyDescriptor::builder()
            .value(JsValue::from(input.chars().count() as f64))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    result.define_property_or_throw(
        JsString::from("written"),
        PropertyDescriptor::builder()
            .value(JsValue::from(written as f64))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    Ok(JsValue::from(result))
}

// ============================================================================
// TextDecoder Implementation
// ============================================================================

fn register_text_decoder(context: &mut Context) -> JsResult<()> {
    let constructor = NativeFunction::from_fn_ptr(text_decoder_constructor);
    
    context.global_object().define_property_or_throw(
        JsString::from("TextDecoder"),
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

fn text_decoder_constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Get encoding label (default: utf-8)
    let encoding = if args.is_empty() {
        "utf-8".to_string()
    } else {
        args.get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped()
            .to_lowercase()
    };
    
    // Validate encoding
    let normalized_encoding = normalize_encoding(&encoding);
    if normalized_encoding.is_none() {
        return Err(JsNativeError::range()
            .with_message(format!("The encoding label '{}' is not supported", encoding))
            .into());
    }
    let encoding_name = normalized_encoding.unwrap();
    
    // Get options
    let (fatal, ignore_bom) = if args.len() > 1 {
        let options = args.get_or_undefined(1);
        if let Some(opts) = options.as_object() {
            let fatal = opts.get(JsString::from("fatal"), context)?
                .to_boolean();
            let ignore_bom = opts.get(JsString::from("ignoreBOM"), context)?
                .to_boolean();
            (fatal, ignore_bom)
        } else {
            (false, false)
        }
    } else {
        (false, false)
    };
    
    let decoder = JsObject::with_null_proto();
    
    // encoding property
    decoder.define_property_or_throw(
        JsString::from("encoding"),
        PropertyDescriptor::builder()
            .value(JsString::from(encoding_name.as_str()))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // fatal property
    decoder.define_property_or_throw(
        JsString::from("fatal"),
        PropertyDescriptor::builder()
            .value(JsValue::from(fatal))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // ignoreBOM property
    decoder.define_property_or_throw(
        JsString::from("ignoreBOM"),
        PropertyDescriptor::builder()
            .value(JsValue::from(ignore_bom))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;
    
    // decode method
    register_method(&decoder, "decode", text_decoder_decode, context)?;
    
    Ok(JsValue::from(decoder))
}

fn text_decoder_decode(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Invalid this value"))?;
    
    let encoding = this_obj.get(JsString::from("encoding"), context)?
        .to_string(context)?
        .to_std_string_escaped();
    
    let fatal = this_obj.get(JsString::from("fatal"), context)?
        .to_boolean();
    
    // Get input bytes
    let input = args.get_or_undefined(0);
    
    if input.is_undefined() || input.is_null() {
        return Ok(JsValue::from(JsString::from("")));
    }
    
    let bytes = extract_bytes(input, context)?;
    
    // Decode based on encoding
    let decoded = match encoding.as_str() {
        "utf-8" => decode_utf8(&bytes, fatal)?,
        "utf-16le" => decode_utf16le(&bytes, fatal)?,
        "utf-16be" => decode_utf16be(&bytes, fatal)?,
        "iso-8859-1" | "latin1" => decode_latin1(&bytes),
        "ascii" | "us-ascii" => decode_ascii(&bytes, fatal)?,
        _ => decode_utf8(&bytes, fatal)?, // Default to UTF-8
    };
    
    Ok(JsValue::from(JsString::from(decoded)))
}

// ============================================================================
// Encoding Helpers
// ============================================================================

fn normalize_encoding(label: &str) -> Option<String> {
    let label = label.trim().to_lowercase();
    
    // Map common aliases to canonical names
    match label.as_str() {
        "utf-8" | "utf8" | "unicode-1-1-utf-8" => Some("utf-8".to_string()),
        "utf-16le" | "utf-16" => Some("utf-16le".to_string()),
        "utf-16be" => Some("utf-16be".to_string()),
        "iso-8859-1" | "latin1" | "latin-1" | "iso8859-1" | "windows-1252" | "cp1252" => {
            Some("iso-8859-1".to_string())
        }
        "ascii" | "us-ascii" | "iso-646-us" => Some("ascii".to_string()),
        _ => None,
    }
}

fn extract_bytes(value: &JsValue, context: &mut Context) -> JsResult<Vec<u8>> {
    if let Some(obj) = value.as_object() {
        // Check if it's an ArrayBuffer or TypedArray
        let length = obj.get(JsString::from("length"), context)?
            .to_length(context)? as u32;
        
        let mut bytes = Vec::with_capacity(length as usize);
        for i in 0..length {
            let byte = obj.get(i, context)?
                .to_number(context)? as u8;
            bytes.push(byte);
        }
        
        Ok(bytes)
    } else {
        Ok(Vec::new())
    }
}

fn decode_utf8(bytes: &[u8], fatal: bool) -> JsResult<String> {
    if fatal {
        String::from_utf8(bytes.to_vec())
            .map_err(|e| JsNativeError::typ()
                .with_message(format!("Invalid UTF-8: {}", e))
                .into())
    } else {
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
}

fn decode_utf16le(bytes: &[u8], fatal: bool) -> JsResult<String> {
    if bytes.len() % 2 != 0 && fatal {
        return Err(JsNativeError::typ()
            .with_message("Invalid UTF-16LE: odd number of bytes")
            .into());
    }
    
    let chars: Vec<u16> = bytes.chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                Some(u16::from_le_bytes([chunk[0], chunk[1]]))
            } else {
                None
            }
        })
        .collect();
    
    String::from_utf16(&chars)
        .or_else(|_| {
            if fatal {
                Err(JsNativeError::typ().with_message("Invalid UTF-16LE").into())
            } else {
                Ok(String::from_utf16_lossy(&chars))
            }
        })
}

fn decode_utf16be(bytes: &[u8], fatal: bool) -> JsResult<String> {
    if bytes.len() % 2 != 0 && fatal {
        return Err(JsNativeError::typ()
            .with_message("Invalid UTF-16BE: odd number of bytes")
            .into());
    }
    
    let chars: Vec<u16> = bytes.chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                Some(u16::from_be_bytes([chunk[0], chunk[1]]))
            } else {
                None
            }
        })
        .collect();
    
    String::from_utf16(&chars)
        .or_else(|_| {
            if fatal {
                Err(JsNativeError::typ().with_message("Invalid UTF-16BE").into())
            } else {
                Ok(String::from_utf16_lossy(&chars))
            }
        })
}

fn decode_latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

fn decode_ascii(bytes: &[u8], fatal: bool) -> JsResult<String> {
    if fatal && bytes.iter().any(|&b| b > 127) {
        return Err(JsNativeError::typ()
            .with_message("Invalid ASCII: byte > 127")
            .into());
    }
    
    Ok(bytes.iter().map(|&b| (b & 0x7F) as char).collect())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_encoder_creation() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        // Call TextEncoder() as a factory function
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"TextEncoder().encoding"#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "utf-8");
    }

    #[test]
    fn test_text_decoder_creation() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"TextDecoder().encoding"#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "utf-8");
    }

    #[test]
    fn test_text_decoder_with_encoding() {
        let mut context = Context::default();
        register(&mut context).unwrap();
        
        let result = context.eval(boa_engine::Source::from_bytes(
            r#"TextDecoder('utf-16le').encoding"#
        )).unwrap();
        
        assert_eq!(result.as_string().unwrap().to_std_string_escaped(), "utf-16le");
    }

    #[test]
    fn test_normalize_encoding() {
        assert_eq!(normalize_encoding("UTF-8"), Some("utf-8".to_string()));
        assert_eq!(normalize_encoding("utf8"), Some("utf-8".to_string()));
        assert_eq!(normalize_encoding("latin1"), Some("iso-8859-1".to_string()));
        assert_eq!(normalize_encoding("unknown"), None);
    }

    #[test]
    fn test_decode_utf8() {
        let bytes = "Hello, World!".as_bytes();
        assert_eq!(decode_utf8(bytes, false).unwrap(), "Hello, World!");
    }

    #[test]
    fn test_decode_latin1() {
        let bytes = vec![0xC0, 0xC1, 0xC2]; // Latin-1 characters
        let result = decode_latin1(&bytes);
        // Each byte is a character in Latin-1
        assert_eq!(result.chars().count(), 3);
    }
}
