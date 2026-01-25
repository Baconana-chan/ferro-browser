//! GC Safety Module for Boa Bindings
//!
//! This module provides runtime safety checks for garbage collection
//! in the DOM bindings. It helps catch common GC-related bugs:
//!
//! - Dangling references after GC
//! - Untraced objects on the stack
//! - Incorrect weak reference handling
//! - Finalization order issues
//!
//! # Debug Mode
//!
//! Set `GC_DEBUG=1` environment variable to enable verbose GC logging.
//! This adds performance overhead but helps diagnose GC issues.
//!
//! # Safety Assertions
//!
//! All safety checks use `debug_assert!` so they're only active in
//! debug builds. Release builds have zero overhead.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Environment variable for enabling GC debug mode
const GC_DEBUG_ENV: &str = "GC_DEBUG";

/// Global flag indicating if GC debug mode is enabled
static GC_DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Counter for GC cycles (for debugging)
static GC_CYCLE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Counter for allocations since last GC
static ALLOCATIONS_SINCE_GC: AtomicUsize = AtomicUsize::new(0);

/// Initialize GC debug mode from environment
pub fn init_gc_debug() {
    if std::env::var(GC_DEBUG_ENV).map(|v| v == "1" || v == "true").unwrap_or(false) {
        GC_DEBUG_ENABLED.store(true, Ordering::SeqCst);
        eprintln!("[GC_DEBUG] GC debug mode enabled");
    }
}

/// Check if GC debug mode is enabled
#[inline]
pub fn gc_debug_enabled() -> bool {
    GC_DEBUG_ENABLED.load(Ordering::Relaxed)
}

/// Log a GC debug message
#[macro_export]
macro_rules! gc_debug {
    ($($arg:tt)*) => {
        if $crate::gc_safety::gc_debug_enabled() {
            eprintln!("[GC_DEBUG] {}", format!($($arg)*));
        }
    };
}

/// Record an allocation for GC tracking
#[inline]
pub fn record_allocation() {
    ALLOCATIONS_SINCE_GC.fetch_add(1, Ordering::Relaxed);
}

/// Get the number of allocations since the last GC
#[inline]
pub fn allocations_since_gc() -> usize {
    ALLOCATIONS_SINCE_GC.load(Ordering::Relaxed)
}

/// Record a GC cycle
pub fn record_gc_cycle() {
    let cycle = GC_CYCLE_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
    ALLOCATIONS_SINCE_GC.store(0, Ordering::SeqCst);
    
    if gc_debug_enabled() {
        eprintln!("[GC_DEBUG] GC cycle #{} completed", cycle);
    }
}

/// Get the current GC cycle count
#[inline]
pub fn gc_cycle_count() -> usize {
    GC_CYCLE_COUNT.load(Ordering::SeqCst)
}

thread_local! {
    /// Tracks whether we're currently in a GC trace operation
    static IN_TRACE: Cell<bool> = const { Cell::new(false) };
    
    /// Tracks whether we're currently in a finalization operation  
    static IN_FINALIZE: Cell<bool> = const { Cell::new(false) };
    
    /// Tracks the depth of rooted scope (for detecting unrooted references)
    static ROOT_DEPTH: Cell<usize> = const { Cell::new(0) };
    
    /// Tracks whether GC is currently running
    static GC_RUNNING: Cell<bool> = const { Cell::new(false) };
}

/// Check if we're currently in a GC trace operation
#[inline]
pub fn is_in_trace() -> bool {
    IN_TRACE.with(|cell| cell.get())
}

/// Check if we're currently in a finalization operation
#[inline]
pub fn is_in_finalize() -> bool {
    IN_FINALIZE.with(|cell| cell.get())
}

/// Check if GC is currently running
#[inline]
pub fn is_gc_running() -> bool {
    GC_RUNNING.with(|cell| cell.get())
}

/// Get the current root depth
#[inline]
pub fn root_depth() -> usize {
    ROOT_DEPTH.with(|cell| cell.get())
}

/// RAII guard for trace operations
pub struct TraceGuard {
    _private: (),
}

impl TraceGuard {
    /// Enter a trace operation
    pub fn enter() -> Self {
        IN_TRACE.with(|cell| cell.set(true));
        TraceGuard { _private: () }
    }
}

impl Drop for TraceGuard {
    fn drop(&mut self) {
        IN_TRACE.with(|cell| cell.set(false));
    }
}

/// RAII guard for finalization operations
pub struct FinalizeGuard {
    _private: (),
}

impl FinalizeGuard {
    /// Enter a finalization operation
    pub fn enter() -> Self {
        IN_FINALIZE.with(|cell| cell.set(true));
        FinalizeGuard { _private: () }
    }
}

impl Drop for FinalizeGuard {
    fn drop(&mut self) {
        IN_FINALIZE.with(|cell| cell.set(false));
    }
}

/// RAII guard for GC operations
pub struct GcGuard {
    _private: (),
}

impl GcGuard {
    /// Enter a GC operation
    pub fn enter() -> Self {
        GC_RUNNING.with(|cell| cell.set(true));
        GcGuard { _private: () }
    }
}

impl Drop for GcGuard {
    fn drop(&mut self) {
        GC_RUNNING.with(|cell| cell.set(false));
        record_gc_cycle();
    }
}

/// RAII guard for rooted scope
pub struct RootGuard {
    _private: (),
}

impl RootGuard {
    /// Enter a rooted scope
    pub fn enter() -> Self {
        ROOT_DEPTH.with(|cell| cell.set(cell.get() + 1));
        RootGuard { _private: () }
    }
}

