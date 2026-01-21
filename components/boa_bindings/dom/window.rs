//! Window Interface Binding for Boa
//!
//! The Window interface represents a browser window containing a DOM document.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use boa_engine::{
    Context, JsObject, JsResult, JsValue, js_string,
};

use super::event_target::EventTarget;
use super::document::Document;

/// Global counter for timer IDs
static TIMER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Global counter for animation frame IDs
static ANIMATION_FRAME_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Timer entry
#[derive(Clone)]
pub struct TimerEntry {
    pub id: u64,
    pub callback: JsObject,
    pub timeout_ms: u64,
    pub is_interval: bool,
    pub cancelled: bool,
}

/// Animation frame callback
#[derive(Clone)]
pub struct AnimationFrameEntry {
    pub id: u64,
    pub callback: JsObject,
    pub cancelled: bool,
}

/// Window implementation
pub struct Window {
    /// Base EventTarget
    event_target: EventTarget,
    
    /// Associated document
    document: RefCell<Option<Rc<RefCell<Document>>>>,
    
    /// Window name
    name: RefCell<String>,
    
    /// Status bar text
    status: RefCell<String>,
    
    /// Is the window closed
    closed: RefCell<bool>,
    
    /// Active timers
    timers: RefCell<HashMap<u64, TimerEntry>>,
    
    /// Animation frame callbacks
    animation_frame_callbacks: RefCell<HashMap<u64, AnimationFrameEntry>>,
    
    /// Inner width
    inner_width: RefCell<u32>,
    
    /// Inner height
    inner_height: RefCell<u32>,
    
    /// Device pixel ratio
    device_pixel_ratio: RefCell<f64>,
}

impl Window {
    /// Create a new Window
    pub fn new() -> Self {
        Window {
            event_target: EventTarget::new(),
            document: RefCell::new(None),
            name: RefCell::new(String::new()),
            status: RefCell::new(String::new()),
            closed: RefCell::new(false),
            timers: RefCell::new(HashMap::new()),
            animation_frame_callbacks: RefCell::new(HashMap::new()),
            inner_width: RefCell::new(1024),
            inner_height: RefCell::new(768),
            device_pixel_ratio: RefCell::new(1.0),
        }
    }

    /// Create a new Window with a document
    pub fn new_with_document(document: Rc<RefCell<Document>>) -> Self {
        let window = Window::new();
        *window.document.borrow_mut() = Some(document);
        window
    }

    /// Get the document
    pub fn document(&self) -> Option<Rc<RefCell<Document>>> {
        self.document.borrow().clone()
    }

    /// Set the document
    pub fn set_document(&self, document: Rc<RefCell<Document>>) {
        *self.document.borrow_mut() = Some(document);
    }

    /// Get the window name
    pub fn name(&self) -> String {
        self.name.borrow().clone()
    }

    /// Set the window name
    pub fn set_name(&self, name: &str) {
        *self.name.borrow_mut() = name.to_string();
    }

    /// Get the status bar text
    pub fn status(&self) -> String {
        self.status.borrow().clone()
    }

    /// Set the status bar text
    pub fn set_status(&self, status: &str) {
        *self.status.borrow_mut() = status.to_string();
    }

    /// Check if window is closed
    pub fn closed(&self) -> bool {
        *self.closed.borrow()
    }

    /// Close the window
    pub fn close(&self) {
        *self.closed.borrow_mut() = true;
    }

    /// Stop loading
    pub fn stop(&self) {
        // Stub
    }

    /// Focus the window
    pub fn focus(&self) {
        // Stub
    }

    /// Blur the window
    pub fn blur(&self) {
        // Stub
    }

    /// Get inner width
    pub fn inner_width(&self) -> u32 {
        *self.inner_width.borrow()
    }

    /// Set inner width
    pub fn set_inner_width(&self, width: u32) {
        *self.inner_width.borrow_mut() = width;
    }

    /// Get inner height
    pub fn inner_height(&self) -> u32 {
        *self.inner_height.borrow()
    }

    /// Set inner height
    pub fn set_inner_height(&self, height: u32) {
        *self.inner_height.borrow_mut() = height;
    }

    /// Get device pixel ratio
    pub fn device_pixel_ratio(&self) -> f64 {
        *self.device_pixel_ratio.borrow()
    }

