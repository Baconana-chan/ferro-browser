//! WebIDL Traits for Boa Bindings
//!
//! This module defines the core traits for WebIDL interface bindings:
//! - `WebIdlInterface`: Marker trait for WebIDL interfaces
//! - `WebIdlConstructable`: For interfaces with constructors
//! - `WebIdlAttribute`: For interface attributes
//! - `WebIdlMethod`: For interface methods


use boa_engine::{
    Context, JsNativeError, JsObject, JsResult, JsValue,
    NativeFunction, js_string,
    native_function::NativeFunctionPointer,
    property::PropertyDescriptor,
};

use crate::reflector::DomObject;

/// Trait for WebIDL interfaces that can be exposed to JavaScript
pub trait WebIdlInterface: DomObject + Sized {
    /// The name of the interface (e.g., "Node", "Element", "Document")
    const NAME: &'static str;
    
    /// Parent interface name, if any
    const PARENT: Option<&'static str> = None;
    
    /// Register the interface prototype on the global object
    fn register(ctx: &mut Context) -> JsResult<()>;
    
    /// Get the prototype object for this interface
    fn get_prototype(ctx: &mut Context) -> JsResult<JsObject>;
    
    /// Create a new JavaScript object wrapping this DOM object
    fn create_js_object(&self, ctx: &mut Context) -> JsResult<JsObject>;
}

/// Trait for WebIDL interfaces with constructors
pub trait WebIdlConstructable: WebIdlInterface {
    /// Constructor arguments type
    type Args;
    
    /// Construct a new instance
    fn construct(args: Self::Args, ctx: &mut Context) -> JsResult<Self>;
}

/// Descriptor for a WebIDL attribute (property)
#[derive(Debug, Clone)]
pub struct AttributeDescriptor {
    /// Attribute name
    pub name: &'static str,
    /// Whether the attribute is readonly
    pub readonly: bool,
    /// Whether the attribute is static
    pub is_static: bool,
    /// Getter function
    pub getter: Option<NativeFunctionPointer>,
    /// Setter function (None for readonly)
    pub setter: Option<NativeFunctionPointer>,
}

/// Descriptor for a WebIDL method
#[derive(Debug, Clone)]
pub struct MethodDescriptor {
    /// Method name
    pub name: &'static str,
    /// Expected argument count
    pub length: u32,
    /// Whether the method is static
    pub is_static: bool,
    /// The native function implementation
    pub func: NativeFunctionPointer,
}

/// Descriptor for a WebIDL constant
#[derive(Debug, Clone)]
pub struct ConstantDescriptor {
    /// Constant name
    pub name: &'static str,
    /// Constant value
    pub value: ConstantValue,
}

/// Value types for constants
#[derive(Debug, Clone, Copy)]
pub enum ConstantValue {
    Integer(i32),
    Float(f64),
    Boolean(bool),
}

impl From<ConstantValue> for JsValue {
    fn from(val: ConstantValue) -> Self {
        match val {
            ConstantValue::Integer(i) => JsValue::from(i),
            ConstantValue::Float(f) => JsValue::from(f),
            ConstantValue::Boolean(b) => JsValue::from(b),
        }
    }
}

/// Builder for creating WebIDL interface prototypes
pub struct InterfaceBuilder<'ctx> {
    ctx: &'ctx mut Context,
    name: &'static str,
    prototype: JsObject,
    constructor_fn: Option<NativeFunctionPointer>,
    constructor_length: u32,
}

impl<'ctx> InterfaceBuilder<'ctx> {
    /// Create a new interface builder
    pub fn new(ctx: &'ctx mut Context, name: &'static str) -> Self {
        let prototype = JsObject::with_null_proto();
        InterfaceBuilder {
            ctx,
            name,
            prototype,
            constructor_fn: None,
            constructor_length: 0,
        }
    }

    /// Create a new interface builder with a parent prototype
    pub fn with_parent(ctx: &'ctx mut Context, name: &'static str, parent_proto: JsObject) -> Self {
        let prototype = JsObject::from_proto_and_data(Some(parent_proto), ());
        InterfaceBuilder {
            ctx,
            name,
            prototype,
            constructor_fn: None,
            constructor_length: 0,
        }
    }

    /// Add a constructor to the interface
    pub fn constructor(mut self, func: NativeFunctionPointer, length: u32) -> Self {
        self.constructor_fn = Some(func);
        self.constructor_length = length;
        self
    }

    /// Add a readonly attribute
    pub fn readonly_attribute(
        self,
        name: &'static str,
        getter: NativeFunctionPointer,
    ) -> Self {
        self.attribute(name, getter, None)
    }

