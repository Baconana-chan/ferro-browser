//! RegisterBindings - Interface registration for Boa bindings
//!
//! This module provides the infrastructure for registering WebIDL interfaces
//! with the Boa JavaScript engine. It's the equivalent of SpiderMonkey's
//! generated RegisterBindings.rs.
//!
//! Unlike SpiderMonkey's Python-generated code, this provides a runtime
//! registration system that allows interfaces to be registered dynamically.

use std::collections::HashMap;
use std::sync::RwLock;

use boa_engine::{Context, JsObject, JsResult};

use super::prototype_list::{ID, Constructor, InterfaceChain};

/// Type alias for interface registration functions
pub type RegisterFn = fn(&mut Context) -> JsResult<()>;

/// Type alias for prototype creation functions
pub type CreatePrototypeFn = fn(&mut Context) -> JsResult<JsObject>;

/// Type alias for constructor creation functions  
pub type CreateConstructorFn = fn(&mut Context, JsObject) -> JsResult<JsObject>;

/// Descriptor for a registered interface
#[derive(Clone)]
pub struct InterfaceDescriptor {
    /// The name of the interface (e.g., "Node", "Element")
    pub name: &'static str,
    
    /// The prototype ID for this interface
    pub proto_id: ID,
    
    /// The constructor ID, if this interface has a constructor
    pub constructor_id: Option<Constructor>,
    
    /// The prototype chain for inheritance
    pub interface_chain: InterfaceChain,
    
    /// Function to register the interface on global
    pub register_fn: Option<RegisterFn>,
    
    /// Function to create the prototype object
    pub create_prototype_fn: Option<CreatePrototypeFn>,
    
    /// Function to create the constructor
    pub create_constructor_fn: Option<CreateConstructorFn>,
    
    /// Whether this interface is exposed to Window
    pub exposed_to_window: bool,
    
    /// Whether this interface is exposed to Worker
    pub exposed_to_worker: bool,
    
    /// Whether this is a namespace (like `console`)
    pub is_namespace: bool,
    
    /// Whether this is a callback interface
    pub is_callback: bool,
}

impl InterfaceDescriptor {
    /// Create a new interface descriptor with default values
    pub fn new(name: &'static str, proto_id: ID) -> Self {
        InterfaceDescriptor {
            name,
            proto_id,
            constructor_id: None,
            interface_chain: InterfaceChain::default(),
            register_fn: None,
            create_prototype_fn: None,
            create_constructor_fn: None,
            exposed_to_window: true,
            exposed_to_worker: false,
            is_namespace: false,
            is_callback: false,
        }
    }
    
    /// Set the constructor ID
    pub fn with_constructor(mut self, ctor_id: Constructor) -> Self {
        self.constructor_id = Some(ctor_id);
        self
    }
    
    /// Set the interface chain
    pub fn with_chain(mut self, chain: InterfaceChain) -> Self {
        self.interface_chain = chain;
        self
    }
    
    /// Set the registration function
    pub fn with_register_fn(mut self, f: RegisterFn) -> Self {
        self.register_fn = Some(f);
        self
    }
    
    /// Set the prototype creation function
    pub fn with_prototype_fn(mut self, f: CreatePrototypeFn) -> Self {
        self.create_prototype_fn = Some(f);
        self
    }
    
    /// Set the constructor creation function
    pub fn with_constructor_fn(mut self, f: CreateConstructorFn) -> Self {
        self.create_constructor_fn = Some(f);
        self
    }
    
    /// Mark as exposed to Worker
    pub fn exposed_to_worker(mut self) -> Self {
        self.exposed_to_worker = true;
        self
    }
    
    /// Mark as not exposed to Window
    pub fn not_exposed_to_window(mut self) -> Self {
        self.exposed_to_window = false;
        self
    }
    
    /// Mark as a namespace
    pub fn as_namespace(mut self) -> Self {
        self.is_namespace = true;
        self
    }
    
    /// Mark as a callback interface
    pub fn as_callback(mut self) -> Self {
        self.is_callback = true;
        self
    }
}

/// Global registry of interface descriptors
struct InterfaceRegistry {
    /// Map from interface name to descriptor
    by_name: HashMap<&'static str, InterfaceDescriptor>,
    
    /// Map from proto ID to descriptor
    by_proto_id: HashMap<u16, InterfaceDescriptor>,
    
    /// Map from constructor ID to descriptor
    by_constructor_id: HashMap<u16, InterfaceDescriptor>,
    
    /// Registration order for deterministic initialization
    registration_order: Vec<&'static str>,
}

impl InterfaceRegistry {
    fn new() -> Self {
        InterfaceRegistry {
            by_name: HashMap::new(),
            by_proto_id: HashMap::new(),
            by_constructor_id: HashMap::new(),
            registration_order: Vec::new(),
        }
    }
}

