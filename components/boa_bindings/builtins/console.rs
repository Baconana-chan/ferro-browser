// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Console API extensions for Ferro.
//!
//! boa_runtime provides the standard Console, this adds Ferro-specific methods.

use boa_engine::{Context, JsResult, JsValue, NativeFunction};

/// Register custom console extensions.
pub fn register(context: &mut Context) -> JsResult<()> {
    // Get existing console object
    let console = context.global_object().get(
        boa_engine::JsString::from("console"),
        context
    )?;

    if let Some(console_obj) = console.as_object() {
        // Add console.ferro() for Ferro-specific logging
        let ferro_log = NativeFunction::from_fn_ptr(|_this, args, context| {
            let message = if !args.is_empty() {
                args[0].to_string(context)?.to_std_string_escaped()
            } else {
                String::new()
            };
            log::info!("[Ferro Console] {}", message);
            Ok(JsValue::undefined())
        });

        console_obj.define_property_or_throw(
            boa_engine::JsString::from("ferro"),
            boa_engine::property::PropertyDescriptor::builder()
                .value(ferro_log.to_js_function(context.realm()))
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;
    }

    Ok(())
}

/// Format JavaScript values for console output.
pub fn format_console_args(args: &[JsValue], context: &mut Context) -> String {
    args.iter()
        .map(|v| format_js_value(v, context))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format a single JavaScript value for display.
fn format_js_value(value: &JsValue, context: &mut Context) -> String {
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
        s.to_std_string_escaped()
    } else if value.is_symbol() {
        if let Some(sym) = value.as_symbol() {
            format!("Symbol({})",
                sym.description()
                    .map(|d| d.to_std_string_escaped())
                    .unwrap_or_default()
            )
        } else {
            "Symbol()".to_string()
        }
    } else if value.is_bigint() {
        value.as_bigint()
            .map(|bi| format!("{}n", bi))
            .unwrap_or("[BigInt]".to_string())
    } else if let Some(obj) = value.as_object() {
        if obj.is_array() {
            format_array(&obj, context)
        } else if obj.is_callable() {
            "[Function]".to_string()
        } else {
            value.to_string(context)
                .map(|s| s.to_std_string_escaped())
                .unwrap_or("[object Object]".to_string())
        }
    } else {
        "[unknown]".to_string()
    }
}

/// Format an array for display.
fn format_array(obj: &boa_engine::JsObject, context: &mut Context) -> String {
    let length = obj.get(boa_engine::JsString::from("length"), context)
        .and_then(|v| v.to_length(context))
        .unwrap_or(0);

    let mut items = Vec::new();
    for i in 0..length.min(10) {
        if let Ok(item) = obj.get(i, context) {
            items.push(format_js_value(&item, context));
        }
    }
    if length > 10 {
        items.push(format!("... {} more", length - 10));
    }
    format!("[{}]", items.join(", "))
}

// ============================================================================
// Phase C3: Error Stack Trace Formatting
// ============================================================================

use boa_engine::JsError;

/// A single frame in a JavaScript stack trace.
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function name (empty for anonymous functions)
    pub function_name: String,
    /// Script/file name
    pub file_name: String,
    /// Line number (1-indexed)
    pub line_number: u32,
    /// Column number (1-indexed)
    pub column_number: u32,
}

impl std::fmt::Display for StackFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let func = if self.function_name.is_empty() {
            "<anonymous>".to_string()
        } else {
            self.function_name.clone()
        };
        write!(f, "    at {} ({}:{}:{})", 
            func, self.file_name, self.line_number, self.column_number)
    }
}

/// A formatted JavaScript stack trace.
#[derive(Debug, Clone)]
pub struct StackTrace {
    /// The error message
    pub message: String,
    /// Stack frames (caller first)
    pub frames: Vec<StackFrame>,
}