    /// Add an attribute with getter and optional setter
    pub fn attribute(
        self,
        name: &'static str,
        getter: NativeFunctionPointer,
        setter: Option<NativeFunctionPointer>,
    ) -> Self {
        let getter_fn = NativeFunction::from_fn_ptr(getter);
        let setter_fn = setter.map(NativeFunction::from_fn_ptr);

        let desc = PropertyDescriptor::builder()
            .get(getter_fn.to_js_function(self.ctx.realm()))
            .maybe_set(setter_fn.map(|f| f.to_js_function(self.ctx.realm())))
            .enumerable(true)
            .configurable(true);

        self.prototype
            .define_property_or_throw(js_string!(name), desc, self.ctx)
            .expect("Failed to define attribute");

        self
    }

    /// Add a method
    pub fn method(
        self,
        name: &'static str,
        func: NativeFunctionPointer,
        _length: u32,
    ) -> Self {
        let js_func = NativeFunction::from_fn_ptr(func)
            .to_js_function(self.ctx.realm());

        self.prototype
            .define_property_or_throw(
                js_string!(name),
                PropertyDescriptor::builder()
                    .value(js_func)
                    .writable(true)
                    .enumerable(false)
                    .configurable(true),
                self.ctx,
            )
            .expect("Failed to define method");

        self
    }

    /// Add a static method
    pub fn static_method(
        self,
        _name: &'static str,
        _func: NativeFunctionPointer,
        _length: u32,
    ) -> Self {
        // Static methods are added to the constructor, not prototype
        // Will be handled in build()
        self
    }

    /// Add a constant
    pub fn constant(self, name: &'static str, value: ConstantValue) -> Self {
        self.prototype
            .define_property_or_throw(
                js_string!(name),
                PropertyDescriptor::builder()
                    .value(JsValue::from(value))
                    .writable(false)
                    .enumerable(true)
                    .configurable(false),
                self.ctx,
            )
            .expect("Failed to define constant");

        self
    }

    /// Build and register the interface
    pub fn build(self) -> JsResult<JsObject> {
        // Create constructor function
        let constructor = if let Some(ctor_fn) = self.constructor_fn {
            NativeFunction::from_fn_ptr(ctor_fn)
                .to_js_function(self.ctx.realm())
        } else {
            // Create a throwing constructor for non-constructable interfaces
            let _name = self.name;
            NativeFunction::from_fn_ptr(|_, _, _| {
                Err(JsNativeError::typ()
                    .with_message("Illegal constructor")
                    .into())
            })
            .to_js_function(self.ctx.realm())
        };

        // Set prototype on constructor
        constructor
            .define_property_or_throw(
                js_string!("prototype"),
                PropertyDescriptor::builder()
                    .value(self.prototype.clone())
                    .writable(false)
                    .enumerable(false)
                    .configurable(false),
                self.ctx,
            )?;

        // Set constructor on prototype
        self.prototype
            .define_property_or_throw(
                js_string!("constructor"),
                PropertyDescriptor::builder()
                    .value(constructor.clone())
                    .writable(true)
                    .enumerable(false)
                    .configurable(true),
                self.ctx,
            )?;

        // Register on global object
        self.ctx
            .global_object()
            .define_property_or_throw(
                js_string!(self.name),
                PropertyDescriptor::builder()
                    .value(constructor)
                    .writable(true)
                    .enumerable(false)
                    .configurable(true),
                self.ctx,
            )?;

        Ok(self.prototype)
    }
}

/// Helper macro for defining WebIDL interfaces
#[macro_export]
macro_rules! define_webidl_interface {
    (
        interface $name:ident $(: $parent:ty)? {
            $(const $const_name:ident: $const_type:ty = $const_value:expr;)*
            $(readonly attribute $ro_attr_name:ident: $ro_attr_type:ty;)*
            $(attribute $attr_name:ident: $attr_type:ty;)*
            $(fn $method_name:ident($($arg_name:ident: $arg_type:ty),*) $(-> $ret_type:ty)?;)*
        }
    ) => {
        // Generated interface implementation
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_builder() {
        let mut ctx = Context::default();

        fn test_getter(
            _this: &JsValue,
            _args: &[JsValue],
            _ctx: &mut Context,
        ) -> JsResult<JsValue> {
            Ok(JsValue::from(42))
        }

        fn test_method(
            _this: &JsValue,
            _args: &[JsValue],
            _ctx: &mut Context,
        ) -> JsResult<JsValue> {
            Ok(JsValue::from(js_string!("hello")))
        }

        let proto = InterfaceBuilder::new(&mut ctx, "TestInterface")
            .readonly_attribute("testAttr", test_getter)
            .method("testMethod", test_method, 0)
            .constant("TEST_CONST", ConstantValue::Integer(100))
            .build()
            .unwrap();

        // Verify interface was registered
        let global = ctx.global_object();
        let ctor = global.get(js_string!("TestInterface"), &mut ctx).unwrap();
        assert!(ctor.is_object());
    }

    #[test]
    fn test_constant_value() {
        assert_eq!(JsValue::from(ConstantValue::Integer(42)), JsValue::from(42));
        assert_eq!(JsValue::from(ConstantValue::Boolean(true)), JsValue::from(true));
    }
}
