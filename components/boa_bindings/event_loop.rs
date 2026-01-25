// MIT License
// Copyright (c) 2025-2026 Ferro Browser Contributors
// See LICENSE-MIT in the project root for full license text.

//! Event loop integration for Boa engine.
//!
//! This module provides the bridge between Boa's job queue (microtasks)
//! and the browser's event loop (tasks). It ensures proper ordering
//! according to the HTML specification.
//!
//! ## Task vs Microtask
//!
//! Per HTML spec:
//! - **Tasks**: setTimeout, setInterval, I/O callbacks, UI events
//! - **Microtasks**: Promise reactions, queueMicrotask(), MutationObserver
//!
//! After each task completes, all microtasks must be drained before
//! the next task runs.
//!
//! ## Integration Points
//!
//! - `drain_microtasks()`: Must be called after each task
//! - `schedule_task()`: Queue a new task (e.g., timer callback)
//! - `has_pending_work()`: Check if there's more work to do

use boa_engine::{Context, JsObject, JsResult, JsValue};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

// ============================================================================
// Task Queue
// ============================================================================

/// A task source identifier (per HTML spec)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskSource {
    /// Timer tasks (setTimeout, setInterval)
    Timer,
    /// DOM manipulation tasks
    DomManipulation,
    /// User interaction tasks (click, input, etc.)
    UserInteraction,
    /// Networking tasks (fetch, XHR)
    Networking,
    /// History traversal tasks
    HistoryTraversal,
    /// Generic tasks
    Other,
}

impl Default for TaskSource {
    fn default() -> Self {
        Self::Other
    }
}

/// A scheduled task in the event loop
pub struct Task {
    /// The callback to execute
    pub callback: Box<dyn FnOnce(&mut Context) -> JsResult<()>>,
    /// The task source
    pub source: TaskSource,
    /// Unique task ID
    pub id: u64,
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("source", &self.source)
            .field("id", &self.id)
            .finish()
    }
}

// ============================================================================
// Event Loop State
// ============================================================================

thread_local! {
    /// The task queue for this thread
    static TASK_QUEUE: RefCell<VecDeque<Task>> = RefCell::new(VecDeque::new());
    
    /// Counter for task IDs
    static NEXT_TASK_ID: Cell<u64> = const { Cell::new(1) };
    
    /// Flag indicating if we're currently processing a task
    static PROCESSING_TASK: Cell<bool> = const { Cell::new(false) };
    
    /// The microtask queue (Promise reactions, queueMicrotask)
    static MICROTASK_QUEUE: RefCell<VecDeque<JsObject>> = RefCell::new(VecDeque::new());
    
    /// Microtask checkpoint counter (for debugging)
    static MICROTASK_CHECKPOINTS: Cell<u64> = const { Cell::new(0) };
}

/// Generate a new unique task ID
fn next_task_id() -> u64 {
    NEXT_TASK_ID.with(|id| {
        let current = id.get();
        id.set(current + 1);
        current
    })
}

// ============================================================================
// Task Scheduling
// ============================================================================

/// Schedule a task to be executed in the event loop.
/// 
/// Returns the task ID which can be used to cancel the task.
pub fn schedule_task<F>(source: TaskSource, callback: F) -> u64
where
    F: FnOnce(&mut Context) -> JsResult<()> + 'static,
{
    let id = next_task_id();
    let task = Task {
        callback: Box::new(callback),
        source,
        id,
    };
    
    TASK_QUEUE.with(|queue| {
        queue.borrow_mut().push_back(task);
    });
    
    id
}

/// Schedule a JavaScript callback as a task.
pub fn schedule_js_callback(source: TaskSource, callback: JsObject) -> u64 {
    schedule_task(source, move |ctx| {
        if callback.is_callable() {
            callback.call(&JsValue::undefined(), &[], ctx)?;
        }
        Ok(())
    })
}

/// Cancel a scheduled task by ID.
/// 
/// Returns true if the task was found and cancelled.
pub fn cancel_task(task_id: u64) -> bool {
    TASK_QUEUE.with(|queue| {
        let mut q = queue.borrow_mut();
        if let Some(pos) = q.iter().position(|t| t.id == task_id) {
            q.remove(pos);
            true
        } else {
            false
        }
    })
}

/// Get the number of pending tasks.
pub fn pending_task_count() -> usize {
    TASK_QUEUE.with(|queue| queue.borrow().len())
}

// ============================================================================
// Microtask Queue
// ============================================================================

/// Queue a microtask (Promise reaction or queueMicrotask callback).
pub fn queue_microtask(callback: JsObject) {
    MICROTASK_QUEUE.with(|queue| {
        queue.borrow_mut().push_back(callback);
    });
}

/// Get the number of pending microtasks.
pub fn pending_microtask_count() -> usize {
    MICROTASK_QUEUE.with(|queue| queue.borrow().len())
}

