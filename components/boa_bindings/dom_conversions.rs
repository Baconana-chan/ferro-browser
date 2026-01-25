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
// Phase B2: Enhanced native_from_object with detailed errors
// ============================================================================

/// Detailed error types for native_from_object failures.
#[derive(Debug, Clone, PartialEq)]
pub enum NativeFromObjectError {
    /// The value is not a JavaScript object
    NotAnObject,
    /// The object does not have native data (not a DOM wrapper)
    NoNativeData,
    /// The native data is not of the expected type
    WrongType {
        expected: &'static str,
        actual: &'static str,
    },
    /// The object is from a different realm/global
    CrossRealm,
    /// The object has been detached or is no longer valid
    Detached,
    /// The object is a proxy without unwrappable target
    OpaqueProxy,
}

impl std::fmt::Display for NativeFromObjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NativeFromObjectError::NotAnObject => {
                write!(f, "Value is not an object")
            }
            NativeFromObjectError::NoNativeData => {
                write!(f, "Object does not have native DOM data")
            }
            NativeFromObjectError::WrongType { expected, actual } => {
                write!(f, "Expected {} but got {}", expected, actual)
            }
            NativeFromObjectError::CrossRealm => {
                write!(f, "Cannot unwrap cross-realm object")
            }
            NativeFromObjectError::Detached => {
                write!(f, "Object has been detached")
            }
            NativeFromObjectError::OpaqueProxy => {
                write!(f, "Cannot unwrap opaque proxy object")
            }
        }
    }
}

impl std::error::Error for NativeFromObjectError {}

impl From<NativeFromObjectError> for JsNativeError {
    fn from(err: NativeFromObjectError) -> Self {
        JsNativeError::typ().with_message(err.to_string())
    }
}

/// Result type for native_from_object operations.
pub type NativeFromObjectResult<T> = Result<T, NativeFromObjectError>;

/// Enhanced trait for extracting native DOM objects with detailed errors.
/// 
/// This is the Boa equivalent of SpiderMonkey's native_from_object, but with:
/// - Detailed error types instead of generic TypeError
/// - Support for cross-realm validation
/// - Proper proxy unwrapping
/// 
/// Note: Returns bool for is_instance checks since returning references
/// to borrowed data has lifetime issues with Boa's GC model.
pub trait NativeFromObjectExt: DomObject + Sized + 'static {
    /// Interface name for error messages
    const INTERFACE_NAME: &'static str;
    
    /// Check if a JsObject is a valid wrapper for this type.
    /// 
    /// This is faster than full extraction when you only need to check type.
    fn is_instance(obj: &JsObject) -> bool;
    
    /// Check if a JsValue is a valid wrapper for this type.
    fn is_instance_value(val: &JsValue) -> bool {
        match val.as_object() {
            Some(obj) => Self::is_instance(&obj),
            None => false,
        }
    }
    
    /// Validate the object type and return error if wrong.
    fn validate_object(obj: &JsObject) -> NativeFromObjectResult<()> {
        if Self::is_instance(obj) {
            Ok(())
        } else {
            Err(NativeFromObjectError::WrongType {
                expected: Self::INTERFACE_NAME,
                actual: "unknown",
            })
        }
    }
    
    /// Validate a value, handling non-object case.
    fn validate_value(val: &JsValue) -> NativeFromObjectResult<()> {
        match val.as_object() {
            Some(obj) => Self::validate_object(&obj),
            None => Err(NativeFromObjectError::NotAnObject),
        }
    }
    
    /// Convert validation result to JsResult for error throwing.
    fn validate_object_js(obj: &JsObject) -> JsResult<()> {
        Self::validate_object(obj)
            .map_err(|e| JsNativeError::from(e).into())
    }
    
    /// Create a TypeError for this interface.
    fn type_error(actual: &str) -> JsNativeError {
        type_error_for_interface(Self::INTERFACE_NAME, actual)
    }
}

/// Marker trait for DOM objects that can be extracted across realms.
/// 
/// Some DOM objects (like Window) need special handling when accessed
/// from a different realm. This trait marks objects that support this.
pub trait CrossRealmExtractable: NativeFromObjectExt {
    /// Validate cross-realm access is allowed for this object.
    fn validate_cross_realm(obj: &JsObject) -> NativeFromObjectResult<()>;
}

/// Helper function to create a TypeError with interface name.
pub fn type_error_for_interface(interface: &str, actual: &str) -> JsNativeError {
    JsNativeError::typ().with_message(format!(
        "'this' is not a {} (got {})",
        interface, actual
    ))
}

/// Helper function to check if object is a valid DOM wrapper.
/// 
/// In Boa, DOM objects are stored using NativeFunction or custom object types.
/// This is a heuristic check - actual DOM detection requires checking the
/// object's prototype chain or internal slots.
pub fn is_dom_wrapper(_obj: &JsObject) -> bool {
    // For now, assume any object could be a DOM wrapper
    // Real implementation would check for specific prototype chain
    // or internal slot marker
    true
}

/// Helper to get the interface name of a DOM wrapper.
pub fn dom_wrapper_interface_name(_obj: &JsObject) -> Option<&'static str> {
    // Placeholder - would need actual type metadata
    // In a real implementation, this would read from the NativeObject
    None
}

/// Macro to implement NativeFromObjectExt for a DOM type
#[macro_export]
macro_rules! impl_native_from_object_ext {
    ($type:ty, $interface_name:expr) => {
        impl $crate::dom_conversions::NativeFromObjectExt for $type {
            const INTERFACE_NAME: &'static str = $interface_name;
            
            fn is_instance(obj: &::boa_engine::JsObject) -> bool {
                use ::boa_engine::object::NativeObject;
                obj.borrow().as_any().is::<Self>()
            }
        }
    };
}

#[cfg(test)]
mod native_from_object_tests {
    use super::*;
    
    #[test]
    fn test_native_from_object_error_display() {
        let err = NativeFromObjectError::NotAnObject;
        assert!(err.to_string().contains("not an object"));
        
        let err = NativeFromObjectError::WrongType {
            expected: "HTMLElement",
            actual: "SVGElement",
        };
        assert!(err.to_string().contains("HTMLElement"));
        assert!(err.to_string().contains("SVGElement"));
        
        let err = NativeFromObjectError::CrossRealm;
        assert!(err.to_string().contains("cross-realm"));
    }
    
    #[test]
    fn test_native_from_object_error_to_js() {
        let err = NativeFromObjectError::NoNativeData;
        let js_err: JsNativeError = err.into();
        // Just verify it compiles and converts
        let _ = js_err;
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