static REGISTRY: RwLock<Option<InterfaceRegistry>> = RwLock::new(None);

/// Initialize the global interface registry
fn ensure_registry() {
    let mut guard = REGISTRY.write().unwrap();
    if guard.is_none() {
        *guard = Some(InterfaceRegistry::new());
    }
}

/// Register an interface descriptor
pub fn register_interface(desc: InterfaceDescriptor) {
    ensure_registry();
    
    let mut guard = REGISTRY.write().unwrap();
    let registry = guard.as_mut().unwrap();
    
    registry.registration_order.push(desc.name);
    registry.by_proto_id.insert(desc.proto_id.0, desc.clone());
    
    if let Some(ctor_id) = desc.constructor_id {
        registry.by_constructor_id.insert(ctor_id.0, desc.clone());
    }
    
    registry.by_name.insert(desc.name, desc);
}

/// Get an interface descriptor by name
pub fn get_interface_by_name(name: &str) -> Option<InterfaceDescriptor> {
    let guard = REGISTRY.read().unwrap();
    guard.as_ref()?.by_name.get(name).cloned()
}

/// Get an interface descriptor by prototype ID
pub fn get_interface_by_proto_id(proto_id: ID) -> Option<InterfaceDescriptor> {
    let guard = REGISTRY.read().unwrap();
    guard.as_ref()?.by_proto_id.get(&proto_id.0).cloned()
}

/// Get an interface descriptor by constructor ID
pub fn get_interface_by_constructor_id(ctor_id: Constructor) -> Option<InterfaceDescriptor> {
    let guard = REGISTRY.read().unwrap();
    guard.as_ref()?.by_constructor_id.get(&ctor_id.0).cloned()
}

/// Get all registered interface names in registration order
pub fn get_all_interfaces() -> Vec<&'static str> {
    let guard = REGISTRY.read().unwrap();
    guard.as_ref()
        .map(|r| r.registration_order.clone())
        .unwrap_or_default()
}

/// Register all interfaces on a JavaScript context.
/// 
/// This is called during realm initialization to set up all
/// the WebIDL interfaces on the global object.
pub fn register_all_bindings(ctx: &mut Context) -> JsResult<()> {
    let interfaces: Vec<InterfaceDescriptor> = {
        let guard = REGISTRY.read().unwrap();
        guard.as_ref()
            .map(|r| {
                r.registration_order.iter()
                    .filter_map(|name| r.by_name.get(name).cloned())
                    .collect()
            })
            .unwrap_or_default()
    };
    
    for desc in interfaces {
        if let Some(register_fn) = desc.register_fn {
            register_fn(ctx)?;
        }
    }
    
    Ok(())
}

/// Register bindings for Window global
pub fn register_window_bindings(ctx: &mut Context) -> JsResult<()> {
    let interfaces: Vec<InterfaceDescriptor> = {
        let guard = REGISTRY.read().unwrap();
        guard.as_ref()
            .map(|r| {
                r.registration_order.iter()
                    .filter_map(|name| r.by_name.get(name).cloned())
                    .filter(|d| d.exposed_to_window)
                    .collect()
            })
            .unwrap_or_default()
    };
    
    for desc in interfaces {
        if let Some(register_fn) = desc.register_fn {
            register_fn(ctx)?;
        }
    }
    
    Ok(())
}

/// Register bindings for Worker global
pub fn register_worker_bindings(ctx: &mut Context) -> JsResult<()> {
    let interfaces: Vec<InterfaceDescriptor> = {
        let guard = REGISTRY.read().unwrap();
        guard.as_ref()
            .map(|r| {
                r.registration_order.iter()
                    .filter_map(|name| r.by_name.get(name).cloned())
                    .filter(|d| d.exposed_to_worker)
                    .collect()
            })
            .unwrap_or_default()
    };
    
    for desc in interfaces {
        if let Some(register_fn) = desc.register_fn {
            register_fn(ctx)?;
        }
    }
    
    Ok(())
}

/// Cache for prototype and constructor objects
/// This is similar to SpiderMonkey's ProtoAndIfaceArray
pub struct ProtoAndIfaceCache {
    /// Prototype objects indexed by proto ID
    prototypes: HashMap<u16, JsObject>,
    
    /// Constructor objects indexed by constructor ID
    constructors: HashMap<u16, JsObject>,
}

