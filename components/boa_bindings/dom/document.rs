//! Document Interface Binding for Boa
//!
//! The Document interface represents the entire HTML or XML document.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use boa_engine::{
    Context, JsObject, JsResult, JsValue, js_string,
};

use super::node::{Node, NodeType};
use super::element::Element;

/// Document implementation
pub struct Document {
    /// Base Node
    node: Node,
    
    /// Document URL
    url: RefCell<String>,
    
    /// Character encoding
    character_set: String,
    
    /// Content type (MIME type)
    content_type: String,
    
    /// Compatibility mode
    compat_mode: String,
    
    /// Document element (root element)
    document_element: RefCell<Option<Rc<RefCell<Element>>>>,
    
    /// All elements by ID
    elements_by_id: RefCell<HashMap<String, Rc<RefCell<Element>>>>,
}

impl Document {
    /// Create a new Document
    pub fn new() -> Self {
        Document {
            node: Node::new(NodeType::Document, "#document"),
            url: RefCell::new("about:blank".to_string()),
            character_set: "UTF-8".to_string(),
            content_type: "text/html".to_string(),
            compat_mode: "CSS1Compat".to_string(),
            document_element: RefCell::new(None),
            elements_by_id: RefCell::new(HashMap::new()),
        }
    }

    /// Create a new HTML document
    pub fn new_html(url: &str) -> Self {
        let doc = Document::new();
        *doc.url.borrow_mut() = url.to_string();
        doc
    }

    /// Get the document URL
    pub fn url(&self) -> String {
        self.url.borrow().clone()
    }

    /// Get the document URI (alias for URL)
    pub fn document_uri(&self) -> String {
        self.url()
    }

    /// Get the character set
    pub fn character_set(&self) -> &str {
        &self.character_set
    }

    /// Get the content type
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    /// Get the compatibility mode
    pub fn compat_mode(&self) -> &str {
        &self.compat_mode
    }

    /// Get the document element (root element)
    pub fn document_element(&self) -> Option<Rc<RefCell<Element>>> {
        self.document_element.borrow().clone()
    }

    /// Set the document element
    pub fn set_document_element(&self, element: Rc<RefCell<Element>>) {
        *self.document_element.borrow_mut() = Some(element);
    }

    /// Access the underlying Node
    pub fn as_node(&self) -> &Node {
        &self.node
    }

    /// Access the underlying Node mutably
    pub fn as_node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    /// Create a new element
    pub fn create_element(&self, tag_name: &str) -> Element {
        Element::new_html(tag_name)
    }

    /// Create a new element with namespace
    pub fn create_element_ns(&self, namespace: Option<&str>, qualified_name: &str) -> Element {
        let (prefix, local_name) = if let Some(colon_pos) = qualified_name.find(':') {
            (Some(&qualified_name[..colon_pos]), &qualified_name[colon_pos + 1..])
        } else {
            (None, qualified_name)
        };
        Element::new(local_name, namespace, prefix)
    }

    /// Create a new text node
    pub fn create_text_node(&self, data: &str) -> Node {
        Node::new_text(data)
    }

    /// Create a new comment node
    pub fn create_comment(&self, data: &str) -> Node {
        Node::new_comment(data)
    }

    /// Register an element by ID
    pub fn register_element_by_id(&self, id: &str, element: Rc<RefCell<Element>>) {
        self.elements_by_id.borrow_mut().insert(id.to_string(), element);
    }

    /// Unregister an element by ID
    pub fn unregister_element_by_id(&self, id: &str) {
        self.elements_by_id.borrow_mut().remove(id);
    }

    /// Get element by ID
    pub fn get_element_by_id(&self, id: &str) -> Option<Rc<RefCell<Element>>> {
        self.elements_by_id.borrow().get(id).cloned()
    }

    /// Query selector (stub)
    pub fn query_selector(&self, _selector: &str) -> Option<Rc<RefCell<Element>>> {
        None
    }

    /// Query selector all (stub)
    pub fn query_selector_all(&self, _selector: &str) -> Vec<Rc<RefCell<Element>>> {
        Vec::new()
    }

    /// Get elements by tag name (stub)
    pub fn get_elements_by_tag_name(&self, _tag_name: &str) -> Vec<Rc<RefCell<Element>>> {
        Vec::new()
    }

