//! Element Interface Binding for Boa
//!
//! The Element interface represents an element in an HTML or XML document.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use boa_engine::{
    Context, JsObject, JsResult, JsValue, js_string,
};

use super::node::{Node, NodeType};

/// Element implementation
pub struct Element {
    /// Base Node
    node: Node,
    
    /// Namespace URI
    namespace_uri: Option<String>,
    
    /// Namespace prefix
    prefix: Option<String>,
    
    /// Local name
    local_name: String,
    
    /// Element attributes
    attributes: RefCell<HashMap<String, String>>,
    
    /// Namespaced attributes (namespace, localName) -> value
    ns_attributes: RefCell<HashMap<(String, String), String>>,
}

impl Element {
    /// Create a new Element
    pub fn new(local_name: &str, namespace_uri: Option<&str>, prefix: Option<&str>) -> Self {
        let tag_name = if let Some(p) = prefix {
            format!("{}:{}", p, local_name)
        } else {
            local_name.to_string()
        };

        Element {
            node: Node::new(NodeType::Element, &tag_name.to_uppercase()),
            namespace_uri: namespace_uri.map(String::from),
            prefix: prefix.map(String::from),
            local_name: local_name.to_string(),
            attributes: RefCell::new(HashMap::new()),
            ns_attributes: RefCell::new(HashMap::new()),
        }
    }

    /// Create a new HTML element
    pub fn new_html(tag_name: &str) -> Self {
        Element::new(
            &tag_name.to_lowercase(),
            Some("http://www.w3.org/1999/xhtml"),
            None,
        )
    }

    /// Get the namespace URI
    pub fn namespace_uri(&self) -> Option<&str> {
        self.namespace_uri.as_deref()
    }

    /// Get the namespace prefix
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// Get the local name
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Get the tag name (uppercase for HTML elements)
    pub fn tag_name(&self) -> &str {
        self.node.node_name()
    }

    /// Get the id attribute
    pub fn id(&self) -> String {
        self.attributes.borrow().get("id").cloned().unwrap_or_default()
    }

    /// Set the id attribute
    pub fn set_id(&self, id: &str) {
        self.set_attribute("id", id);
    }

    /// Get the className attribute
    pub fn class_name(&self) -> String {
        self.attributes.borrow().get("class").cloned().unwrap_or_default()
    }

    /// Set the className attribute
    pub fn set_class_name(&self, class_name: &str) {
        self.set_attribute("class", class_name);
    }

    /// Check if element has any attributes
    pub fn has_attributes(&self) -> bool {
        !self.attributes.borrow().is_empty() || !self.ns_attributes.borrow().is_empty()
    }

    /// Get all attribute names
    pub fn get_attribute_names(&self) -> Vec<String> {
        self.attributes.borrow().keys().cloned().collect()
    }

    /// Get an attribute value
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes.borrow().get(&name.to_lowercase()).cloned()
    }

    /// Get an attribute value with namespace
    pub fn get_attribute_ns(&self, namespace: Option<&str>, local_name: &str) -> Option<String> {
        let key = (
            namespace.unwrap_or("").to_string(),
            local_name.to_string(),
        );
        self.ns_attributes.borrow().get(&key).cloned()
    }

    /// Set an attribute
    pub fn set_attribute(&self, name: &str, value: &str) {
        self.attributes.borrow_mut().insert(name.to_lowercase(), value.to_string());
    }

    /// Set an attribute with namespace
    pub fn set_attribute_ns(&self, namespace: Option<&str>, qualified_name: &str, value: &str) {
        let local_name = qualified_name.split(':').last().unwrap_or(qualified_name);
        let key = (
            namespace.unwrap_or("").to_string(),
            local_name.to_string(),
        );
        self.ns_attributes.borrow_mut().insert(key, value.to_string());
        
        // Also set in regular attributes for simple access
        self.attributes.borrow_mut().insert(local_name.to_string(), value.to_string());
    }

    /// Remove an attribute
    pub fn remove_attribute(&self, name: &str) {
        self.attributes.borrow_mut().remove(&name.to_lowercase());
    }

    /// Remove an attribute with namespace
    pub fn remove_attribute_ns(&self, namespace: Option<&str>, local_name: &str) {
        let key = (
            namespace.unwrap_or("").to_string(),
            local_name.to_string(),
        );
        self.ns_attributes.borrow_mut().remove(&key);
    }

    /// Check if attribute exists
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.borrow().contains_key(&name.to_lowercase())
    }

    /// Check if attribute exists with namespace
    pub fn has_attribute_ns(&self, namespace: Option<&str>, local_name: &str) -> bool {
        let key = (
            namespace.unwrap_or("").to_string(),
            local_name.to_string(),
        );
        self.ns_attributes.borrow().contains_key(&key)
    }

    /// Get innerHTML (stub implementation)
    pub fn inner_html(&self) -> String {
        String::new()
    }

    /// Set innerHTML (stub implementation)
    pub fn set_inner_html(&self, _html: &str) {
        // Stub: would parse HTML and update children
    }

    /// Get outerHTML (stub implementation)
    pub fn outer_html(&self) -> String {
        let mut result = format!("<{}", self.tag_name());
        for (name, value) in self.attributes.borrow().iter() {
            result.push_str(&format!(" {}=\"{}\"", name, value));
        }
        result.push_str(">");
        result.push_str(&self.inner_html());
        result.push_str(&format!("</{}>", self.tag_name()));
        result
    }

    /// Access the underlying Node
    pub fn as_node(&self) -> &Node {
        &self.node
    }

    /// Access the underlying Node mutably
    pub fn as_node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    /// Check if element matches a CSS selector (stub)
    pub fn matches(&self, _selector: &str) -> bool {
        false
    }

    /// Find closest ancestor matching selector (stub)
    pub fn closest(&self, _selector: &str) -> Option<Rc<RefCell<Element>>> {
        None
    }
}

