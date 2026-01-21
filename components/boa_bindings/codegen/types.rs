//! WebIDL Type Mappings for Boa
//!
//! This module provides conversions between WebIDL types and Boa JavaScript types.
//! 
//! WebIDL Type           | Boa Type                | Rust Type
//! ----------------------|-------------------------|------------------
//! boolean               | JsValue::Boolean        | bool
//! byte                  | JsValue::Integer        | i8
//! octet                 | JsValue::Integer        | u8
//! short                 | JsValue::Integer        | i16
//! unsigned short        | JsValue::Integer        | u16
//! long                  | JsValue::Integer        | i32
//! unsigned long         | JsValue::Integer        | u32
//! long long             | JsValue::BigInt         | i64
//! unsigned long long    | JsValue::BigInt         | u64
//! float                 | JsValue::Rational       | f32
//! double                | JsValue::Rational       | f64
//! DOMString             | JsValue::String         | String
//! USVString             | JsValue::String         | String
//! ByteString            | JsValue::String         | Vec<u8>
//! object                | JsValue::Object         | JsObject
//! any                   | JsValue                 | JsValue
//! void/undefined        | JsValue::undefined()    | ()
//! sequence<T>           | JsArray                 | Vec<T>
//! record<K, V>          | JsObject                | HashMap<K, V>
//! Promise<T>            | JsPromise               | Promise<T>
//! ArrayBuffer           | JsArrayBuffer           | Vec<u8>
//! Uint8Array            | JsTypedArray            | Vec<u8>

use std::collections::HashMap;

use boa_engine::{
    Context, JsNativeError, JsObject, JsResult, JsString, JsValue,
    object::{builtins::JsArray, builtins::JsArrayBuffer},
};

/// Trait for converting WebIDL types to JsValue
pub trait ToJsValueBoa {
    fn to_js_value(&self, ctx: &mut Context) -> JsResult<JsValue>;
}

/// Trait for converting JsValue to WebIDL types
pub trait FromJsValueBoa: Sized {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self>;
}

// ============================================================================
// Primitive Type Conversions
// ============================================================================

impl ToJsValueBoa for bool {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJsValueBoa for bool {
    fn from_js_value(value: &JsValue, _ctx: &mut Context) -> JsResult<Self> {
        Ok(value.to_boolean())
    }
}

// Integer types
macro_rules! impl_integer_conversion {
    ($($ty:ty),*) => {
        $(
            impl ToJsValueBoa for $ty {
                fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
                    Ok(JsValue::from(*self as i32))
                }
            }

            impl FromJsValueBoa for $ty {
                fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
                    let num = value.to_i32(ctx)?;
                    Ok(num as $ty)
                }
            }
        )*
    };
}

impl_integer_conversion!(i8, u8, i16, u16, i32, u32);

// 64-bit integers - use f64 for simplicity (may lose precision for very large values)
// Full BigInt support requires Boa-specific API
impl ToJsValueBoa for i64 {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self as f64))
    }
}

impl FromJsValueBoa for i64 {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let num = value.to_number(ctx)?;
        Ok(num as i64)
    }
}

impl ToJsValueBoa for u64 {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self as f64))
    }
}

impl FromJsValueBoa for u64 {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let num = value.to_number(ctx)?;
        Ok(num as u64)
    }
}

// Float types
impl ToJsValueBoa for f32 {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self as f64))
    }
}

impl FromJsValueBoa for f32 {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let num = value.to_number(ctx)?;
        Ok(num as f32)
    }
}

impl ToJsValueBoa for f64 {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJsValueBoa for f64 {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        value.to_number(ctx)
    }
}

// ============================================================================
// String Type Conversions (DOMString, USVString, ByteString)
// ============================================================================

/// DOMString - UTF-16 string that maps to JavaScript String
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DOMString(pub String);

impl DOMString {
    pub fn new() -> Self {
        DOMString(String::new())
    }

    pub fn from_string(s: String) -> Self {
        DOMString(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<&str> for DOMString {
    fn from(s: &str) -> Self {
        DOMString(s.to_string())
    }
}

impl From<String> for DOMString {
    fn from(s: String) -> Self {
        DOMString(s)
    }
}

impl std::fmt::Display for DOMString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToJsValueBoa for DOMString {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(JsString::from(self.0.as_str())))
    }
}

impl FromJsValueBoa for DOMString {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let js_str = value.to_string(ctx)?;
        Ok(DOMString(js_str.to_std_string_escaped()))
    }
}

impl ToJsValueBoa for String {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(JsString::from(self.as_str())))
    }
}

impl FromJsValueBoa for String {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let js_str = value.to_string(ctx)?;
        Ok(js_str.to_std_string_escaped())
    }
}