impl StackTrace {
    /// Create an empty stack trace with just a message.
    pub fn from_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            frames: Vec::new(),
        }
    }
    
    /// Create a stack trace from a JsError.
    pub fn from_js_error(error: &JsError, _context: &mut Context) -> Self {
        // Extract message from the error
        let message = format!("{:?}", error);
        
        // Boa doesn't expose stack frames directly yet
        // This is a placeholder for when Boa adds stack trace support
        Self {
            message,
            frames: Vec::new(),
        }
    }
    
    /// Format as a multi-line string (Chrome-style).
    pub fn format_chrome_style(&self) -> String {
        let mut result = self.message.clone();
        for frame in &self.frames {
            result.push('\n');
            result.push_str(&frame.to_string());
        }
        result
    }
    
    /// Format as a multi-line string (Firefox-style).
    pub fn format_firefox_style(&self) -> String {
        let mut result = self.message.clone();
        for frame in &self.frames {
            result.push('\n');
            let func = if frame.function_name.is_empty() {
                "@".to_string()
            } else {
                format!("{}@", frame.function_name)
            };
            result.push_str(&format!("{}{}:{}:{}", 
                func, frame.file_name, frame.line_number, frame.column_number));
        }
        result
    }
}

impl std::fmt::Display for StackTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_chrome_style())
    }
}

/// Extract error details from a JavaScript error value.
pub fn extract_error_info(error: &JsValue, context: &mut Context) -> (String, String, Option<StackTrace>) {
    if let Some(obj) = error.as_object() {
        // Try to get name
        let name = obj.get(boa_engine::JsString::from("name"), context)
            .ok()
            .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
            .unwrap_or_else(|| "Error".to_string());
        
        // Try to get message  
        let message = obj.get(boa_engine::JsString::from("message"), context)
            .ok()
            .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
            .unwrap_or_else(|| error.to_string(context)
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default());
        
        // Try to get stack
        let stack = obj.get(boa_engine::JsString::from("stack"), context)
            .ok()
            .and_then(|v| v.as_string().map(|s| {
                StackTrace::from_message(format!("{}: {}", name, message))
            }));
        
        (name, message, stack)
    } else {
        let message = error.to_string(context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_else(|_| "[Error]".to_string());
        
        ("Error".to_string(), message, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stack_frame_display() {
        let frame = StackFrame {
            function_name: "myFunction".to_string(),
            file_name: "script.js".to_string(),
            line_number: 10,
            column_number: 5,
        };
        
        let display = frame.to_string();
        assert!(display.contains("myFunction"));
        assert!(display.contains("script.js"));
        assert!(display.contains("10"));
        assert!(display.contains("5"));
    }
    
    #[test]
    fn test_stack_frame_anonymous() {
        let frame = StackFrame {
            function_name: "".to_string(),
            file_name: "script.js".to_string(),
            line_number: 1,
            column_number: 1,
        };
        
        let display = frame.to_string();
        assert!(display.contains("<anonymous>"));
    }
    
    #[test]
    fn test_stack_trace_format() {
        let trace = StackTrace {
            message: "Error: test error".to_string(),
            frames: vec![
                StackFrame {
                    function_name: "foo".to_string(),
                    file_name: "a.js".to_string(),
                    line_number: 1,
                    column_number: 10,
                },
                StackFrame {
                    function_name: "bar".to_string(),
                    file_name: "b.js".to_string(),
                    line_number: 5,
                    column_number: 3,
                },
            ],
        };
        
        let chrome = trace.format_chrome_style();
        assert!(chrome.contains("Error: test error"));
        assert!(chrome.contains("at foo"));
        assert!(chrome.contains("at bar"));
        
        let firefox = trace.format_firefox_style();
        assert!(firefox.contains("foo@a.js"));
        assert!(firefox.contains("bar@b.js"));
    }
    
    #[test]
    fn test_format_console_args() {
        let mut ctx = Context::default();
        
        let args = vec![
            JsValue::from(boa_engine::JsString::from("hello")),
            JsValue::from(42),
            JsValue::from(true),
        ];
        
        let formatted = format_console_args(&args, &mut ctx);
        assert!(formatted.contains("hello"));
        assert!(formatted.contains("42"));
        assert!(formatted.contains("true"));
    }
    
    #[test]
    fn test_format_special_values() {
        let mut ctx = Context::default();
        
        assert_eq!(format_js_value(&JsValue::undefined(), &mut ctx), "undefined");
        assert_eq!(format_js_value(&JsValue::null(), &mut ctx), "null");
        assert_eq!(format_js_value(&JsValue::from(f64::NAN), &mut ctx), "NaN");
        assert_eq!(format_js_value(&JsValue::from(f64::INFINITY), &mut ctx), "Infinity");
        assert_eq!(format_js_value(&JsValue::from(f64::NEG_INFINITY), &mut ctx), "-Infinity");
    }
}