    /// Get elements by class name (stub)
    pub fn get_elements_by_class_name(&self, _class_names: &str) -> Vec<Rc<RefCell<Element>>> {
        Vec::new()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize Document prototype on the global object
pub fn init_document_prototype(context: &mut Context) -> JsResult<JsObject> {
    let prototype = JsObject::with_null_proto();

    // Set simple value properties
    prototype.set(js_string!("URL"), JsValue::from(js_string!("about:blank")), false, context)?;
    prototype.set(js_string!("documentURI"), JsValue::from(js_string!("about:blank")), false, context)?;
    prototype.set(js_string!("characterSet"), JsValue::from(js_string!("UTF-8")), false, context)?;
    prototype.set(js_string!("contentType"), JsValue::from(js_string!("text/html")), false, context)?;
    prototype.set(js_string!("compatMode"), JsValue::from(js_string!("CSS1Compat")), false, context)?;
    prototype.set(js_string!("documentElement"), JsValue::null(), false, context)?;

    // Set stub method objects
    let empty_func = JsObject::with_null_proto();
    prototype.set(js_string!("createElement"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("createElementNS"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("createTextNode"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("createComment"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("getElementById"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("querySelector"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("querySelectorAll"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("getElementsByTagName"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("getElementsByClassName"), JsValue::from(empty_func), false, context)?;

    Ok(prototype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new();
        assert_eq!(doc.url(), "about:blank");
        assert_eq!(doc.character_set(), "UTF-8");
        assert_eq!(doc.content_type(), "text/html");
        assert_eq!(doc.compat_mode(), "CSS1Compat");
    }

    #[test]
    fn test_html_document_creation() {
        let doc = Document::new_html("https://example.com/");
        assert_eq!(doc.url(), "https://example.com/");
    }

    #[test]
    fn test_document_uri() {
        let doc = Document::new_html("https://example.com/page.html");
        assert_eq!(doc.document_uri(), "https://example.com/page.html");
    }

    #[test]
    fn test_create_element() {
        let doc = Document::new();
        let element = doc.create_element("div");
        assert_eq!(element.tag_name(), "DIV");
        assert_eq!(element.local_name(), "div");
    }

    #[test]
    fn test_create_element_ns() {
        let doc = Document::new();
        let element = doc.create_element_ns(
            Some("http://www.w3.org/2000/svg"),
            "svg:rect",
        );
        assert_eq!(element.local_name(), "rect");
        assert_eq!(element.namespace_uri(), Some("http://www.w3.org/2000/svg"));
    }

    #[test]
    fn test_create_text_node() {
        let doc = Document::new();
        let text = doc.create_text_node("Hello, World!");
        assert_eq!(text.node_value(), Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_create_comment() {
        let doc = Document::new();
        let comment = doc.create_comment("This is a comment");
        assert_eq!(comment.node_value(), Some("This is a comment".to_string()));
    }

    #[test]
    fn test_document_element() {
        let doc = Document::new();
        assert!(doc.document_element().is_none());

        let html_element = Rc::new(RefCell::new(Element::new_html("html")));
        doc.set_document_element(html_element.clone());
        
        assert!(doc.document_element().is_some());
        assert_eq!(doc.document_element().unwrap().borrow().tag_name(), "HTML");
    }

    #[test]
    fn test_element_by_id() {
        let doc = Document::new();
        let element = Rc::new(RefCell::new(Element::new_html("div")));
        element.borrow().set_id("test-id");
        
        doc.register_element_by_id("test-id", element.clone());
        
        let found = doc.get_element_by_id("test-id");
        assert!(found.is_some());
        assert_eq!(found.unwrap().borrow().id(), "test-id");
        
        doc.unregister_element_by_id("test-id");
        assert!(doc.get_element_by_id("test-id").is_none());
    }

    #[test]
    fn test_query_selector_stub() {
        let doc = Document::new();
        assert!(doc.query_selector(".test").is_none());
        assert!(doc.query_selector_all(".test").is_empty());
    }

    #[test]
    fn test_get_elements_by_stub() {
        let doc = Document::new();
        assert!(doc.get_elements_by_tag_name("div").is_empty());
        assert!(doc.get_elements_by_class_name("test").is_empty());
    }
}
