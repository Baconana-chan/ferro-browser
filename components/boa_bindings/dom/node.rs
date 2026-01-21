//! Node Interface Binding for Boa
//!
//! The Node interface is the primary DOM interface that defines the basic
//! structure of a document tree.

use std::cell::RefCell;
use std::rc::Rc;

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, js_string,
};

use crate::codegen::traits::{ConstantValue, InterfaceBuilder};
use crate::codegen::interface::get_arg_with_default;
use super::event_target::EventTarget;

/// Node type constants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum NodeType {
    Element = 1,
    Attribute = 2,
    Text = 3,
    CDATASection = 4,
    ProcessingInstruction = 7,
    Comment = 8,
    Document = 9,
    DocumentType = 10,
    DocumentFragment = 11,
}

impl From<NodeType> for u16 {
    fn from(nt: NodeType) -> u16 {
        nt as u16
    }
}

/// Node implementation
pub struct Node {
    /// Base EventTarget functionality
    event_target: EventTarget,
    
    /// Node type
    node_type: NodeType,
    
    /// Node name
    node_name: String,
    
    /// Base URI
    base_uri: String,
    
    /// Node value (for text nodes, comments, etc.)
    node_value: RefCell<Option<String>>,
    
    /// Parent node
    parent: RefCell<Option<Rc<RefCell<Node>>>>,
    
    /// Child nodes
    children: RefCell<Vec<Rc<RefCell<Node>>>>,
    
    /// Is connected to document
    is_connected: RefCell<bool>,
}

impl Node {
    /// Create a new Node with the given type
    pub fn new(node_type: NodeType, node_name: &str) -> Self {
        Node {
            event_target: EventTarget::new(),
            node_type,
            node_name: node_name.to_string(),
            base_uri: String::new(),
            node_value: RefCell::new(None),
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
            is_connected: RefCell::new(false),
        }
    }

    /// Create a new text node
    pub fn new_text(data: &str) -> Self {
        let node = Node::new(NodeType::Text, "#text");
        *node.node_value.borrow_mut() = Some(data.to_string());
        node
    }

    /// Create a new comment node
    pub fn new_comment(data: &str) -> Self {
        let node = Node::new(NodeType::Comment, "#comment");
        *node.node_value.borrow_mut() = Some(data.to_string());
        node
    }

    // ========================================================================
    // Attributes
    // ========================================================================

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn node_name(&self) -> &str {
        &self.node_name
    }

    pub fn base_uri(&self) -> &str {
        &self.base_uri
    }

    pub fn is_connected(&self) -> bool {
        *self.is_connected.borrow()
    }

    pub fn parent_node(&self) -> Option<Rc<RefCell<Node>>> {
        self.parent.borrow().clone()
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.children.borrow().first().cloned()
    }

    pub fn last_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.children.borrow().last().cloned()
    }

    pub fn node_value(&self) -> Option<String> {
        self.node_value.borrow().clone()
    }

    pub fn set_node_value(&self, value: Option<String>) {
        *self.node_value.borrow_mut() = value;
    }

    pub fn has_child_nodes(&self) -> bool {
        !self.children.borrow().is_empty()
    }

    pub fn child_nodes(&self) -> Vec<Rc<RefCell<Node>>> {
        self.children.borrow().clone()
    }

    // ========================================================================
    // Methods
    // ========================================================================

    /// Append a child node
    pub fn append_child(this: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) -> JsResult<Rc<RefCell<Node>>> {
        // Remove from old parent if any
        if let Some(old_parent) = child.borrow().parent.borrow().clone() {
            Node::remove_child(&old_parent, child.clone())?;
        }

        // Update child's parent reference
        *child.borrow().parent.borrow_mut() = Some(this.clone());

        // Add to children
        this.borrow().children.borrow_mut().push(child.clone());

        // Update connected status
        if this.borrow().is_connected() {
            *child.borrow().is_connected.borrow_mut() = true;
        }

        Ok(child)
    }

    /// Remove a child node
    pub fn remove_child(this: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) -> JsResult<Rc<RefCell<Node>>> {
        let this_borrowed = this.borrow();
        let mut children = this_borrowed.children.borrow_mut();
        
        let index = children
            .iter()
            .position(|n| Rc::ptr_eq(n, &child))
            .ok_or_else(|| {
                JsNativeError::error()
                    .with_message("Node is not a child of this node")
            })?;

        // Clear references
        *child.borrow().parent.borrow_mut() = None;
        *child.borrow().is_connected.borrow_mut() = false;

        // Remove from children
        children.remove(index);

        Ok(child)
    }

    /// Check if two nodes are equal
    pub fn is_equal_node(&self, other: &Node) -> bool {
        if self.node_type != other.node_type {
            return false;
        }
        if self.node_name != other.node_name {
            return false;
        }
        if self.node_value.borrow().as_ref() != other.node_value.borrow().as_ref() {
            return false;
        }
        true
    }

    /// Check if this node contains another node
    pub fn contains(&self, other: &Rc<RefCell<Node>>) -> bool {
        for child in self.children.borrow().iter() {
            if Rc::ptr_eq(child, other) {
                return true;
            }
            if child.borrow().contains(other) {
                return true;
            }
        }
        false
    }

    /// Get text content
    pub fn text_content(&self) -> Option<String> {
        match self.node_type {
            NodeType::Text | NodeType::Comment | NodeType::CDATASection | NodeType::ProcessingInstruction => {
                self.node_value.borrow().clone()
            }
            NodeType::Element | NodeType::DocumentFragment => {
                let mut text = String::new();
                self.collect_text_content(&mut text);
                Some(text)
            }
            _ => None,
        }
    }

    fn collect_text_content(&self, text: &mut String) {
        for child in self.children.borrow().iter() {
            let child_borrowed = child.borrow();
            if child_borrowed.node_type == NodeType::Text {
                if let Some(value) = child_borrowed.node_value.borrow().as_ref() {
                    text.push_str(value);
                }
            } else {
                child_borrowed.collect_text_content(text);
            }
        }
    }
}

