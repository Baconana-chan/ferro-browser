// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// JSID module for SpiderMonkey compatibility

use super::jsapi::RawJSContext;
use super::glue::jsid;

/// Well-known JavaScript Symbol IDs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SymbolCode {
    Iterator = 0,
    Match = 1,
    MatchAll = 2,
    Replace = 3,
    Search = 4,
    Species = 5,
    Split = 6,
    HasInstance = 7,
    IsConcatSpreadable = 8,
    Unscopables = 9,
    AsyncIterator = 10,
    ToStringTag = 11,
    ToPrimitive = 12,
    WellKnownSymbolLimit = 13,
}

/// SymbolId - represents a JS Symbol ID
#[derive(Clone, Copy, Debug)]
pub struct SymbolId(pub(crate) u32);

impl SymbolId {
    /// Create a SymbolId from a well-known symbol code
    pub fn new(code: SymbolCode) -> Self {
        Self(code as u32)
    }
    
    /// Get the symbol code
    pub fn code(&self) -> u32 {
        self.0
    }
}

/// Get the JSID for a well-known symbol
pub unsafe fn GetWellKnownSymbolKey(_cx: *mut RawJSContext, code: SymbolCode) -> jsid {
    jsid::from_well_known_symbol(code as u32)
}

/// Get the symbol code from a JSID
pub fn symbol_id(_id: jsid) -> Option<SymbolId> {
    // Stub implementation
    None
}