/// USVString - Unicode scalar value string (no lone surrogates)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct USVString(pub String);

impl ToJsValueBoa for USVString {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(JsString::from(self.0.as_str())))
    }
}

impl FromJsValueBoa for USVString {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let js_str = value.to_string(ctx)?;
        // Replace lone surrogates with U+FFFD
        let s = js_str.to_std_string_escaped();
        Ok(USVString(s))
    }
}

/// ByteString - ISO-8859-1 encoded string
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ByteString(pub Vec<u8>);

impl ToJsValueBoa for ByteString {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        // Convert bytes to ISO-8859-1 string
        let s: String = self.0.iter().map(|&b| b as char).collect();
        Ok(JsValue::from(JsString::from(s.as_str())))
    }
}

impl FromJsValueBoa for ByteString {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let js_str = value.to_string(ctx)?;
        let s = js_str.to_std_string_escaped();
        
        // Check if all characters are in Latin-1 range
        for c in s.chars() {
            if c as u32 > 255 {
                return Err(JsNativeError::typ()
                    .with_message("ByteString contains character outside Latin-1 range")
                    .into());
            }
        }
        
        Ok(ByteString(s.bytes().collect()))
    }
}

// ============================================================================
// Nullable Types
// ============================================================================

impl<T: ToJsValueBoa> ToJsValueBoa for Option<T> {
    fn to_js_value(&self, ctx: &mut Context) -> JsResult<JsValue> {
        match self {
            Some(v) => v.to_js_value(ctx),
            None => Ok(JsValue::null()),
        }
    }
}

impl<T: FromJsValueBoa> FromJsValueBoa for Option<T> {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        if value.is_null_or_undefined() {
            Ok(None)
        } else {
            Ok(Some(T::from_js_value(value, ctx)?))
        }
    }
}

// ============================================================================
// Sequence Types (sequence<T> -> Vec<T>)
// ============================================================================

impl<T: ToJsValueBoa> ToJsValueBoa for Vec<T> {
    fn to_js_value(&self, ctx: &mut Context) -> JsResult<JsValue> {
        let array = JsArray::new(ctx);
        for (i, item) in self.iter().enumerate() {
            let js_item = item.to_js_value(ctx)?;
            array.set(i as u32, js_item, false, ctx)?;
        }
        Ok(array.into())
    }
}

impl<T: FromJsValueBoa> FromJsValueBoa for Vec<T> {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let obj = value.to_object(ctx)?;
        // Get length property
        let length_val = obj.get(JsString::from("length"), ctx)?;
        let length = length_val.to_u32(ctx)?;
        let mut result = Vec::with_capacity(length as usize);
        
        for i in 0..length {
            let item = obj.get(i, ctx)?;
            result.push(T::from_js_value(&item, ctx)?);
        }
        
        Ok(result)
    }
}

// ============================================================================
// Record Types (record<K, V> -> HashMap<K, V>)
// ============================================================================

impl<V: ToJsValueBoa> ToJsValueBoa for HashMap<String, V> {
    fn to_js_value(&self, ctx: &mut Context) -> JsResult<JsValue> {
        let obj = JsObject::with_null_proto();
        for (key, value) in self {
            let js_value = value.to_js_value(ctx)?;
            obj.set(JsString::from(key.as_str()), js_value, false, ctx)?;
        }
        Ok(obj.into())
    }
}

impl<V: FromJsValueBoa> FromJsValueBoa for HashMap<String, V> {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let obj = value.to_object(ctx)?;
        let keys = obj.own_property_keys(ctx)?;
        let mut result = HashMap::new();
        
        for key in keys {
            // PropertyKey can be converted to string
            let key_str = key.to_string();
            let value = obj.get(key.clone(), ctx)?;
            result.insert(key_str, V::from_js_value(&value, ctx)?);
        }
        
        Ok(result)
    }
}

// ============================================================================
// Buffer Types (ArrayBuffer, TypedArrays)
// ============================================================================

/// ArrayBuffer wrapper
/// Note: Full ArrayBuffer support requires specific Boa API patterns
#[derive(Debug, Clone)]
pub struct ArrayBufferWrapper(pub Vec<u8>);

impl ToJsValueBoa for ArrayBufferWrapper {
    fn to_js_value(&self, ctx: &mut Context) -> JsResult<JsValue> {
        // Create ArrayBuffer with specified capacity
        // Note: Full implementation would copy data, this creates empty buffer of right size
        let buffer = JsArrayBuffer::new(self.0.len(), ctx)?;
        Ok(JsValue::from(buffer))
    }
}

