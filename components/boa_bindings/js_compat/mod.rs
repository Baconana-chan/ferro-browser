// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey compatibility layer for Boa
// This module provides types and functions with the same interface as mozjs
// to allow components/script to compile with Boa instead of SpiderMonkey.

pub mod jsapi;
pub mod jsval;
pub mod rust;
pub mod gc;
pub mod typedarray;
pub mod conversions;
pub mod glue;
pub mod context;
pub mod realm;
pub mod error;
pub mod panic;

// Re-export common types at the root level
pub use jsapi::*;
pub use jsval::*;

/// JSCLASS flags
pub const JSCLASS_IS_DOMJSCLASS: u32 = 1 << 0;
pub const JSCLASS_IS_GLOBAL: u32 = 1 << 1;
pub const JSCLASS_RESERVED_SLOTS_SHIFT: u32 = 8;
pub const JSCLASS_RESERVED_SLOTS_MASK: u32 = 0xFF;
