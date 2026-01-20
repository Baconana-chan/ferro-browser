// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Type conversions between Rust and JavaScript values.
//!
//! Boa 0.21 uses NaN-boxing for JsValue, so we use method-based
//! inspection (.is_undefined(), .as_number(), etc.) instead of pattern matching.

use boa_engine::{Context, JsResult, JsValue, JsString, JsNativeError};
use boa_engine::object::builtins::JsArray;

/// Trait for types that can be converted to a JavaScript value.
pub trait ToJsValue {
    fn to_js_value(&self, context: &mut Context) -> JsResult<JsValue>;
}

/// Trait for types that can be created from a JavaScript value.
pub trait FromJsValue: Sized {
    fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self>;
}

// Primitive type implementations

impl ToJsValue for bool {
    fn to_js_value(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJsValue for bool {
    fn from_js_value(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        Ok(value.to_boolean())
    }
}

impl ToJsValue for i32 {
    fn to_js_value(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJsValue for i32 {
    fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        value.to_i32(context)
    }
}

impl ToJsValue for u32 {
    fn to_js_value(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJsValue for u32 {
    fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        value.to_u32(context)
    }
}

impl ToJsValue for f64 {
    fn to_js_value(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJsValue for f64 {
    fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        value.to_number(context)
    }
}

impl ToJsValue for String {
    fn to_js_value(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(JsString::from(self.as_str())))
    }
}

impl FromJsValue for String {
    fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let js_string = value.to_string(context)?;
        Ok(js_string.to_std_string_escaped())
    }
}

impl ToJsValue for &str {
    fn to_js_value(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(JsString::from(*self)))
    }
}

impl<T: ToJsValue> ToJsValue for Option<T> {
    fn to_js_value(&self, context: &mut Context) -> JsResult<JsValue> {
        match self {
            Some(v) => v.to_js_value(context),
            None => Ok(JsValue::null()),
        }
    }
}

impl<T: FromJsValue> FromJsValue for Option<T> {
    fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        if value.is_null() || value.is_undefined() {
            Ok(None)
        } else {
            T::from_js_value(value, context).map(Some)
        }
    }
}

impl<T: ToJsValue> ToJsValue for Vec<T> {
    fn to_js_value(&self, context: &mut Context) -> JsResult<JsValue> {
        let array = JsArray::new(context);
        for (i, item) in self.iter().enumerate() {
            let value = item.to_js_value(context)?;
            array.set(i as u32, value, false, context)?;
        }
        Ok(array.into())
    }
}

impl<T: FromJsValue> FromJsValue for Vec<T> {
    fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let object = value.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Expected an array")
        })?;

        let length = object.get(JsString::from("length"), context)?.to_length(context)?;
        let mut result = Vec::with_capacity(length as usize);

        for i in 0..length {
            let item = object.get(i, context)?;
            result.push(T::from_js_value(&item, context)?);
        }

        Ok(result)
    }
}

/// Convert a JsValue to a debug string representation.
pub fn js_value_to_debug_string(value: &JsValue, context: &mut Context) -> String {
    if value.is_undefined() {
        "undefined".to_string()
    } else if value.is_null() {
        "null".to_string()
    } else if let Some(b) = value.as_boolean() {
        b.to_string()
    } else if let Some(n) = value.as_number() {
        if n.is_nan() {
            "NaN".to_string()
        } else if n.is_infinite() {
            if n > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
        } else {
            n.to_string()
        }
    } else if let Some(s) = value.as_string() {
        format!("\"{}\"", s.to_std_string_escaped())
    } else if value.is_symbol() {
        "[Symbol]".to_string()
    } else if value.is_bigint() {
        value.as_bigint().map(|bi| format!("{}n", bi)).unwrap_or("[BigInt]".to_string())
    } else if value.is_object() {
        value.to_string(context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_else(|_| "[object Object]".to_string())
    } else {
        "[unknown]".to_string()
    }
}

/// DOMString type - represents a JavaScript string in DOM contexts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DOMString(pub String);

impl DOMString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
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

impl ToJsValue for DOMString {
    fn to_js_value(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(JsString::from(self.0.as_str())))
    }
}

impl FromJsValue for DOMString {
    fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let js_string = value.to_string(context)?;
        Ok(DOMString(js_string.to_std_string_escaped()))
    }
}

impl From<String> for DOMString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DOMString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for DOMString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
