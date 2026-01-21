//! HTMLElement Interface Binding for Boa
//!
//! The HTMLElement interface represents any HTML element.
//! It extends Element with HTML-specific properties and methods.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use boa_engine::{
    Context, JsObject, JsResult, JsValue, js_string,
};

use super::element::Element;

/// DOMStringMap for dataset property
#[derive(Debug, Clone, Default)]
pub struct DOMStringMap {
    data: RefCell<HashMap<String, String>>,
}

impl DOMStringMap {
    pub fn new() -> Self {
        DOMStringMap {
            data: RefCell::new(HashMap::new()),
        }
    }

    /// Get a data attribute value
    pub fn get(&self, name: &str) -> Option<String> {
        self.data.borrow().get(name).cloned()
    }

    /// Set a data attribute value
    pub fn set(&self, name: &str, value: &str) {
        self.data.borrow_mut().insert(name.to_string(), value.to_string());
    }

    /// Remove a data attribute
    pub fn remove(&self, name: &str) -> bool {
        self.data.borrow_mut().remove(name).is_some()
    }

    /// Check if a data attribute exists
    pub fn has(&self, name: &str) -> bool {
        self.data.borrow().contains_key(name)
    }

    /// Get all data attribute names
    pub fn keys(&self) -> Vec<String> {
        self.data.borrow().keys().cloned().collect()
    }
}

/// CSSStyleDeclaration for inline styles
#[derive(Debug, Clone, Default)]
pub struct CSSStyleDeclaration {
    properties: RefCell<HashMap<String, String>>,
    priority: RefCell<HashMap<String, String>>,
}

impl CSSStyleDeclaration {
    pub fn new() -> Self {
        CSSStyleDeclaration {
            properties: RefCell::new(HashMap::new()),
            priority: RefCell::new(HashMap::new()),
        }
    }

