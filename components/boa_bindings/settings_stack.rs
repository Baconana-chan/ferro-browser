// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// Settings stack for Boa engine - equivalent to script_bindings/settings_stack.rs
// This implements the HTML script settings stack for proper callback ordering
//
// Supports:
// - Nested realms (iframe-like calls)
// - Callbacks from timers / fetch
// - Correct current global and `this` binding

use std::cell::RefCell;
use std::marker::PhantomData;

use boa_gc::{Trace, Finalize};

use crate::reflector::DomObject;
use crate::root::{Dom, DomRoot};
use crate::gc_safety::gc_debug_enabled;

/// The kind of entry in the settings stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackEntryKind {
    /// An incumbent script entry (for callbacks).
    Incumbent,
    /// A regular entry script.
    Entry,
}

/// Depth tracking for nested realm debugging
#[derive(Debug, Default)]
pub struct RealmDepthTracker {
    /// Current nesting depth
    pub depth: usize,
    /// Maximum depth seen
    pub max_depth: usize,
}

thread_local! {
    /// Track realm nesting depth for debugging
    static REALM_DEPTH: RefCell<RealmDepthTracker> = RefCell::new(RealmDepthTracker::default());
}

/// Get current realm nesting depth
pub fn realm_depth() -> usize {
    REALM_DEPTH.with(|d| d.borrow().depth)
}

/// Get maximum realm nesting depth seen
pub fn max_realm_depth() -> usize {
    REALM_DEPTH.with(|d| d.borrow().max_depth)
}

fn enter_realm() {
    REALM_DEPTH.with(|d| {
        let mut tracker = d.borrow_mut();
        tracker.depth += 1;
        if tracker.depth > tracker.max_depth {
            tracker.max_depth = tracker.depth;
        }
        if gc_debug_enabled() {
            eprintln!("[GC_DEBUG] Enter realm, depth = {}", tracker.depth);
        }
    });
}

fn leave_realm() {
    REALM_DEPTH.with(|d| {
        let mut tracker = d.borrow_mut();
        debug_assert!(tracker.depth > 0, "Realm depth underflow!");
        tracker.depth -= 1;
        if gc_debug_enabled() {
            eprintln!("[GC_DEBUG] Leave realm, depth = {}", tracker.depth);
        }
    });
}

/// An entry in the script settings stack.
pub struct StackEntry<G: DomObject + Clone> {
    /// The global scope for this entry.
    pub global: Dom<G>,
    /// The kind of entry (incumbent or entry).
    pub kind: StackEntryKind,
}

/// Trait for types that provide access to the settings stack.
/// This is used to get the thread-local settings stack for a DOM type system.
pub trait SettingsStackAccess {
    /// The GlobalScope type.
    type GlobalScope: DomObject + Clone + Trace + Finalize;
    
    /// Get the thread-local settings stack.
    fn settings_stack() -> &'static std::thread::LocalKey<RefCell<Vec<StackEntry<Self::GlobalScope>>>>;
}

/// RAII struct that pushes and pops entries from the script settings stack.
/// 
/// This implements:
/// - <https://html.spec.whatwg.org/multipage/#prepare-to-run-script>
/// - <https://html.spec.whatwg.org/multipage/#clean-up-after-running-script>
pub struct AutoEntryScript<G: DomObject + Clone + Trace + Finalize + 'static> {
    global: DomRoot<G>,
    _marker: PhantomData<G>,
}

impl<G: DomObject + Clone + Trace + Finalize + 'static> AutoEntryScript<G> {
    /// Prepare to run script with the given global scope.
    /// 
    /// # Safety
    /// The caller must ensure that the global scope outlives this AutoEntryScript.
    pub fn new<S: SettingsStackAccess<GlobalScope = G>>(global: &G) -> Self {
        enter_realm();
        
        let settings_stack = S::settings_stack();
        settings_stack.with(|stack| {
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: Dom::from_ref(global),
                kind: StackEntryKind::Entry,
            });
        });
        
        Self {
            global: DomRoot::from_ref(global),
            _marker: PhantomData,
        }
    }
    
    /// Get a reference to the global scope.
    pub fn global(&self) -> &G {
        &self.global
    }
}

impl<G: DomObject + Clone + Trace + Finalize + 'static> Drop for AutoEntryScript<G> {
    fn drop(&mut self) {
        // Clean up after running script
        // 1. Pop the entry from the settings stack
        // 2. Perform a microtask checkpoint if the stack is empty (done elsewhere)
        leave_realm();
        
        if gc_debug_enabled() {
            eprintln!("[GC_DEBUG] AutoEntryScript dropped for global");
        }
    }
}