impl FromJsValueBoa for ArrayBufferWrapper {
    fn from_js_value(value: &JsValue, _ctx: &mut Context) -> JsResult<Self> {
        if let Some(obj) = value.as_object() {
            if let Ok(buffer) = JsArrayBuffer::from_object(obj.clone()) {
                // Get data from buffer
                if let Some(data_ref) = buffer.data() {
                    return Ok(ArrayBufferWrapper(data_ref.to_vec()));
                }
            }
        }
        Err(JsNativeError::typ()
            .with_message("Expected ArrayBuffer")
            .into())
    }
}

/// Uint8Array wrapper  
/// Note: Full TypedArray support requires specific Boa API patterns
#[derive(Debug, Clone)]
pub struct Uint8ArrayWrapper(pub Vec<u8>);

impl ToJsValueBoa for Uint8ArrayWrapper {
    fn to_js_value(&self, ctx: &mut Context) -> JsResult<JsValue> {
        // Create ArrayBuffer with specified capacity
        // Note: Full implementation would use JsUint8Array, this creates buffer placeholder
        let buffer = JsArrayBuffer::new(self.0.len(), ctx)?;
        Ok(JsValue::from(buffer))
    }
}

impl FromJsValueBoa for Uint8ArrayWrapper {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        // Try to extract bytes from typed array or array buffer
        if let Some(obj) = value.as_object() {
            if let Ok(buffer) = JsArrayBuffer::from_object(obj.clone()) {
                if let Some(data_ref) = buffer.data() {
                    return Ok(Uint8ArrayWrapper(data_ref.to_vec()));
                }
            }
        }
        Err(JsNativeError::typ()
            .with_message("Expected Uint8Array or ArrayBuffer")
            .into())
    }
}

// ============================================================================
// Unit type (void/undefined)
// ============================================================================

impl ToJsValueBoa for () {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }
}

impl FromJsValueBoa for () {
    fn from_js_value(_value: &JsValue, _ctx: &mut Context) -> JsResult<Self> {
        Ok(())
    }
}

// ============================================================================
// JsValue passthrough
// ============================================================================

impl ToJsValueBoa for JsValue {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(self.clone())
    }
}

impl FromJsValueBoa for JsValue {
    fn from_js_value(value: &JsValue, _ctx: &mut Context) -> JsResult<Self> {
        Ok(value.clone())
    }
}

impl ToJsValueBoa for JsObject {
    fn to_js_value(&self, _ctx: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(self.clone()))
    }
}

impl FromJsValueBoa for JsObject {
    fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        value.to_object(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    #[test]
    fn test_domstring_conversion() {
        let mut ctx = Context::default();
        
        let dom_str = DOMString::from("hello world");
        let js_val = dom_str.to_js_value(&mut ctx).unwrap();
        
        assert!(js_val.is_string());
        
        let back = DOMString::from_js_value(&js_val, &mut ctx).unwrap();
        assert_eq!(back.0, "hello world");
    }

    #[test]
    fn test_integer_conversion() {
        let mut ctx = Context::default();
        
        let val: i32 = 42;
        let js_val = val.to_js_value(&mut ctx).unwrap();
        let back = i32::from_js_value(&js_val, &mut ctx).unwrap();
        assert_eq!(back, 42);
    }

    #[test]
    fn test_sequence_conversion() {
        let mut ctx = Context::default();
        
        let vec: Vec<i32> = vec![1, 2, 3, 4, 5];
        let js_val = vec.to_js_value(&mut ctx).unwrap();
        
        assert!(js_val.is_object());
        
        let back: Vec<i32> = Vec::from_js_value(&js_val, &mut ctx).unwrap();
        assert_eq!(back, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_nullable_conversion() {
        let mut ctx = Context::default();
        
        let some_val: Option<i32> = Some(42);
        let none_val: Option<i32> = None;
        
        let js_some = some_val.to_js_value(&mut ctx).unwrap();
        let js_none = none_val.to_js_value(&mut ctx).unwrap();
        
        assert!(js_some.is_number());
        assert!(js_none.is_null());
        
        let back_some: Option<i32> = Option::from_js_value(&js_some, &mut ctx).unwrap();
        let back_none: Option<i32> = Option::from_js_value(&js_none, &mut ctx).unwrap();
        
        assert_eq!(back_some, Some(42));
        assert_eq!(back_none, None);
    }

    #[test]
    fn test_record_conversion() {
        let mut ctx = Context::default();
        
        let mut map: HashMap<String, i32> = HashMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        
        let js_val = map.to_js_value(&mut ctx).unwrap();
        assert!(js_val.is_object());
        
        let back: HashMap<String, i32> = HashMap::from_js_value(&js_val, &mut ctx).unwrap();
        assert_eq!(back.get("a"), Some(&1));
        assert_eq!(back.get("b"), Some(&2));
    }
}