/// Perform a microtask checkpoint.
/// 
/// This drains all microtasks in the queue, including any newly
/// queued microtasks during execution (until the queue is empty).
/// 
/// Per HTML spec, this must be called:
/// 1. After each task completes
/// 2. At specific checkpoints during parsing
/// 3. After callback invocation
pub fn perform_microtask_checkpoint(context: &mut Context) -> JsResult<()> {
    // First, run Boa's internal job queue (Promise reactions)
    let _ = context.run_jobs();
    
    // Then run our microtask queue
    loop {
        let task = MICROTASK_QUEUE.with(|queue| {
            queue.borrow_mut().pop_front()
        });
        
        match task {
            Some(callback) => {
                if callback.is_callable() {
                    // Execute and catch errors (don't propagate)
                    if let Err(e) = callback.call(&JsValue::undefined(), &[], context) {
                        log::warn!("[Microtask] Error: {:?}", e);
                    }
                    
                    // Run any newly queued Promise reactions
                    let _ = context.run_jobs();
                }
            }
            None => break,
        }
    }
    
    // Increment checkpoint counter
    MICROTASK_CHECKPOINTS.with(|c| c.set(c.get() + 1));
    
    Ok(())
}

/// Get the number of microtask checkpoints that have occurred.
pub fn microtask_checkpoint_count() -> u64 {
    MICROTASK_CHECKPOINTS.with(|c| c.get())
}

// ============================================================================
// Event Loop Execution
// ============================================================================

/// Process one task from the task queue.
/// 
/// Returns true if a task was processed, false if the queue was empty.
pub fn process_one_task(context: &mut Context) -> JsResult<bool> {
    let task = TASK_QUEUE.with(|queue| {
        queue.borrow_mut().pop_front()
    });
    
    match task {
        Some(task) => {
            PROCESSING_TASK.with(|p| p.set(true));
            
            // Execute the task
            let result = (task.callback)(context);
            
            PROCESSING_TASK.with(|p| p.set(false));
            
            // Microtask checkpoint after task completes
            perform_microtask_checkpoint(context)?;
            
            // Propagate task error
            result?;
            
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Run the event loop until no more work is pending.
/// 
/// This is useful for testing and simple scripts.
/// In a real browser, the event loop runs indefinitely.
pub fn run_until_empty(context: &mut Context) -> JsResult<()> {
    // Initial microtask checkpoint
    perform_microtask_checkpoint(context)?;
    
    while has_pending_work() {
        process_one_task(context)?;
    }
    
    Ok(())
}

/// Check if there's pending work in the event loop.
pub fn has_pending_work() -> bool {
    TASK_QUEUE.with(|q| !q.borrow().is_empty()) ||
    MICROTASK_QUEUE.with(|q| !q.borrow().is_empty())
}

/// Check if we're currently processing a task.
pub fn is_processing_task() -> bool {
    PROCESSING_TASK.with(|p| p.get())
}

// ============================================================================
// Integration with Boa's Promise System
// ============================================================================

/// A custom job queue that integrates with our event loop.
/// 
/// This can be used to intercept Promise reactions and route them
/// through our microtask queue for proper ordering.
pub struct EventLoopJobQueue;

impl EventLoopJobQueue {
    /// Create a new event loop job queue
    pub fn new() -> Self {
        Self
    }
}

impl Default for EventLoopJobQueue {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Full boa_engine::job::JobQueue implementation would go here
// For now, we rely on Boa's default job queue and augment with our microtasks

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Context;
    
    #[test]
    fn test_task_scheduling() {
        let task_id = schedule_task(TaskSource::Timer, |_ctx| {
            Ok(())
        });
        
        assert!(task_id > 0);
        assert_eq!(pending_task_count(), 1);
        
        // Cancel the task
        assert!(cancel_task(task_id));
        assert_eq!(pending_task_count(), 0);
    }
    
    #[test]
    fn test_task_execution() {
        let mut ctx = Context::default();
        
        // Schedule a simple task
        let executed = std::rc::Rc::new(std::cell::Cell::new(false));
        let executed_clone = executed.clone();
        
        schedule_task(TaskSource::Other, move |_ctx| {
            executed_clone.set(true);
            Ok(())
        });
        
        assert!(!executed.get());
        
        // Process the task
        let _ = process_one_task(&mut ctx);
        
        assert!(executed.get());
    }
    
    #[test]
    fn test_microtask_checkpoint() {
        let mut ctx = Context::default();
        
        let initial_count = microtask_checkpoint_count();
        
        perform_microtask_checkpoint(&mut ctx).unwrap();
        
        assert_eq!(microtask_checkpoint_count(), initial_count + 1);
    }
    
    #[test]
    fn test_task_source_default() {
        let source = TaskSource::default();
        assert_eq!(source, TaskSource::Other);
    }
    
    #[test]
    fn test_has_pending_work() {
        // Clear any existing tasks
        while pending_task_count() > 0 {
            TASK_QUEUE.with(|q| q.borrow_mut().pop_front());
        }
        
        assert!(!has_pending_work());
        
        schedule_task(TaskSource::Timer, |_ctx| Ok(()));
        
        assert!(has_pending_work());
    }
    
    #[test]
    fn test_run_until_empty() {
        let mut ctx = Context::default();
        
        let counter = std::rc::Rc::new(std::cell::Cell::new(0));
        
        // Schedule 3 tasks
        for _ in 0..3 {
            let counter_clone = counter.clone();
            schedule_task(TaskSource::Other, move |_ctx| {
                counter_clone.set(counter_clone.get() + 1);
                Ok(())
            });
        }
        
        assert_eq!(counter.get(), 0);
        
        run_until_empty(&mut ctx).unwrap();
        
        assert_eq!(counter.get(), 3);
    }
}
