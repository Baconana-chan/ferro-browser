// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// DOM-specific conversion traits for Boa engine
// These traits provide conversions between DOM objects and JavaScript values

use boa_engine::{Context, JsResult, JsValue, JsObject, JsNativeError};
use boa_gc::{Trace, Finalize};

use crate::reflector::{DomObject, Reflector};
use crate::root::{Dom, DomRoot};

/// Result type for conversion operations.
#[derive(Debug)]
pub enum ConversionResult<T> {
    /// Conversion succeeded.
    Success(T),
    /// Conversion failed with an error message.
    Failure(String),
}

impl<T> ConversionResult<T> {
    /// Returns true if the conversion succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, ConversionResult::Success(_))
    }
    
    /// Unwraps the successful value, panicking on failure.
    pub fn unwrap(self) -> T {
        match self {
            ConversionResult::Success(v) => v,
            ConversionResult::Failure(msg) => panic!("Conversion failed: {}", msg),
        }
    }
    
    /// Converts to a JsResult.
    pub fn to_js_result(self) -> JsResult<T> {
        match self {
            ConversionResult::Success(v) => Ok(v),
            ConversionResult::Failure(msg) => {
                Err(JsNativeError::typ().with_message(msg).into())
            }
        }
    }
}

/// Behavior for stringification of null/undefined values.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StringificationBehavior {
    /// Convert null/undefined to the string "null"/"undefined".
    Default,
    /// Convert null/undefined to the empty string.
    Empty,
}

impl Default for StringificationBehavior {
    fn default() -> Self {
        Self::Default
    }
}

/// A trait to check whether a given object implements an IDL interface.
pub trait IDLInterface {
    /// The interface name for error messages.
    const NAME: &'static str;
    
    /// Returns whether the given object implements this interface.
    fn is_instance(obj: &JsObject, context: &mut Context) -> bool;
}

/// Safe wrapper for converting Rust values to JavaScript values.
/// This is the Boa equivalent of js::conversions::ToJSValConvertible.
pub trait ToJSValConvertible {
    /// Convert this value to a JavaScript value.
    fn to_jsval(&self, context: &mut Context) -> JsResult<JsValue>;
}

/// Safe wrapper for converting JavaScript values to Rust values.
/// This is the Boa equivalent of js::conversions::FromJSValConvertible.
pub trait FromJSValConvertible: Sized {
    /// Configuration for the conversion (e.g., StringificationBehavior).
    type Config: Default;
    
    /// Convert a JavaScript value to this type.
    fn from_jsval(
        context: &mut Context,
        value: &JsValue,
        config: Self::Config,
    ) -> ConversionResult<Self>;
}

// ============================================================================
// Implementation for DOM objects
// ============================================================================

impl<T: DomObject + Trace + Finalize + 'static> ToJSValConvertible for DomRoot<T> {
    fn to_jsval(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(self.reflector().get_jsval())
    }
}

impl<T: DomObject + Trace + Finalize + 'static> ToJSValConvertible for Dom<T> {
    fn to_jsval(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(self.reflector().get_jsval())
    }
}

/// Trait for extracting a native DOM object from a JavaScript object.
/// This is the Boa equivalent of native_from_object in script_bindings.
pub trait NativeFromObject<'a, T: DomObject>: Sized {
    /// Extract the native object from a JavaScript object.
    fn native_from_object(obj: &'a JsObject, context: &'a mut Context) -> JsResult<&'a T>;
    
    /// Extract the native object, returning None if conversion fails.
    fn try_native_from_object(obj: &'a JsObject, context: &'a mut Context) -> Option<&'a T> {
        Self::native_from_object(obj, context).ok()
    }
}

// ============================================================================
// Implementation for primitive types
// ============================================================================

