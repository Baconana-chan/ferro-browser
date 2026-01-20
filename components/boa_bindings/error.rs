// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! JavaScript error handling for Boa 0.21.

use boa_engine::{JsError, JsNativeError, JsValue, Context};
use std::fmt;

/// A JavaScript exception that occurred during script execution.
#[derive(Debug, Clone)]
pub struct JsException {
    /// The error message.
    pub message: String,
    /// The error name (e.g., "TypeError", "ReferenceError").
    pub name: String,
    /// The JavaScript stack trace, if available.
    pub stack: Option<String>,
    /// The source file where the error occurred.
    pub filename: Option<String>,
    /// The line number where the error occurred.
    pub line: Option<u32>,
    /// The column number where the error occurred.
    pub column: Option<u32>,
}

impl JsException {
    /// Create a new JsException from a Boa JsError.
    pub fn from_boa_error(error: &JsError, _context: &mut Context) -> Self {
        let message = error.to_string();

        // Determine error type from message
        let name = if message.starts_with("TypeError") {
            "TypeError"
        } else if message.starts_with("ReferenceError") {
            "ReferenceError"
        } else if message.starts_with("SyntaxError") {
            "SyntaxError"
        } else if message.starts_with("RangeError") {
            "RangeError"
        } else if message.starts_with("EvalError") {
            "EvalError"
        } else if message.starts_with("URIError") {
            "URIError"
        } else {
            "Error"
        };

        Self {
            message,
            name: name.to_string(),
            stack: None,
            filename: None,
            line: None,
            column: None,
        }
    }

    /// Create a generic error with just a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            name: "Error".to_string(),
            stack: None,
            filename: None,
            line: None,
            column: None,
        }
    }

    /// Create a TypeError.
    pub fn type_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            name: "TypeError".to_string(),
            stack: None,
            filename: None,
            line: None,
            column: None,
        }
    }

    /// Create a ReferenceError.
    pub fn reference_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            name: "ReferenceError".to_string(),
            stack: None,
            filename: None,
            line: None,
            column: None,
        }
    }

    /// Create a SyntaxError.
    pub fn syntax_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            name: "SyntaxError".to_string(),
            stack: None,
            filename: None,
            line: None,
            column: None,
        }
    }

    /// Create a RangeError.
    pub fn range_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            name: "RangeError".to_string(),
            stack: None,
            filename: None,
            line: None,
            column: None,
        }
    }

    /// Convert to a Boa JsError.
    pub fn to_boa_error(&self) -> JsError {
        let msg = self.message.clone();
        let native = match self.name.as_str() {
            "TypeError" => JsNativeError::typ().with_message(msg),
            "ReferenceError" => JsNativeError::reference().with_message(msg),
            "SyntaxError" => JsNativeError::syntax().with_message(msg),
            "RangeError" => JsNativeError::range().with_message(msg),
            "EvalError" => JsNativeError::eval().with_message(msg),
            "URIError" => JsNativeError::uri().with_message(msg),
            _ => JsNativeError::error().with_message(msg),
        };
        native.into()
    }
}

impl fmt::Display for JsException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.message)?;
        if let Some(ref filename) = self.filename {
            write!(f, " at {}", filename)?;
            if let Some(line) = self.line {
                write!(f, ":{}", line)?;
                if let Some(col) = self.column {
                    write!(f, ":{}", col)?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for JsException {}

/// Result type for JavaScript operations.
pub type JsExceptionResult<T> = Result<T, JsException>;

/// Throw a JavaScript exception from native code.
pub fn throw_error(message: impl Into<String>) -> JsError {
    JsNativeError::error().with_message(message.into()).into()
}

/// Throw a TypeError from native code.
pub fn throw_type_error(message: impl Into<String>) -> JsError {
    JsNativeError::typ().with_message(message.into()).into()
}

/// Throw a ReferenceError from native code.
pub fn throw_reference_error(message: impl Into<String>) -> JsError {
    JsNativeError::reference().with_message(message.into()).into()
}

/// Throw a SyntaxError from native code.
pub fn throw_syntax_error(message: impl Into<String>) -> JsError {
    JsNativeError::syntax().with_message(message.into()).into()
}

/// Throw a RangeError from native code.
pub fn throw_range_error(message: impl Into<String>) -> JsError {
    JsNativeError::range().with_message(message.into()).into()
}

/// Report a JavaScript error to the console.
pub fn report_error(exception: &JsException) {
    log::error!("[JavaScript {}] {}", exception.name, exception.message);
    if let Some(ref stack) = exception.stack {
        for line in stack.lines() {
            log::error!("    {}", line);
        }
    }
}

/// Check if a JsValue is an error object.
pub fn is_error(value: &JsValue) -> bool {
    if let Some(obj) = value.as_object() {
        // Check for stack property as indicator of Error
        obj.has_own_property(
            boa_engine::JsString::from("stack"),
            &mut Context::default()
        ).unwrap_or(false)
    } else {
        false
    }
}
