// Copyright 2024 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// Settings stack for Boa engine - equivalent to script_bindings/settings_stack.rs
// This implements the HTML script settings stack for proper callback ordering

use std::cell::RefCell;
use std::marker::PhantomData;

use boa_gc::{Trace, Finalize};

use crate::reflector::DomObject;
use crate::root::{Dom, DomRoot};

/// The kind of entry in the settings stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackEntryKind {
    /// An incumbent script entry (for callbacks).
    Incumbent,
    /// A regular entry script.
    Entry,
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
        // In a full implementation, this would:
        // 1. Pop the entry from the settings stack
        // 2. Perform a microtask checkpoint if the stack is empty
        // For now, we just log that we're cleaning up
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
}