    /// Get the CSS text representation
    pub fn css_text(&self) -> String {
        let props = self.properties.borrow();
        let prio = self.priority.borrow();
        props.iter()
            .map(|(name, value)| {
                let priority = prio.get(name).map(|p| format!(" !{}", p)).unwrap_or_default();
                format!("{}: {}{};", name, value, priority)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Set CSS text (parses and replaces all properties)
    pub fn set_css_text(&self, text: &str) {
        self.properties.borrow_mut().clear();
        self.priority.borrow_mut().clear();
        
        for declaration in text.split(';') {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                continue;
            }
            
            if let Some((name, value)) = declaration.split_once(':') {
                let name = name.trim();
                let mut value = value.trim().to_string();
                let mut priority = String::new();
                
                if value.ends_with("!important") {
                    value = value.trim_end_matches("!important").trim().to_string();
                    priority = "important".to_string();
                }
                
                self.properties.borrow_mut().insert(name.to_string(), value);
                if !priority.is_empty() {
                    self.priority.borrow_mut().insert(name.to_string(), priority);
                }
            }
        }
    }

    /// Get a property value
    pub fn get_property_value(&self, property: &str) -> String {
        self.properties.borrow().get(property).cloned().unwrap_or_default()
    }

    /// Get property priority
    pub fn get_property_priority(&self, property: &str) -> String {
        self.priority.borrow().get(property).cloned().unwrap_or_default()
    }

    /// Set a property
    pub fn set_property(&self, property: &str, value: &str, priority: &str) {
        if value.is_empty() {
            self.remove_property(property);
        } else {
            self.properties.borrow_mut().insert(property.to_string(), value.to_string());
            if !priority.is_empty() {
                self.priority.borrow_mut().insert(property.to_string(), priority.to_string());
            } else {
                self.priority.borrow_mut().remove(property);
            }
        }
    }

    /// Remove a property
    pub fn remove_property(&self, property: &str) -> String {
        self.priority.borrow_mut().remove(property);
        self.properties.borrow_mut().remove(property).unwrap_or_default()
    }

    /// Get number of properties
    pub fn length(&self) -> usize {
        self.properties.borrow().len()
    }

    /// Get property name by index
    pub fn item(&self, index: usize) -> Option<String> {
        self.properties.borrow().keys().nth(index).cloned()
    }
}

/// DOMTokenList for classList
#[derive(Debug, Clone, Default)]
pub struct DOMTokenList {
    tokens: RefCell<Vec<String>>,
}

impl DOMTokenList {
    pub fn new() -> Self {
        DOMTokenList {
            tokens: RefCell::new(Vec::new()),
        }
    }

    /// Create from space-separated string
    pub fn from_string(value: &str) -> Self {
        let list = DOMTokenList::new();
        for token in value.split_whitespace() {
            if !token.is_empty() {
                list.tokens.borrow_mut().push(token.to_string());
            }
        }
        list
    }

    /// Get the string value
    pub fn value(&self) -> String {
        self.tokens.borrow().join(" ")
    }

    /// Set the string value
    pub fn set_value(&self, value: &str) {
        let mut tokens = self.tokens.borrow_mut();
        tokens.clear();
        for token in value.split_whitespace() {
            if !token.is_empty() {
                tokens.push(token.to_string());
            }
        }
    }

    /// Get length
    pub fn length(&self) -> usize {
        self.tokens.borrow().len()
    }

    /// Get item at index
    pub fn item(&self, index: usize) -> Option<String> {
        self.tokens.borrow().get(index).cloned()
    }

    /// Check if token exists
    pub fn contains(&self, token: &str) -> bool {
        self.tokens.borrow().iter().any(|t| t == token)
    }

    /// Add tokens
    pub fn add(&self, tokens: &[&str]) {
        let mut list = self.tokens.borrow_mut();
        for token in tokens {
            if !token.is_empty() && !list.iter().any(|t| t == *token) {
                list.push(token.to_string());
            }
        }
    }

    /// Remove tokens
    pub fn remove(&self, tokens: &[&str]) {
        let mut list = self.tokens.borrow_mut();
        list.retain(|t| !tokens.contains(&t.as_str()));
    }

    /// Toggle a token, returns true if token is now present
    pub fn toggle(&self, token: &str, force: Option<bool>) -> bool {
        match force {
            Some(true) => {
                self.add(&[token]);
                true
            }
            Some(false) => {
                self.remove(&[token]);
                false
            }
            None => {
                if self.contains(token) {
                    self.remove(&[token]);
                    false
                } else {
                    self.add(&[token]);
                    true
                }
            }
        }
    }

    /// Replace a token
    pub fn replace(&self, old_token: &str, new_token: &str) -> bool {
        let mut list = self.tokens.borrow_mut();
        if let Some(pos) = list.iter().position(|t| t == old_token) {
            list[pos] = new_token.to_string();
            true
        } else {
            false
        }
    }

    /// Check if token is supported (for feature detection)
    pub fn supports(&self, _token: &str) -> bool {
        // Stub - would check if token is valid for this attribute
        true
    }
}

/// HTMLElement implementation
pub struct HTMLElement {
    /// Base Element
    element: Rc<RefCell<Element>>,
    
    /// Inline style
    style: CSSStyleDeclaration,
    
    /// Dataset (data-* attributes)
    dataset: DOMStringMap,
    
    /// Class list
    class_list: DOMTokenList,
    
    /// Hidden attribute
    hidden: RefCell<bool>,
    
    /// Tab index
    tab_index: RefCell<i32>,
    
    /// Access key
    access_key: RefCell<String>,
    
    /// Content editable
    content_editable: RefCell<String>,
    
    /// Draggable
    draggable: RefCell<bool>,
    
    /// Spellcheck
    spellcheck: RefCell<bool>,
    
    /// Title
    title: RefCell<String>,
    
    /// Language
    lang: RefCell<String>,
    
    /// Direction
    dir: RefCell<String>,
    
    /// Inner text
    inner_text: RefCell<String>,
}

impl HTMLElement {
    /// Create a new HTMLElement
    pub fn new(tag_name: &str) -> Self {
        let element = Rc::new(RefCell::new(Element::new_html(tag_name)));
        let class_attr = element.borrow().get_attribute("class").unwrap_or_default();
        
        HTMLElement {
            element,
            style: CSSStyleDeclaration::new(),
            dataset: DOMStringMap::new(),
            class_list: DOMTokenList::from_string(&class_attr),
            hidden: RefCell::new(false),
            tab_index: RefCell::new(-1),
            access_key: RefCell::new(String::new()),
            content_editable: RefCell::new("inherit".to_string()),
            draggable: RefCell::new(false),
            spellcheck: RefCell::new(true),
            title: RefCell::new(String::new()),
            lang: RefCell::new(String::new()),
            dir: RefCell::new(String::new()),
            inner_text: RefCell::new(String::new()),
        }
    }

    /// Get the underlying Element
    pub fn element(&self) -> Rc<RefCell<Element>> {
        self.element.clone()
    }

    /// Get the tag name
    pub fn tag_name(&self) -> String {
        self.element.borrow().tag_name().to_string()
    }

    /// Get the style object
    pub fn style(&self) -> &CSSStyleDeclaration {
        &self.style
    }

    /// Get the dataset
    pub fn dataset(&self) -> &DOMStringMap {
        &self.dataset
    }

    /// Get the classList
    pub fn class_list(&self) -> &DOMTokenList {
        &self.class_list
    }

    /// Get hidden attribute
    pub fn hidden(&self) -> bool {
        *self.hidden.borrow()
    }

    /// Set hidden attribute
    pub fn set_hidden(&self, hidden: bool) {
        *self.hidden.borrow_mut() = hidden;
    }

    /// Get tabIndex
    pub fn tab_index(&self) -> i32 {
        *self.tab_index.borrow()
    }

    /// Set tabIndex
    pub fn set_tab_index(&self, index: i32) {
        *self.tab_index.borrow_mut() = index;
    }

    /// Get accessKey
    pub fn access_key(&self) -> String {
        self.access_key.borrow().clone()
    }

    /// Set accessKey
    pub fn set_access_key(&self, key: &str) {
        *self.access_key.borrow_mut() = key.to_string();
    }

    /// Get contentEditable
    pub fn content_editable(&self) -> String {
        self.content_editable.borrow().clone()
    }

    /// Set contentEditable
    pub fn set_content_editable(&self, value: &str) {
        *self.content_editable.borrow_mut() = value.to_string();
    }

    /// Check if content is editable
    pub fn is_content_editable(&self) -> bool {
        self.content_editable.borrow().as_str() == "true"
    }

    /// Get draggable
    pub fn draggable(&self) -> bool {
        *self.draggable.borrow()
    }

    /// Set draggable
    pub fn set_draggable(&self, value: bool) {
        *self.draggable.borrow_mut() = value;
    }

    /// Get spellcheck
    pub fn spellcheck(&self) -> bool {
        *self.spellcheck.borrow()
    }

    /// Set spellcheck
    pub fn set_spellcheck(&self, value: bool) {
        *self.spellcheck.borrow_mut() = value;
    }

    /// Get title
    pub fn title(&self) -> String {
        self.title.borrow().clone()
    }

    /// Set title
    pub fn set_title(&self, value: &str) {
        *self.title.borrow_mut() = value.to_string();
    }

    /// Get lang
    pub fn lang(&self) -> String {
        self.lang.borrow().clone()
    }

    /// Set lang
    pub fn set_lang(&self, value: &str) {
        *self.lang.borrow_mut() = value.to_string();
    }

    /// Get dir
    pub fn dir(&self) -> String {
        self.dir.borrow().clone()
    }

    /// Set dir
    pub fn set_dir(&self, value: &str) {
        *self.dir.borrow_mut() = value.to_string();
    }

    /// Get innerText
    pub fn inner_text(&self) -> String {
        self.inner_text.borrow().clone()
    }

    /// Set innerText
    pub fn set_inner_text(&self, value: &str) {
        *self.inner_text.borrow_mut() = value.to_string();
    }

    /// Focus the element (stub)
    pub fn focus(&self) {
        // Stub - would need integration with focus manager
    }

    /// Blur the element (stub)
    pub fn blur(&self) {
        // Stub - would need integration with focus manager
    }

    /// Click the element (stub)
    pub fn click(&self) {
        // Stub - would dispatch click event
    }

    /// Get offset properties (stubs)
    pub fn offset_top(&self) -> i32 { 0 }
    pub fn offset_left(&self) -> i32 { 0 }
    pub fn offset_width(&self) -> i32 { 0 }
    pub fn offset_height(&self) -> i32 { 0 }
    pub fn offset_parent(&self) -> Option<()> { None }

    /// Get client properties (stubs)
    pub fn client_top(&self) -> i32 { 0 }
    pub fn client_left(&self) -> i32 { 0 }
    pub fn client_width(&self) -> i32 { 0 }
    pub fn client_height(&self) -> i32 { 0 }

    /// Get scroll properties (stubs)
    pub fn scroll_top(&self) -> i32 { 0 }
    pub fn set_scroll_top(&self, _value: i32) {}
    pub fn scroll_left(&self) -> i32 { 0 }
    pub fn set_scroll_left(&self, _value: i32) {}
    pub fn scroll_width(&self) -> i32 { 0 }
    pub fn scroll_height(&self) -> i32 { 0 }
}

/// Initialize HTMLElement prototype on a JsObject
pub fn init_html_element_prototype(context: &mut Context) -> JsResult<JsObject> {
    let prototype = JsObject::with_null_proto();

    // Set simple value properties
    prototype.set(js_string!("hidden"), JsValue::from(false), false, context)?;
    prototype.set(js_string!("tabIndex"), JsValue::from(-1), false, context)?;
    prototype.set(js_string!("accessKey"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("contentEditable"), JsValue::from(js_string!("inherit")), false, context)?;
    prototype.set(js_string!("isContentEditable"), JsValue::from(false), false, context)?;
    prototype.set(js_string!("draggable"), JsValue::from(false), false, context)?;
    prototype.set(js_string!("spellcheck"), JsValue::from(true), false, context)?;
    prototype.set(js_string!("title"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("lang"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("dir"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("innerText"), JsValue::from(js_string!("")), false, context)?;
    prototype.set(js_string!("outerText"), JsValue::from(js_string!("")), false, context)?;

    // Offset properties (stubs)
    prototype.set(js_string!("offsetTop"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("offsetLeft"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("offsetWidth"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("offsetHeight"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("offsetParent"), JsValue::null(), false, context)?;

    // Client properties (stubs)
    prototype.set(js_string!("clientTop"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("clientLeft"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("clientWidth"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("clientHeight"), JsValue::from(0), false, context)?;

    // Scroll properties (stubs)
    prototype.set(js_string!("scrollTop"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("scrollLeft"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("scrollWidth"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("scrollHeight"), JsValue::from(0), false, context)?;

    // Style object (stub)
    let style = JsObject::with_null_proto();
    style.set(js_string!("cssText"), JsValue::from(js_string!("")), false, context)?;
    style.set(js_string!("length"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("style"), JsValue::from(style), false, context)?;

    // Dataset object (stub)
    let dataset = JsObject::with_null_proto();
    prototype.set(js_string!("dataset"), JsValue::from(dataset), false, context)?;

    // ClassList object (stub)
    let class_list = JsObject::with_null_proto();
    class_list.set(js_string!("value"), JsValue::from(js_string!("")), false, context)?;
    class_list.set(js_string!("length"), JsValue::from(0), false, context)?;
    prototype.set(js_string!("classList"), JsValue::from(class_list), false, context)?;

    // Set stub method objects
    let empty_func = JsObject::with_null_proto();
    prototype.set(js_string!("focus"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("blur"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("click"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("scrollIntoView"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("scroll"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("scrollTo"), JsValue::from(empty_func.clone()), false, context)?;
    prototype.set(js_string!("scrollBy"), JsValue::from(empty_func), false, context)?;

    Ok(prototype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_element_creation() {
        let elem = HTMLElement::new("div");
        assert_eq!(elem.tag_name(), "DIV");
    }

    #[test]
    fn test_hidden_attribute() {
        let elem = HTMLElement::new("div");
        assert!(!elem.hidden());
        elem.set_hidden(true);
        assert!(elem.hidden());
    }

    #[test]
    fn test_tab_index() {
        let elem = HTMLElement::new("button");
        assert_eq!(elem.tab_index(), -1);
        elem.set_tab_index(0);
        assert_eq!(elem.tab_index(), 0);
    }

    #[test]
    fn test_content_editable() {
        let elem = HTMLElement::new("div");
        assert_eq!(elem.content_editable(), "inherit");
        assert!(!elem.is_content_editable());
        elem.set_content_editable("true");
        assert!(elem.is_content_editable());
    }

    #[test]
    fn test_title_lang_dir() {
        let elem = HTMLElement::new("div");
        elem.set_title("My Title");
        elem.set_lang("en");
        elem.set_dir("ltr");
        
        assert_eq!(elem.title(), "My Title");
        assert_eq!(elem.lang(), "en");
        assert_eq!(elem.dir(), "ltr");
    }

    #[test]
    fn test_inner_text() {
        let elem = HTMLElement::new("p");
        elem.set_inner_text("Hello, World!");
        assert_eq!(elem.inner_text(), "Hello, World!");
    }

    #[test]
    fn test_css_style_declaration() {
        let style = CSSStyleDeclaration::new();
        
        style.set_property("color", "red", "");
        assert_eq!(style.get_property_value("color"), "red");
        
        style.set_property("font-size", "16px", "important");
        assert_eq!(style.get_property_priority("font-size"), "important");
        
        assert_eq!(style.length(), 2);
        
        let removed = style.remove_property("color");
        assert_eq!(removed, "red");
        assert_eq!(style.length(), 1);
    }

    #[test]
    fn test_css_text() {
        let style = CSSStyleDeclaration::new();
        style.set_css_text("color: blue; font-size: 14px");
        
        assert_eq!(style.get_property_value("color"), "blue");
        assert_eq!(style.get_property_value("font-size"), "14px");
    }

    #[test]
    fn test_dom_token_list() {
        let list = DOMTokenList::new();
        
        list.add(&["foo", "bar"]);
        assert!(list.contains("foo"));
        assert!(list.contains("bar"));
        assert_eq!(list.length(), 2);
        
        list.remove(&["foo"]);
        assert!(!list.contains("foo"));
        assert_eq!(list.length(), 1);
    }

    #[test]
    fn test_class_list_toggle() {
        let list = DOMTokenList::new();
        
        assert!(list.toggle("active", None)); // Add
        assert!(list.contains("active"));
        
        assert!(!list.toggle("active", None)); // Remove
        assert!(!list.contains("active"));
        
        assert!(list.toggle("forced", Some(true))); // Force add
        assert!(list.contains("forced"));
        
        assert!(list.toggle("forced", Some(true))); // Still there
        assert!(list.contains("forced"));
    }

    #[test]
    fn test_class_list_replace() {
        let list = DOMTokenList::from_string("old-class another");
        
        assert!(list.replace("old-class", "new-class"));
        assert!(list.contains("new-class"));
        assert!(!list.contains("old-class"));
        assert!(list.contains("another"));
    }

    #[test]
    fn test_dom_string_map() {
        let dataset = DOMStringMap::new();
        
        dataset.set("userId", "123");
        assert_eq!(dataset.get("userId"), Some("123".to_string()));
        assert!(dataset.has("userId"));
        
        dataset.remove("userId");
        assert!(!dataset.has("userId"));
    }

    #[test]
    fn test_html_element_class_list() {
        let elem = HTMLElement::new("div");
        elem.class_list().add(&["container", "flex"]);
        
        assert!(elem.class_list().contains("container"));
        assert!(elem.class_list().contains("flex"));
        assert_eq!(elem.class_list().value(), "container flex");
    }

    #[test]
    fn test_html_element_style() {
        let elem = HTMLElement::new("div");
        elem.style().set_property("display", "flex", "");
        elem.style().set_property("color", "blue", "");
        
        assert_eq!(elem.style().get_property_value("display"), "flex");
        assert_eq!(elem.style().get_property_value("color"), "blue");
    }

    #[test]
    fn test_html_element_dataset() {
        let elem = HTMLElement::new("div");
        elem.dataset().set("testValue", "hello");
        elem.dataset().set("count", "42");
        
        assert_eq!(elem.dataset().get("testValue"), Some("hello".to_string()));
        assert_eq!(elem.dataset().get("count"), Some("42".to_string()));
    }
}
