//! WebIDL Interface Generator for Boa
//!
//! This module provides the infrastructure for generating JavaScript bindings
//! from WebIDL interface definitions.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue,
};

use crate::codegen::types::FromJsValueBoa;

// Note: The unwrap_dom_object, define_getter, and define_setter macros have been
// removed because Boa 0.21 uses a different pattern for native object storage.
// Future implementations should use Boa's NativeData trait pattern.

/// Helper to check and convert arguments
pub fn check_args_length(
    args: &[JsValue],
    expected: usize,
    method_name: &str,
) -> JsResult<()> {
    if args.len() < expected {
        return Err(JsNativeError::typ()
            .with_message(format!(
                "{} requires at least {} argument(s), but only {} were passed",
                method_name,
                expected,
                args.len()
            ))
            .into());
    }
    Ok(())
}

/// Helper to get argument at index with type conversion
pub fn get_arg<T: FromJsValueBoa>(
    args: &[JsValue],
    index: usize,
    ctx: &mut Context,
) -> JsResult<T> {
    T::from_js_value(args.get_or_undefined(index), ctx)
}

/// Helper to get optional argument at index
pub fn get_optional_arg<T: FromJsValueBoa>(
    args: &[JsValue],
    index: usize,
    ctx: &mut Context,
) -> JsResult<Option<T>> {
    let value = args.get(index);
    match value {
        Some(v) if !v.is_undefined() => Ok(Some(T::from_js_value(v, ctx)?)),
        _ => Ok(None),
    }
}

/// Helper to get argument with default value
pub fn get_arg_with_default<T: FromJsValueBoa + Clone>(
    args: &[JsValue],
    index: usize,
    default: T,
    ctx: &mut Context,
) -> JsResult<T> {
    let value = args.get(index);
    match value {
        Some(v) if !v.is_undefined() => T::from_js_value(v, ctx),
        _ => Ok(default),
    }
}

/// Standard WebIDL error types
pub mod exceptions {
    use boa_engine::{JsError, JsNativeError};

    /// DOMException types as defined in WebIDL
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u16)]
    pub enum DOMExceptionCode {
        IndexSizeError = 1,
        HierarchyRequestError = 3,
        WrongDocumentError = 4,
        InvalidCharacterError = 5,
        NoModificationAllowedError = 7,
        NotFoundError = 8,
        NotSupportedError = 9,
        InUseAttributeError = 10,
        InvalidStateError = 11,
        SyntaxError = 12,
        InvalidModificationError = 13,
        NamespaceError = 14,
        InvalidAccessError = 15,
        TypeMismatchError = 17,
        SecurityError = 18,
        NetworkError = 19,
        AbortError = 20,
        URLMismatchError = 21,
        QuotaExceededError = 22,
        TimeoutError = 23,
        InvalidNodeTypeError = 24,
        DataCloneError = 25,
    }

    impl DOMExceptionCode {
        pub fn name(&self) -> &'static str {
            match self {
                Self::IndexSizeError => "IndexSizeError",
                Self::HierarchyRequestError => "HierarchyRequestError",
                Self::WrongDocumentError => "WrongDocumentError",
                Self::InvalidCharacterError => "InvalidCharacterError",
                Self::NoModificationAllowedError => "NoModificationAllowedError",
                Self::NotFoundError => "NotFoundError",
                Self::NotSupportedError => "NotSupportedError",
                Self::InUseAttributeError => "InUseAttributeError",
                Self::InvalidStateError => "InvalidStateError",
                Self::SyntaxError => "SyntaxError",
                Self::InvalidModificationError => "InvalidModificationError",
                Self::NamespaceError => "NamespaceError",
                Self::InvalidAccessError => "InvalidAccessError",
                Self::TypeMismatchError => "TypeMismatchError",
                Self::SecurityError => "SecurityError",
                Self::NetworkError => "NetworkError",
                Self::AbortError => "AbortError",
                Self::URLMismatchError => "URLMismatchError",
                Self::QuotaExceededError => "QuotaExceededError",
                Self::TimeoutError => "TimeoutError",
                Self::InvalidNodeTypeError => "InvalidNodeTypeError",
                Self::DataCloneError => "DataCloneError",
            }
        }
    }

    /// Create a DOMException error
    pub fn dom_exception(code: DOMExceptionCode, message: &str) -> JsError {
        JsNativeError::error()
            .with_message(format!("{}: {}", code.name(), message))
            .into()
    }

    /// Create a TypeError
    pub fn type_error(message: &str) -> JsError {
        JsNativeError::typ().with_message(message.to_string()).into()
    }

    /// Create a RangeError
    pub fn range_error(message: &str) -> JsError {
        JsNativeError::range().with_message(message.to_string()).into()
    }

    /// Create a SyntaxError
    pub fn syntax_error(message: &str) -> JsError {
        JsNativeError::syntax().with_message(message.to_string()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::types::DOMString;

    #[test]
    fn test_check_args_length() {
        let args: Vec<JsValue> = vec![JsValue::from(1), JsValue::from(2)];
        
        assert!(check_args_length(&args, 2, "test").is_ok());
        assert!(check_args_length(&args, 3, "test").is_err());
    }

    #[test]
    fn test_get_arg() {
        let mut ctx = Context::default();
        let args: Vec<JsValue> = vec![JsValue::from(42)];
        
        let result: i32 = get_arg(&args, 0, &mut ctx).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_get_optional_arg() {
        let mut ctx = Context::default();
        let args: Vec<JsValue> = vec![JsValue::from(42)];
        
        let result: Option<i32> = get_optional_arg(&args, 0, &mut ctx).unwrap();
        assert_eq!(result, Some(42));
        
        let result2: Option<i32> = get_optional_arg(&args, 1, &mut ctx).unwrap();
        assert_eq!(result2, None);
    }

    #[test]
    fn test_dom_exception_codes() {
        use exceptions::*;
        
        assert_eq!(DOMExceptionCode::NotFoundError.name(), "NotFoundError");
        assert_eq!(DOMExceptionCode::InvalidStateError as u16, 11);
    }
}