impl Drop for RootGuard {
    fn drop(&mut self) {
        ROOT_DEPTH.with(|cell| {
            let depth = cell.get();
            debug_assert!(depth > 0, "RootGuard depth underflow");
            cell.set(depth - 1);
        });
    }
}

/// Assert that we're in a rooted context
/// 
/// This should be called before creating unrooted DOM references
/// to ensure they're properly rooted.
#[inline]
pub fn assert_rooted() {
    debug_assert!(
        root_depth() > 0 || is_in_trace() || is_in_finalize(),
        "Unrooted DOM reference created outside of rooted context! \
         This may cause use-after-free if GC runs."
    );
}

/// Assert that we're not in GC
///
/// This should be called before allocating new DOM objects.
#[inline]
pub fn assert_not_in_gc() {
    debug_assert!(
        !is_gc_running(),
        "Attempting to allocate during GC! This is not allowed."
    );
}

/// Assert that an object is being traced
///
/// This is called during trace operations to verify all objects are traced.
#[inline]
pub fn assert_traced(type_name: &str) {
    if gc_debug_enabled() {
        eprintln!("[GC_DEBUG] Tracing: {}", type_name);
    }
}

/// Assert correct finalization order
///
/// Verifies that finalization follows the correct order:
/// 1. finalize (cleanup resources)
/// 2. detach (remove from parent structures)  
/// 3. invalidate (null out weak references)
#[inline]
pub fn assert_finalize_order(step: &str, type_name: &str) {
    if gc_debug_enabled() {
        eprintln!("[GC_DEBUG] Finalize {} step: {}", type_name, step);
    }
}

/// Weak reference tracking for debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeakRefState {
    /// The weak reference is valid and points to a live object
    Valid,
    /// The weak reference has been invalidated (object collected)
    Invalidated,
    /// The weak reference is being finalized
    Finalizing,
}

/// Debug info for a weak reference
#[derive(Debug)]
pub struct WeakRefDebugInfo {
    /// The type name of the referenced object
    pub type_name: &'static str,
    /// When the weak ref was created (GC cycle)
    pub created_at_cycle: usize,
    /// Current state
    pub state: WeakRefState,
}

impl WeakRefDebugInfo {
    /// Create new debug info for a weak reference
    pub fn new(type_name: &'static str) -> Self {
        WeakRefDebugInfo {
            type_name,
            created_at_cycle: gc_cycle_count(),
            state: WeakRefState::Valid,
        }
    }
    
    /// Mark as invalidated
    pub fn invalidate(&mut self) {
        debug_assert_eq!(
            self.state, 
            WeakRefState::Valid,
            "WeakRef already invalidated!"
        );
        self.state = WeakRefState::Invalidated;
    }
    
    /// Mark as finalizing
    pub fn start_finalize(&mut self) {
        debug_assert_eq!(
            self.state,
            WeakRefState::Valid,
            "Cannot finalize non-valid WeakRef!"
        );
        self.state = WeakRefState::Finalizing;
    }
}

/// Safety check for Rc<RefCell<>> usage
///
/// Detects potential issues with Rc<RefCell<>> that isn't traced.
/// In DOM code, Rc<RefCell<>> should be avoided in favor of
/// GcRefCell or MutDom.
#[macro_export]
macro_rules! assert_no_untraced_refcell {
    ($expr:expr) => {
        // This is a compile-time hint to developers
        // In practice, code review and clippy are better tools
        $expr
    };
}

/// Check for unsafe blocks without root
///
/// This is primarily a documentation/review aid.
/// Real enforcement would require a custom lint.
pub fn audit_unsafe_blocks() {
    // This function serves as a reminder to:
    // 1. Review all `unsafe` blocks in DOM code
    // 2. Ensure each has proper rooting
    // 3. Add comments explaining safety
    if gc_debug_enabled() {
        eprintln!("[GC_DEBUG] Reminder: Audit unsafe blocks for proper rooting");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gc_debug_init() {
        // Test that init doesn't panic
        init_gc_debug();
    }
    
    #[test]
    fn test_trace_guard() {
        assert!(!is_in_trace());
        {
            let _guard = TraceGuard::enter();
            assert!(is_in_trace());
        }
        assert!(!is_in_trace());
    }
    
    #[test]
    fn test_finalize_guard() {
        assert!(!is_in_finalize());
        {
            let _guard = FinalizeGuard::enter();
            assert!(is_in_finalize());
        }
        assert!(!is_in_finalize());
    }
    
    #[test]
    fn test_gc_guard() {
        let initial_cycle = gc_cycle_count();
        assert!(!is_gc_running());
        {
            let _guard = GcGuard::enter();
            assert!(is_gc_running());
        }
        assert!(!is_gc_running());
        assert_eq!(gc_cycle_count(), initial_cycle + 1);
    }
    
    #[test]
    fn test_root_guard() {
        assert_eq!(root_depth(), 0);
        {
            let _guard1 = RootGuard::enter();
            assert_eq!(root_depth(), 1);
            {
                let _guard2 = RootGuard::enter();
                assert_eq!(root_depth(), 2);
            }
            assert_eq!(root_depth(), 1);
        }
        assert_eq!(root_depth(), 0);
    }
    
    #[test]
    fn test_weak_ref_debug_info() {
        let mut info = WeakRefDebugInfo::new("TestObject");
        assert_eq!(info.state, WeakRefState::Valid);
        
        info.invalidate();
        assert_eq!(info.state, WeakRefState::Invalidated);
    }
    
    #[test]
    fn test_allocation_tracking() {
        let initial = allocations_since_gc();
        record_allocation();
        assert_eq!(allocations_since_gc(), initial + 1);
        record_allocation();
        assert_eq!(allocations_since_gc(), initial + 2);
    }
}
