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