impl ProtoAndIfaceCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        ProtoAndIfaceCache {
            prototypes: HashMap::new(),
            constructors: HashMap::new(),
        }
    }
    
    /// Get a prototype by ID
    pub fn get_prototype(&self, id: ID) -> Option<&JsObject> {
        self.prototypes.get(&id.0)
    }
    
    /// Set a prototype
    pub fn set_prototype(&mut self, id: ID, proto: JsObject) {
        self.prototypes.insert(id.0, proto);
    }
    
    /// Get a constructor by ID
    pub fn get_constructor(&self, id: Constructor) -> Option<&JsObject> {
        self.constructors.get(&id.0)
    }
    
    /// Set a constructor
    pub fn set_constructor(&mut self, id: Constructor, ctor: JsObject) {
        self.constructors.insert(id.0, ctor);
    }
    
    /// Check if a prototype is cached
    pub fn has_prototype(&self, id: ID) -> bool {
        self.prototypes.contains_key(&id.0)
    }
    
    /// Check if a constructor is cached
    pub fn has_constructor(&self, id: Constructor) -> bool {
        self.constructors.contains_key(&id.0)
    }
    
    /// Clear all cached objects
    pub fn clear(&mut self) {
        self.prototypes.clear();
        self.constructors.clear();
    }
}

impl Default for ProtoAndIfaceCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be registered as WebIDL bindings.
/// 
/// This is implemented by DOM types to provide their binding information.
pub trait RegisterBinding {
    /// The prototype ID for this interface
    const PROTO_ID: ID;
    
    /// The constructor ID, if any
    const CONSTRUCTOR_ID: Option<Constructor> = None;
    
    /// The interface name
    const NAME: &'static str;
    
    /// Register this interface on a context
    fn register(ctx: &mut Context) -> JsResult<()>;
    
    /// Create the prototype object
    fn create_prototype(ctx: &mut Context) -> JsResult<JsObject>;
    
    /// Get the interface descriptor
    fn descriptor() -> InterfaceDescriptor {
        InterfaceDescriptor::new(Self::NAME, Self::PROTO_ID)
            .with_register_fn(Self::register)
            .with_prototype_fn(Self::create_prototype)
    }
}

/// Macro to help define interface bindings
#[macro_export]
macro_rules! register_binding {
    (
        $interface:ty,
        proto_id: $proto_id:expr,
        name: $name:expr
        $(, constructor_id: $ctor_id:expr)?
    ) => {
        impl $crate::codegen::register_bindings::RegisterBinding for $interface {
            const PROTO_ID: $crate::codegen::prototype_list::ID = $proto_id;
            $(const CONSTRUCTOR_ID: Option<$crate::codegen::prototype_list::Constructor> = Some($ctor_id);)?
            const NAME: &'static str = $name;
            
            fn register(ctx: &mut ::boa_engine::Context) -> ::boa_engine::JsResult<()> {
                // Default implementation - override as needed
                Ok(())
            }
            
            fn create_prototype(ctx: &mut ::boa_engine::Context) -> ::boa_engine::JsResult<::boa_engine::JsObject> {
                Ok(::boa_engine::JsObject::with_null_proto())
            }
        }
    };
}

/// Initialize the binding registration system.
/// Call this once at startup before registering any bindings.
pub fn init() {
    ensure_registry();
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::prototype_list::well_known;
    
    fn dummy_register(_ctx: &mut Context) -> JsResult<()> {
        Ok(())
    }
    
    fn dummy_create_proto(_ctx: &mut Context) -> JsResult<JsObject> {
        Ok(JsObject::with_null_proto())
    }
    
    #[test]
    fn test_interface_descriptor() {
        let desc = InterfaceDescriptor::new("TestInterface", well_known::NODE)
            .with_constructor(Constructor::new(100))
            .with_register_fn(dummy_register)
            .with_prototype_fn(dummy_create_proto)
            .exposed_to_worker();
        
        assert_eq!(desc.name, "TestInterface");
        assert_eq!(desc.proto_id, well_known::NODE);
        assert_eq!(desc.constructor_id, Some(Constructor::new(100)));
        assert!(desc.exposed_to_window);
        assert!(desc.exposed_to_worker);
    }
    
    #[test]
    fn test_proto_and_iface_cache() {
        let mut cache = ProtoAndIfaceCache::new();
        let proto = JsObject::with_null_proto();
        
        assert!(!cache.has_prototype(well_known::NODE));
        
        cache.set_prototype(well_known::NODE, proto.clone());
        
        assert!(cache.has_prototype(well_known::NODE));
        assert!(cache.get_prototype(well_known::NODE).is_some());
    }
    
    #[test]
    fn test_registration() {
        init();
        
        let desc = InterfaceDescriptor::new("TestInterface2", ID::new(200))
            .with_register_fn(dummy_register);
        
        register_interface(desc);
        
        let retrieved = get_interface_by_name("TestInterface2");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "TestInterface2");
    }
}