// ============================================================================
// JavaScript Binding Functions
// ============================================================================

fn node_type_getter(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    // Stub - would extract from native data
    Ok(JsValue::from(1i32))
}

fn node_name_getter(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(js_string!("")))
}

fn has_child_nodes_method(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(false))
}

fn clone_node_method(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _deep: bool = get_arg_with_default(args, 0, false, ctx)?;
    Ok(JsValue::null())
}

fn contains_method(_this: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    if args.get_or_undefined(0).is_null_or_undefined() {
        return Ok(JsValue::from(false));
    }
    Ok(JsValue::from(false))
}

/// Get or create the Node prototype
pub fn get_node_prototype(ctx: &mut Context) -> JsResult<JsObject> {
    let global = ctx.global_object();
    let ctor = global.get(js_string!("Node"), ctx)?;
    
    if ctor.is_object() {
        let ctor_obj = ctor.to_object(ctx)?;
        let proto = ctor_obj.get(js_string!("prototype"), ctx)?;
        if proto.is_object() {
            return proto.to_object(ctx);
        }
    }
    
    register_node(ctx)
}

/// Register the Node interface on the global object
pub fn register_node(ctx: &mut Context) -> JsResult<JsObject> {
    let event_target_proto = super::event_target::register_event_target(ctx)?;
    
    InterfaceBuilder::with_parent(ctx, "Node", event_target_proto)
        // Constants
        .constant("ELEMENT_NODE", ConstantValue::Integer(1))
        .constant("ATTRIBUTE_NODE", ConstantValue::Integer(2))
        .constant("TEXT_NODE", ConstantValue::Integer(3))
        .constant("CDATA_SECTION_NODE", ConstantValue::Integer(4))
        .constant("PROCESSING_INSTRUCTION_NODE", ConstantValue::Integer(7))
        .constant("COMMENT_NODE", ConstantValue::Integer(8))
        .constant("DOCUMENT_NODE", ConstantValue::Integer(9))
        .constant("DOCUMENT_TYPE_NODE", ConstantValue::Integer(10))
        .constant("DOCUMENT_FRAGMENT_NODE", ConstantValue::Integer(11))
        .constant("DOCUMENT_POSITION_DISCONNECTED", ConstantValue::Integer(0x01))
        .constant("DOCUMENT_POSITION_PRECEDING", ConstantValue::Integer(0x02))
        .constant("DOCUMENT_POSITION_FOLLOWING", ConstantValue::Integer(0x04))
        .constant("DOCUMENT_POSITION_CONTAINS", ConstantValue::Integer(0x08))
        .constant("DOCUMENT_POSITION_CONTAINED_BY", ConstantValue::Integer(0x10))
        .constant("DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC", ConstantValue::Integer(0x20))
        // Attributes
        .readonly_attribute("nodeType", node_type_getter)
        .readonly_attribute("nodeName", node_name_getter)
        // Methods
        .method("hasChildNodes", has_child_nodes_method, 0)
        .method("cloneNode", clone_node_method, 0)
        .method("contains", contains_method, 1)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_new() {
        let node = Node::new(NodeType::Element, "DIV");
        assert_eq!(node.node_type(), NodeType::Element);
        assert_eq!(node.node_name(), "DIV");
        assert!(!node.has_child_nodes());
    }

    #[test]
    fn test_text_node() {
        let text = Node::new_text("Hello, World!");
        assert_eq!(text.node_type(), NodeType::Text);
        assert_eq!(text.node_name(), "#text");
        assert_eq!(text.node_value(), Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_append_child() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Element, "DIV")));
        let child = Rc::new(RefCell::new(Node::new(NodeType::Element, "SPAN")));
        
        Node::append_child(&parent, child.clone()).unwrap();
        
        assert!(parent.borrow().has_child_nodes());
        assert!(child.borrow().parent_node().is_some());
    }

    #[test]
    fn test_remove_child() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Element, "DIV")));
        let child = Rc::new(RefCell::new(Node::new(NodeType::Element, "SPAN")));
        
        Node::append_child(&parent, child.clone()).unwrap();
        Node::remove_child(&parent, child.clone()).unwrap();
        
        assert!(!parent.borrow().has_child_nodes());
        assert!(child.borrow().parent_node().is_none());
    }

    #[test]
    fn test_register_node() {
        let mut ctx = Context::default();
        register_node(&mut ctx).unwrap();
        
        let global = ctx.global_object();
        let ctor = global.get(js_string!("Node"), &mut ctx).unwrap();
        assert!(ctor.is_object());
    }
}