impl ToJSValConvertible for bool {
    fn to_jsval(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJSValConvertible for bool {
    type Config = ();
    
    fn from_jsval(
        _context: &mut Context,
        value: &JsValue,
        _config: (),
    ) -> ConversionResult<Self> {
        ConversionResult::Success(value.to_boolean())
    }
}

impl ToJSValConvertible for i32 {
    fn to_jsval(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJSValConvertible for i32 {
    type Config = ();
    
    fn from_jsval(
        context: &mut Context,
        value: &JsValue,
        _config: (),
    ) -> ConversionResult<Self> {
        match value.to_i32(context) {
            Ok(n) => ConversionResult::Success(n),
            Err(e) => ConversionResult::Failure(format!("Failed to convert to i32: {:?}", e)),
        }
    }
}

impl ToJSValConvertible for u32 {
    fn to_jsval(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJSValConvertible for u32 {
    type Config = ();
    
    fn from_jsval(
        context: &mut Context,
        value: &JsValue,
        _config: (),
    ) -> ConversionResult<Self> {
        match value.to_u32(context) {
            Ok(n) => ConversionResult::Success(n),
            Err(e) => ConversionResult::Failure(format!("Failed to convert to u32: {:?}", e)),
        }
    }
}

impl ToJSValConvertible for f64 {
    fn to_jsval(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(*self))
    }
}

impl FromJSValConvertible for f64 {
    type Config = ();
    
    fn from_jsval(
        context: &mut Context,
        value: &JsValue,
        _config: (),
    ) -> ConversionResult<Self> {
        match value.to_number(context) {
            Ok(n) => ConversionResult::Success(n),
            Err(e) => ConversionResult::Failure(format!("Failed to convert to f64: {:?}", e)),
        }
    }
}

impl ToJSValConvertible for String {
    fn to_jsval(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(boa_engine::JsString::from(self.as_str())))
    }
}

impl FromJSValConvertible for String {
    type Config = StringificationBehavior;
    
    fn from_jsval(
        context: &mut Context,
        value: &JsValue,
        config: StringificationBehavior,
    ) -> ConversionResult<Self> {
        if config == StringificationBehavior::Empty {
            if value.is_null() || value.is_undefined() {
                return ConversionResult::Success(String::new());
            }
        }
        
        match value.to_string(context) {
            Ok(s) => ConversionResult::Success(s.to_std_string_escaped()),
            Err(e) => ConversionResult::Failure(format!("Failed to convert to String: {:?}", e)),
        }
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<T> {
    fn to_jsval(&self, context: &mut Context) -> JsResult<JsValue> {
        match self {
            Some(v) => v.to_jsval(context),
            None => Ok(JsValue::null()),
        }
    }
}

impl<T: FromJSValConvertible> FromJSValConvertible for Option<T> {
    type Config = T::Config;
    
    fn from_jsval(
        context: &mut Context,
        value: &JsValue,
        config: Self::Config,
    ) -> ConversionResult<Self> {
        if value.is_null() || value.is_undefined() {
            ConversionResult::Success(None)
        } else {
            match T::from_jsval(context, value, config) {
                ConversionResult::Success(v) => ConversionResult::Success(Some(v)),
                ConversionResult::Failure(e) => ConversionResult::Failure(e),
            }
        }
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Vec<T> {
    fn to_jsval(&self, context: &mut Context) -> JsResult<JsValue> {
        use boa_engine::object::builtins::JsArray;
        
        let array = JsArray::new(context);
        for (i, item) in self.iter().enumerate() {
            let js_val = item.to_jsval(context)?;
            array.set(i as u32, js_val, false, context)?;
        }
        Ok(array.into())
    }
}

impl ToJSValConvertible for () {
    fn to_jsval(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }
}

impl FromJSValConvertible for () {
    type Config = ();
    
    fn from_jsval(
        _context: &mut Context,
        _value: &JsValue,
        _config: (),
    ) -> ConversionResult<Self> {
        ConversionResult::Success(())
    }
}

impl ToJSValConvertible for JsValue {
    fn to_jsval(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(self.clone())
    }
}

impl FromJSValConvertible for JsValue {
    type Config = ();
    
    fn from_jsval(
        _context: &mut Context,
        value: &JsValue,
        _config: (),
    ) -> ConversionResult<Self> {
        ConversionResult::Success(value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_result() {
        let success: ConversionResult<i32> = ConversionResult::Success(42);
        assert!(success.is_success());
        assert_eq!(success.unwrap(), 42);
        
        let failure: ConversionResult<i32> = ConversionResult::Failure("test".to_string());
        assert!(!failure.is_success());
    }
}