/// Initialize Element prototype on the global object
/// 
/// Note: This creates a minimal stub prototype. Full implementation
/// would need proper native data storage pattern.
pub fn init_element_prototype(context: &mut Context) -> JsResult<JsObject> {
    let prototype = JsObject::with_null_proto();
    
    // Set simple value properties
    prototype.set(js_string!("namespaceURI"), JsValue::null(), false, context)?;
    prototype.set(js_string!("localName"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("tagName"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("id"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("className"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("innerHTML"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("outerHTML"), JsValue::from(js_string!("")), false, context)?;
    
    // Set stub method objects - these are placeholders
    // Full implementation would use proper function objects
    let empty_func = JsObject::with_null_proto();
    prototype.set(js_string!("hasAttributes"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("getAttributeNames"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("getAttribute"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("setAttribute"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("removeAttribute"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("hasAttribute"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("matches"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("closest"), JsValue::from(empty_func), false, context)?;

    Ok(prototype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let element = Element::new("div", None, None);
        assert_eq!(element.tag_name(), "DIV");
        assert_eq!(element.local_name(), "div");
    }

    #[test]
    fn test_html_element_creation() {
        let element = Element::new_html("DIV");
        assert_eq!(element.tag_name(), "DIV");
        assert_eq!(element.local_name(), "div");
        assert_eq!(element.namespace_uri(), Some("http://www.w3.org/1999/xhtml"));
    }

    #[test]
    fn test_element_with_namespace() {
        let element = Element::new(
            "rect",
            Some("http://www.w3.org/2000/svg"),
            Some("svg"),
        );
        assert_eq!(element.tag_name(), "SVG:RECT");
        assert_eq!(element.local_name(), "rect");
        assert_eq!(element.namespace_uri(), Some("http://www.w3.org/2000/svg"));
        assert_eq!(element.prefix(), Some("svg"));
    }

    #[test]
    fn test_attributes() {
        let element = Element::new_html("div");
        
        // Set and get
        element.set_attribute("data-test", "value");
        assert_eq!(element.get_attribute("data-test"), Some("value".to_string()));
        
        // Has attribute
        assert!(element.has_attribute("data-test"));
        assert!(!element.has_attribute("nonexistent"));
        
        // Remove
        element.remove_attribute("data-test");
        assert!(!element.has_attribute("data-test"));
    }

    #[test]
    fn test_id_attribute() {
        let element = Element::new_html("div");
        
        element.set_id("my-element");
        assert_eq!(element.id(), "my-element");
        assert_eq!(element.get_attribute("id"), Some("my-element".to_string()));
    }

    #[test]
    fn test_class_name_attribute() {
        let element = Element::new_html("div");
        
        element.set_class_name("foo bar baz");
        assert_eq!(element.class_name(), "foo bar baz");
        assert_eq!(element.get_attribute("class"), Some("foo bar baz".to_string()));
    }

    #[test]
    fn test_namespaced_attributes() {
        let element = Element::new_html("div");
        
        element.set_attribute_ns(
            Some("http://www.w3.org/1999/xlink"),
            "xlink:href",
            "#target",
        );
        
        assert!(element.has_attribute_ns(Some("http://www.w3.org/1999/xlink"), "href"));
        assert_eq!(
            element.get_attribute_ns(Some("http://www.w3.org/1999/xlink"), "href"),
            Some("#target".to_string())
        );
    }

    #[test]
    fn test_attribute_names() {
        let element = Element::new_html("div");
        element.set_attribute("id", "test");
        element.set_attribute("class", "foo");
        element.set_attribute("data-value", "123");
        
        let names = element.get_attribute_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"id".to_string()));
        assert!(names.contains(&"class".to_string()));
        assert!(names.contains(&"data-value".to_string()));
    }

    #[test]
    fn test_outer_html() {
        let element = Element::new_html("div");
        element.set_id("test");
        element.set_class_name("container");
        
        let html = element.outer_html();
        assert!(html.contains("<DIV"));
        assert!(html.contains("</DIV>"));
    }
}
