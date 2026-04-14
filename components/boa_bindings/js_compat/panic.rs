// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// SpiderMonkey panic module compatibility layer for Boa

use std::ptr;
use std::ffi::c_void;
use std::panic::{self, AssertUnwindSafe, UnwindSafe};

use super::jsapi::RawJSContext;

/// PanicResult - result of a panic-safe operation
pub enum PanicResult<T> {
    Ok(T),
    Err,
}

impl<T> PanicResult<T> {
    pub fn is_ok(&self) -> bool {
        matches!(self, PanicResult::Ok(_))
    }
    
    pub fn is_err(&self) -> bool {
        matches!(self, PanicResult::Err)
    }
    
    pub fn unwrap(self) -> T {
        match self {
            PanicResult::Ok(v) => v,
            PanicResult::Err => panic!("called unwrap on PanicResult::Err"),
        }
    }
    
    pub fn ok(self) -> Option<T> {
        match self {
            PanicResult::Ok(v) => Some(v),
            PanicResult::Err => None,
        }
    }
}

/// Wrap a closure in a panic-catching boundary
pub fn wrap_panic<F>(f: F)
where
    F: FnOnce(),
{
    let _ = panic::catch_unwind(AssertUnwindSafe(f));
}

/// Wrap a closure that may panic, catching the panic
pub fn wrap_panic_result<F, R, E>(f: F) -> Result<R, E>
where
    F: FnOnce() -> Result<R, E>,
    E: Default,
{
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(_) => Err(E::default()),
    }
}

/// Wrapper for calling closures from JS callbacks safely
pub fn wrap_panic_for_js<F, R>(f: F) -> Option<R>
where
    F: FnOnce() -> R,
{
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}

/// Wrapper for calling closures from JS callbacks safely
/// This version takes a context and returns a bool for success/failure
pub unsafe fn call_with_unwind_protection<F, R>(
    _cx: *mut RawJSContext,
    f: F,
    default: R,
) -> R
where
    F: FnOnce() -> R,
{
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(_) => default,
    }
}

/// Helper to make things UnwindSafe
pub struct MaybeUnwindSafe<T>(pub T);

impl<T> UnwindSafe for MaybeUnwindSafe<T> {}

impl<T> std::ops::Deref for MaybeUnwindSafe<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for MaybeUnwindSafe<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// AssertUnwindSafe wrapper - re-export from std
pub use std::panic::AssertUnwindSafe as ForceUnwindSafe;

/// Set a custom panic hook for JS context
pub unsafe fn set_panic_hook(
    _cx: *mut RawJSContext,
    _hook: Option<unsafe extern "C" fn(*mut c_void)>,
    _data: *mut c_void,
) {
}

/// Get the current panic hook
pub unsafe fn get_panic_hook(
    _cx: *mut RawJSContext,
) -> Option<unsafe extern "C" fn(*mut c_void)> {
    None
}

/// Check if we're currently panicking
pub fn is_panicking() -> bool {
    std::thread::panicking()
}

/// Resume a panic after doing cleanup
pub fn resume_panic(payload: Box<dyn std::any::Any + Send>) -> ! {
    panic::resume_unwind(payload)
}

/// Maybe resume an unwind if there was a saved panic
/// This is a no-op stub for compatibility
pub fn maybe_resume_unwind() {
    // In SpiderMonkey this checks for a saved panic and resumes it
    // For Boa, we don't need this functionality
}

/// Abort on panic (for unrecoverable situations)
pub fn abort_on_panic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    struct AbortOnDrop;
    
    impl Drop for AbortOnDrop {
        fn drop(&mut self) {
            if std::thread::panicking() {
                std::process::abort();
            }
        }
    }
    
    let _guard = AbortOnDrop;
    f()
}

/// Helper macro for wrapping JS callbacks
#[macro_export]
macro_rules! wrap_js_callback {
    ($cx:expr, $body:expr) => {{
        use $crate::js_compat::panic::wrap_panic_for_js;
        use std::panic::AssertUnwindSafe;
        
        wrap_panic_for_js(AssertUnwindSafe(|| $body))
    }};
}

/// Helper macro for wrapping JS callbacks that return bool
#[macro_export]
macro_rules! wrap_js_callback_bool {
    ($cx:expr, $body:expr) => {{
        use $crate::js_compat::panic::wrap_panic_for_js;
        use std::panic::AssertUnwindSafe;
        
        wrap_panic_for_js(AssertUnwindSafe(|| $body)).unwrap_or(false)
    }};
}
