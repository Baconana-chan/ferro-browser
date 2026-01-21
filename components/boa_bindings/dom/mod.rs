//! DOM Interface Bindings for Boa
//!
//! This module provides implementations of Web DOM interfaces for the Boa
//! JavaScript engine. Each interface is implemented following the WebIDL
//! specification with proper Rust ↔ JavaScript bindings.
//!
//! ## Interface Hierarchy
//!
//! ```text
//! EventTarget
//!     ├── Window
//!     └── Node
//!           ├── Document
//!           ├── DocumentFragment
//!           ├── Element
//!           │     └── HTMLElement
//!           └── CharacterData
//!                 ├── Text
//!                 └── Comment
//! ```
//!
//! ## Phase 3: DOM Bindings Core
//!
//! This phase adds HTMLElement, enhanced Event system, and CSS Object Model.

pub mod event_target;
pub mod event;
pub mod node;
pub mod element;
pub mod html_element;
pub mod document;
pub mod window;

// Re-export event_target types (for backwards compatibility)
pub use event_target::{EventTarget, EventListener, ListenerOptions};
// Re-export enhanced event types
pub use event::{Event, EventInit, EventPhase, CustomEvent, CustomEventInit};
pub use event::{MouseEvent, MouseEventInit, KeyboardEvent, KeyboardEventInit};
// Re-export node types
pub use node::{Node, NodeType};
// Re-export element types
pub use element::Element;
// Re-export HTMLElement types
pub use html_element::{HTMLElement, CSSStyleDeclaration, DOMTokenList, DOMStringMap};
// Re-export document and window
pub use document::Document;
pub use window::Window;

use boa_engine::{Context, JsResult, js_string};

/// Register all DOM interfaces on the global object
pub fn register_all_interfaces(ctx: &mut Context) -> JsResult<()> {
    // Initialize prototypes for each interface
    let _event_target_proto = event_target::register_event_target(ctx)?;
    let _event_proto = event::init_event_prototype(ctx)?;
    let _custom_event_proto = event::init_custom_event_prototype(ctx)?;
    let _node_proto = node::get_node_prototype(ctx)?;
    let _element_proto = element::init_element_prototype(ctx)?;
    let _html_element_proto = html_element::init_html_element_prototype(ctx)?;
    let _document_proto = document::init_document_prototype(ctx)?;
    let _window_proto = window::init_window_prototype(ctx)?;
    
    Ok(())
}

/// Setup a complete browser environment with window, document, etc.
pub fn setup_browser_environment(ctx: &mut Context) -> JsResult<()> {
    // Register all interface prototypes
    register_all_interfaces(ctx)?;
    
    // Create and setup global window object
    let window_proto = window::init_window_prototype(ctx)?;
    let global = ctx.global_object();
    
    // Set window as global property
    global.set(js_string!("window"), window_proto.clone(), false, ctx)?;
    
    Ok(())
}