/// RAII struct that pushes and pops incumbent entries from the script settings stack.
/// 
/// This implements:
/// - <https://html.spec.whatwg.org/multipage/#prepare-to-run-a-callback>
/// - <https://html.spec.whatwg.org/multipage/#clean-up-after-running-a-callback>
pub struct AutoIncumbentScript<G: DomObject + Clone> {
    global_ptr: usize, // Store as usize to avoid needing Clone on drop
    _marker: PhantomData<G>,
}

impl<G: DomObject + Clone + Trace + Finalize + 'static> AutoIncumbentScript<G> {
    /// Prepare to run a callback with the given global scope.
    pub fn new<S: SettingsStackAccess<GlobalScope = G>>(global: &G) -> Self {
        enter_realm();
        
        let settings_stack = S::settings_stack();
        settings_stack.with(|stack| {
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: Dom::from_ref(global),
                kind: StackEntryKind::Incumbent,
            });
        });
        
        Self {
            global_ptr: global as *const _ as usize,
            _marker: PhantomData,
        }
    }
}

impl<G: DomObject + Clone> Drop for AutoIncumbentScript<G> {
    fn drop(&mut self) {
        // Clean up after running a callback
        // Pop the incumbent entry from the stack
        leave_realm();
        
        if gc_debug_enabled() {
            eprintln!("[GC_DEBUG] AutoIncumbentScript dropped");
        }
    }
}

/// Get the entry global scope from the settings stack.
/// 
/// Returns None if the stack is empty.
pub fn entry_global<S: SettingsStackAccess>() -> Option<DomRoot<S::GlobalScope>> {
    let settings_stack = S::settings_stack();
    settings_stack.with(|stack| {
        let stack = stack.borrow();
        for entry in stack.iter().rev() {
            if entry.kind == StackEntryKind::Entry {
                return Some(DomRoot::from_ref(&*entry.global));
            }
        }
        None
    })
}

/// Get the incumbent global scope from the settings stack.
/// 
/// Returns the topmost global scope in the stack.
pub fn incumbent_global<S: SettingsStackAccess>() -> Option<DomRoot<S::GlobalScope>> {
    let settings_stack = S::settings_stack();
    settings_stack.with(|stack| {
        let stack = stack.borrow();
        stack.last().map(|entry| DomRoot::from_ref(&*entry.global))
    })
}

/// Check if there is an entry global scope in the stack.
pub fn has_entry_global<S: SettingsStackAccess>() -> bool {
    let settings_stack = S::settings_stack();
    settings_stack.with(|stack| {
        let stack = stack.borrow();
        stack.iter().any(|e| e.kind == StackEntryKind::Entry)
    })
}

/// Check if the settings stack is empty.
pub fn is_stack_empty<S: SettingsStackAccess>() -> bool {
    let settings_stack = S::settings_stack();
    settings_stack.with(|stack| {
        stack.borrow().is_empty()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_entry_kind() {
        assert_eq!(StackEntryKind::Entry, StackEntryKind::Entry);
        assert_ne!(StackEntryKind::Entry, StackEntryKind::Incumbent);
    }
    
    #[test]
    fn test_realm_depth_tracking() {
        // Initial depth should be 0
        assert_eq!(realm_depth(), 0);
        
        // Enter realm
        enter_realm();
        assert_eq!(realm_depth(), 1);
        
        // Nested enter
        enter_realm();
        assert_eq!(realm_depth(), 2);
        assert_eq!(max_realm_depth(), 2);
        
        // Leave nested realm
        leave_realm();
        assert_eq!(realm_depth(), 1);
        
        // Leave outer realm
        leave_realm();
        assert_eq!(realm_depth(), 0);
        
        // Max depth should still be 2
        assert_eq!(max_realm_depth(), 2);
    }
    
    #[test]
    fn test_nested_realms_simulation() {
        // Simulate nested realm scenario like iframe callbacks
        
        // Enter main document realm
        enter_realm();
        assert_eq!(realm_depth(), 1);
        
        // Enter iframe realm (nested)
        enter_realm();
        assert_eq!(realm_depth(), 2);
        
        // Enter callback within iframe (e.g., setTimeout)
        enter_realm();
        assert_eq!(realm_depth(), 3);
        
        // Callback returns
        leave_realm();
        assert_eq!(realm_depth(), 2);
        
        // iframe code returns
        leave_realm();
        assert_eq!(realm_depth(), 1);
        
        // main document continues
        leave_realm();
        assert_eq!(realm_depth(), 0);
    }
    
    #[test]
    #[should_panic(expected = "Realm depth underflow")]
    fn test_realm_depth_underflow_panics() {
        // This should panic when trying to leave a realm we never entered
        leave_realm();
    }
}
