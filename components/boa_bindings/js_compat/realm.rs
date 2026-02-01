// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey realm module compatibility layer for Boa

use std::ptr;
use std::ffi::c_void;
use std::marker::PhantomData;

use super::jsapi::{RawJSContext, JSObject};
use super::rust::HandleObject;
use super::context::JSContext;

/// Realm - a JavaScript realm (global scope)
pub struct Realm {
    _private: (),
}

/// CurrentRealm - represents the current realm in the context
pub struct CurrentRealm<'a> {
    realm: *mut c_void,
    cx: *mut RawJSContext,
    _phantom: PhantomData<&'a mut JSContext>,
}

impl<'a> CurrentRealm<'a> {
    /// Get the current realm from a context
    pub fn new(cx: *mut RawJSContext) -> Option<Self> {
        let realm = unsafe { GetCurrentRealmOrNull(cx) };
        if realm.is_null() {
            None
        } else {
            Some(Self { realm, cx, _phantom: PhantomData })
        }
    }
    
    /// Get the raw realm pointer
    pub fn as_ptr(&self) -> *mut c_void {
        self.realm
    }
    
    /// Get the raw context
    pub fn raw_cx(&self) -> *mut RawJSContext {
        self.cx
    }
    
    /// Assert that we are in a realm and return a CurrentRealm
    pub fn assert(cx: &'a mut JSContext) -> Self {
        let raw = cx.as_ptr();
        let realm = unsafe { GetCurrentRealmOrNull(raw) };
        assert!(!realm.is_null(), "Not in a realm");
        Self { realm, cx: raw, _phantom: PhantomData }
    }
}

/// AutoRealm - RAII guard for entering/leaving realms
pub struct AutoRealm {
    cx: *mut RawJSContext,
    old_realm: *mut c_void,
}

impl AutoRealm {
    pub fn with_obj(cx: *mut RawJSContext, obj: HandleObject<'_>) -> Self {
        let old_realm = unsafe { super::context::JS_EnterRealm(cx, obj) };
        Self { cx, old_realm }
    }
    
    pub fn with_script(cx: *mut RawJSContext, _script: HandleObject<'_>) -> Self {
        Self {
            cx,
            old_realm: ptr::null_mut(),
        }
    }
    
    /// Create AutoRealm from a CurrentRealm and handle to an object
    pub fn new_from_handle<'a>(realm: &'a mut CurrentRealm<'_>, obj: HandleObject<'_>) -> Self {
        let cx = realm.raw_cx();
        Self::with_obj(cx, obj)
    }
    
    /// Get the raw JSContext pointer
    pub fn raw_cx(&self) -> *mut RawJSContext {
        self.cx
    }
    
    /// Get the current realm pointer
    pub fn current_realm(&self) -> *mut c_void {
        self.old_realm
    }
}

impl Drop for AutoRealm {
    fn drop(&mut self) {
        if !self.old_realm.is_null() {
            unsafe {
                super::context::JS_LeaveRealm(self.cx, self.old_realm);
            }
        }
    }
}

/// JSAutoRealm - alias for AutoRealm
pub type JSAutoRealm = AutoRealm;

/// Get realm for object
pub unsafe fn GetObjectRealm(_obj: *mut JSObject) -> *mut c_void {
    ptr::null_mut()
}

/// Get current realm
pub unsafe fn GetCurrentRealmOrNull(_cx: *mut RawJSContext) -> *mut c_void {
    ptr::null_mut()
}

/// Enter realm
pub unsafe fn EnterRealm(_cx: *mut RawJSContext, _realm: *mut c_void) {
}

/// Leave realm
pub unsafe fn LeaveRealm(_cx: *mut RawJSContext, _old_realm: *mut c_void) {
}

/// Get realm global
pub unsafe fn GetRealmGlobalOrNull(_realm: *mut c_void) -> *mut JSObject {
    ptr::null_mut()
}

/// Get realm object
pub unsafe fn GetRealmObjectOrNull(_realm: *mut c_void) -> *mut JSObject {
    ptr::null_mut()
}

/// Realm creation options
#[repr(C)]
pub struct RealmCreationOptions {
    pub class_is_dom: bool,
    pub shared_memory_and_atomics: bool,
}

impl Default for RealmCreationOptions {
    fn default() -> Self {
        Self {
            class_is_dom: false,
            shared_memory_and_atomics: false,
        }
    }
}

/// Realm behavior options
#[repr(C)]
pub struct RealmBehaviors {
    pub discard_source: bool,
}

impl Default for RealmBehaviors {
    fn default() -> Self {
        Self {
            discard_source: false,
        }
    }
}

/// Realm options combining creation and behavior
#[repr(C)]
pub struct RealmOptions {
    pub creation: RealmCreationOptions,
    pub behaviors: RealmBehaviors,
}

impl Default for RealmOptions {
    fn default() -> Self {
        Self {
            creation: RealmCreationOptions::default(),
            behaviors: RealmBehaviors::default(),
        }
    }
}

/// Create a new realm
pub unsafe fn NewRealm(
    _cx: *mut RawJSContext,
    _options: *const RealmOptions,
    _principals: *mut c_void,
) -> *mut c_void {
    ptr::null_mut()
}

/// Destroy a realm
pub unsafe fn DestroyRealm(_realm: *mut c_void) {
}

/// Set realm private
pub unsafe fn SetRealmPrivate(_realm: *mut c_void, _data: *mut c_void) {
}

/// Get realm private
pub unsafe fn GetRealmPrivate(_realm: *mut c_void) -> *mut c_void {
    ptr::null_mut()
}

/// Realm iterator
pub struct RealmIterator {
    _private: (),
}

impl Iterator for RealmIterator {
    type Item = *mut c_void;
    
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

/// Iterate all realms
pub unsafe fn RealmsIter(_cx: *mut RawJSContext) -> RealmIterator {
    RealmIterator { _private: () }
}
