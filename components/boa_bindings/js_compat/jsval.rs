// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey jsval compatibility layer for Boa

pub use super::jsapi::Value;

/// Constructors for JavaScript values
pub type JSVal = Value;

/// Create an undefined value
pub const fn UndefinedValue() -> Value {
    Value::undefined()
}

/// Create a null value
pub const fn NullValue() -> Value {
    Value::null()
}

/// Create a boolean value
pub fn BooleanValue(b: bool) -> Value {
    Value::from_bool(b)
}

/// Create an integer value
pub fn Int32Value(i: i32) -> Value {
    Value::from_i32(i)
}

/// Create a double value
pub fn DoubleValue(d: f64) -> Value {
    Value::from_f64(d)
}

/// Create a string value
pub fn StringValue(_s: *mut super::jsapi::JSString) -> Value {
    // TODO: proper string encoding
    Value { data: 0x0004_0000_0000_0000 }
}

/// Create an object value
pub fn ObjectValue(obj: *mut super::jsapi::JSObject) -> Value {
    Value::from_object(obj)
}

/// Create an object-or-null value
pub fn ObjectOrNullValue(obj: *mut super::jsapi::JSObject) -> Value {
    if obj.is_null() {
        NullValue()
    } else {
        ObjectValue(obj)
    }
}

/// Create a private value (for internal use)
pub fn PrivateValue(ptr: *const std::ffi::c_void) -> Value {
    Value { data: ptr as u64 | 0x0007_0000_0000_0000 }
}

// Additional Value methods are in jsapi.rs - don't duplicate here