    /// Set device pixel ratio
    pub fn set_device_pixel_ratio(&self, ratio: f64) {
        *self.device_pixel_ratio.borrow_mut() = ratio;
    }

    /// Show an alert dialog (stub)
    pub fn alert(&self, _message: &str) {
        // Stub
    }

    /// Show a confirm dialog (stub)
    pub fn confirm(&self, _message: &str) -> bool {
        false
    }

    /// Show a prompt dialog (stub)
    pub fn prompt(&self, _message: &str, _default: &str) -> Option<String> {
        None
    }

    /// Set a timeout
    pub fn set_timeout(&self, callback: JsObject, timeout_ms: u64) -> u64 {
        let id = TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let entry = TimerEntry {
            id,
            callback,
            timeout_ms,
            is_interval: false,
            cancelled: false,
        };
        self.timers.borrow_mut().insert(id, entry);
        id
    }

    /// Clear a timeout
    pub fn clear_timeout(&self, id: u64) {
        if let Some(entry) = self.timers.borrow_mut().get_mut(&id) {
            entry.cancelled = true;
        }
    }

    /// Set an interval
    pub fn set_interval(&self, callback: JsObject, interval_ms: u64) -> u64 {
        let id = TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let entry = TimerEntry {
            id,
            callback,
            timeout_ms: interval_ms,
            is_interval: true,
            cancelled: false,
        };
        self.timers.borrow_mut().insert(id, entry);
        id
    }

    /// Clear an interval
    pub fn clear_interval(&self, id: u64) {
        self.clear_timeout(id);
    }

    /// Request animation frame
    pub fn request_animation_frame(&self, callback: JsObject) -> u64 {
        let id = ANIMATION_FRAME_COUNTER.fetch_add(1, Ordering::SeqCst);
        let entry = AnimationFrameEntry {
            id,
            callback,
            cancelled: false,
        };
        self.animation_frame_callbacks.borrow_mut().insert(id, entry);
        id
    }

    /// Cancel animation frame
    pub fn cancel_animation_frame(&self, id: u64) {
        if let Some(entry) = self.animation_frame_callbacks.borrow_mut().get_mut(&id) {
            entry.cancelled = true;
        }
    }

    /// Get pending timers
    pub fn get_pending_timers(&self) -> Vec<TimerEntry> {
        self.timers
            .borrow()
            .values()
            .filter(|t| !t.cancelled)
            .cloned()
            .collect()
    }

    /// Get pending animation frame callbacks
    pub fn get_pending_animation_frames(&self) -> Vec<AnimationFrameEntry> {
        self.animation_frame_callbacks
            .borrow()
            .values()
            .filter(|f| !f.cancelled)
            .cloned()
            .collect()
    }

    /// Remove completed timers
    pub fn remove_timer(&self, id: u64) {
        self.timers.borrow_mut().remove(&id);
    }

    /// Remove completed animation frame callback
    pub fn remove_animation_frame(&self, id: u64) {
        self.animation_frame_callbacks.borrow_mut().remove(&id);
    }

    /// Access the underlying EventTarget
    pub fn as_event_target(&self) -> &EventTarget {
        &self.event_target
    }

    /// Access the underlying EventTarget mutably
    pub fn as_event_target_mut(&mut self) -> &mut EventTarget {
        &mut self.event_target
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize Window prototype on the global object
pub fn init_window_prototype(context: &mut Context) -> JsResult<JsObject> {
    let prototype = JsObject::with_null_proto();

    // Set simple value properties
    prototype.set(js_string!("name"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("closed"), JsValue::from(false), false, context)?;
    prototype.set(js_string!("innerWidth"), JsValue::from(1024), false, context)?;
    prototype.set(js_string!("innerHeight"), JsValue::from(768), false, context)?;
    prototype.set(js_string!("devicePixelRatio"), JsValue::from(1.0), false, context)?;
    prototype.set(js_string!("document"), JsValue::null(), false, context)?;

    // Set stub method objects
    let empty_func = JsObject::with_null_proto();
    prototype.set(js_string!("alert"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("confirm"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("prompt"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("close"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("stop"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("focus"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("blur"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("setTimeout"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("clearTimeout"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("setInterval"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("clearInterval"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("requestAnimationFrame"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("cancelAnimationFrame"), JsValue::from(empty_func.clone()), false, context)?;

    // Create console object
    let console = JsObject::with_null_proto();
    console.set(js_string!("log"), JsValue::from(empty_func.clone()), false, context)?;
    console.set(js_string!("error"), JsValue::from(empty_func.clone()), false, context)?;
    console.set(js_string!("warn"), JsValue::from(empty_func.clone()), false, context)?;
    console.set(js_string!("info"), JsValue::from(empty_func.clone()), false, context)?;
    console.set(js_string!("debug"), JsValue::from(empty_func), false, context)?;
    prototype.set(js_string!("console"), JsValue::from(console), false, context)?;

    Ok(prototype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let window = Window::new();
        assert!(!window.closed());
        assert_eq!(window.name(), "");
        assert!(window.document().is_none());
    }

    #[test]
    fn test_window_with_document() {
        let doc = Rc::new(RefCell::new(Document::new()));
        let window = Window::new_with_document(doc.clone());
        assert!(window.document().is_some());
    }

    #[test]
    fn test_window_name() {
        let window = Window::new();
        window.set_name("Test Window");
        assert_eq!(window.name(), "Test Window");
    }

    #[test]
    fn test_window_status() {
        let window = Window::new();
        window.set_status("Loading...");
        assert_eq!(window.status(), "Loading...");
    }

    #[test]
    fn test_window_close() {
        let window = Window::new();
        assert!(!window.closed());
        window.close();
        assert!(window.closed());
    }

    #[test]
    fn test_window_dimensions() {
        let window = Window::new();
        assert_eq!(window.inner_width(), 1024);
        assert_eq!(window.inner_height(), 768);
        
        window.set_inner_width(1920);
        window.set_inner_height(1080);
        
        assert_eq!(window.inner_width(), 1920);
        assert_eq!(window.inner_height(), 1080);
    }

    #[test]
    fn test_device_pixel_ratio() {
        let window = Window::new();
        assert_eq!(window.device_pixel_ratio(), 1.0);
        
        window.set_device_pixel_ratio(2.0);
        assert_eq!(window.device_pixel_ratio(), 2.0);
    }

    #[test]
    fn test_set_timeout() {
        let window = Window::new();
        let callback = JsObject::with_null_proto();
        
        let id1 = window.set_timeout(callback.clone(), 100);
        let id2 = window.set_timeout(callback.clone(), 200);
        
        assert!(id1 > 0);
        assert!(id2 > id1);
        
        let timers = window.get_pending_timers();
        assert_eq!(timers.len(), 2);
    }

    #[test]
    fn test_clear_timeout() {
        let window = Window::new();
        let callback = JsObject::with_null_proto();
        
        let id = window.set_timeout(callback, 100);
        window.clear_timeout(id);
        
        let timers = window.get_pending_timers();
        assert_eq!(timers.len(), 0);
    }

    #[test]
    fn test_set_interval() {
        let window = Window::new();
        let callback = JsObject::with_null_proto();
        
        let id = window.set_interval(callback, 100);
        
        let timers = window.get_pending_timers();
        assert_eq!(timers.len(), 1);
        assert!(timers[0].is_interval);
        
        window.clear_interval(id);
        let timers = window.get_pending_timers();
        assert_eq!(timers.len(), 0);
    }

    #[test]
    fn test_request_animation_frame() {
        let window = Window::new();
        let callback = JsObject::with_null_proto();
        
        let id1 = window.request_animation_frame(callback.clone());
        let id2 = window.request_animation_frame(callback.clone());
        
        assert!(id1 > 0);
        assert!(id2 > id1);
        
        let frames = window.get_pending_animation_frames();
        assert_eq!(frames.len(), 2);
    }

    #[test]
    fn test_cancel_animation_frame() {
        let window = Window::new();
        let callback = JsObject::with_null_proto();
        
        let id = window.request_animation_frame(callback);
        window.cancel_animation_frame(id);
        
        let frames = window.get_pending_animation_frames();
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn test_dialogs_stub() {
        let window = Window::new();
        
        window.alert("Test alert");
        assert!(!window.confirm("Test confirm"));
        assert!(window.prompt("Test prompt", "default").is_none());
    }
}
