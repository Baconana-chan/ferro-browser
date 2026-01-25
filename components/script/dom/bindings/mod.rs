/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The code to expose the DOM to JavaScript through IDL bindings.
//!
//! Exposing a DOM object to JavaScript
//! ===================================
//!
//! As [explained earlier](../index.html#a-dom-object-and-its-reflector), the
//! implementation of an interface `Foo` involves two objects: the DOM object
//! (implemented in Rust) and the reflector (a `JSObject`).
//!
//! In order to expose the interface's members to the web, properties
//! corresponding to the operations and attributes are defined on an object in
//! the reflector's prototype chain or on the reflector itself.
//!
//! Typically, these properties are either value properties whose value is a
//! function (for operations) or accessor properties that have a getter and
//! optionally a setter function (for attributes, depending on whether they are
//! marked `readonly`).
//!
//! All these JavaScript functions are set up such that, when they're called,
//! they call a Rust function in the generated glue code. This glue code does
//! some sanity checks and [argument conversions](conversions/index.html), and
//! calls into API implementation for the DOM object.
//!
//! Rust reflections of WebIDL constructs
//! =====================================
//!
//! WebIDL members are turned into methods on the DOM object (static methods
//! for a static members and instance methods for regular members).
//!
//! The instance methods for an interface `Foo` are defined on a
//! `dom::bindings::codegen::Bindings::FooBinding::FooMethods` trait. This
//! trait is then implemented for `Foo`. (All methods take an `&self`
//! parameter, as pointers to DOM objects can be freely aliased.)
//!
//! The return type and argument types are determined
//! [as described below](#rust-reflections-of-webidl-types).
//! In addition to those, all methods that are
//! [allowed to throw](#throwing-exceptions)
//! will have the return value wrapped in
//! [`Fallible<T>`](error/type.Fallible.html).
//! Methods that use certain WebIDL types like `any` or `object` will get a
//! `*mut JSContext` argument prepended to the argument list. Static methods
//! will be passed a [`&GlobalScope`](../globalscope/struct.GlobalScope.html)
//! for the relevant global. This argument comes before the `*mut JSContext`
//! argument, if any.
//!
//! Rust reflections of WebIDL operations (methods)
//! -----------------------------------------------
//!
//! A WebIDL operation is turned into one method for every overload.
//! The first overload gets the base name, and consecutive overloads have an
//! underscore appended to the name.
//!
//! The base name of the Rust method is simply the name of the WebIDL operation
//! with the first letter converted to uppercase.
//!
//! Rust reflections of WebIDL attributes
//! -------------------------------------
//!
//! A WebIDL attribute is turned into a pair of methods: one for the getter and
//! one for the setter. A readonly attribute only has a getter and no setter.
//!
//! The getter's name is the name of the attribute with the first letter
//! converted to uppercase. It has `Get` prepended to it if the type of the
//! attribute is nullable or if the getter can throw.
//!
//! The method signature for the getter looks just like an operation with no
//! arguments and the attribute's type as the return type.
//!
//! The setter's name is `Set` followed by the name of the attribute with the
//! first letter converted to uppercase. The method signature looks just like
//! an operation with a void return value and a single argument whose type is
//! the attribute's type.
//!
//! Rust reflections of WebIDL constructors
//! ---------------------------------------
//!
//! A WebIDL constructor is turned into a static class method named
//! `Constructor`. The arguments of this method will be the arguments of the
//! WebIDL constructor, with a `&GlobalScope` for the relevant global prepended.
//! The return value of the constructor for MyInterface is exactly the same as
//! that of a method returning an instance of MyInterface. Constructors are
//! always [allowed to throw](#throwing-exceptions).
//!
//! Rust reflections of WebIDL types
//! --------------------------------
//!
//! The exact Rust representation for WebIDL types can depend on the precise
//! way that they're being used (e.g., return values and arguments might have
//! different representations).
//!
//! Optional arguments which do not have a default value are represented by
//! wrapping `Option<T>` around the representation of the argument type.
//! Optional arguments which do have a default value are represented by the
//! argument type itself, set to the default value if the argument was not in
//! fact passed in.
//!
//! Variadic WebIDL arguments are represented by wrapping a `Vec<T>` around the
//! representation of the argument type.
//!
//! See [the type mapping for particular types](conversions/index.html).
//!
//! Rust reflections of stringifiers
//! --------------------------------
//!
//! *To be written.*
//!
//! Rust reflections of legacy callers
//! ---------------------------------
//!
//! Legacy callers are not yet implemented.
//!
//! Throwing exceptions
//! ===================
//!
//! WebIDL methods, getters, and setters that need to throw exceptions need to
//! be explicitly marked as such with the `[Throws]`, `[GetterThrows]` and
//! `[SetterThrows]` custom attributes.
//!
//! `[Throws]` applies to both methods and attributes; for attributes it means
//! both the getter and the setter (if any) can throw. `[GetterThrows]` applies
//! only to attributes. `[SetterThrows]` applies only to writable attributes.
//!
//! The corresponding Rust methods will have the return value wrapped in
//! [`Fallible<T>`](error/type.Fallible.html). To throw an exception, simply
//! return `Err()` from the method with the appropriate [error value]
//! (error/enum.Error.html).

#![expect(unsafe_code)]
#![deny(missing_docs)]
#![deny(non_snake_case)]

pub(crate) mod buffer_source;
#[expect(dead_code)]
pub(crate) mod cell;
pub(crate) mod constructor;
pub(crate) mod conversions;
pub(crate) mod domname;
pub(crate) mod error;
pub(crate) mod frozenarray;
pub(crate) mod function;
pub(crate) mod import;
pub(crate) mod inheritance;
pub(crate) mod like;
pub(crate) mod principals;
pub(crate) mod proxyhandler;
pub(crate) mod refcounted;
pub(crate) mod reflector;
pub(crate) mod root;
pub(crate) mod serializable;
pub(crate) mod settings_stack;
pub(crate) mod str;
pub(crate) mod structuredclone;
pub(crate) mod trace;
pub(crate) mod transferable;
pub(crate) mod utils;
pub(crate) mod weakref;
pub(crate) mod xmlname;

#[cfg(feature = "js-spidermonkey")]
pub(crate) use script_bindings::{callback, iterable, num};
#[cfg(feature = "js-boa")]
pub(crate) use crate::script_bindings::{callback, iterable, num};

/// Generated JS-Rust bindings.
/// Only available when using SpiderMonkey (js-spidermonkey feature)
#[cfg(feature = "js-spidermonkey")]
#[allow(missing_docs, non_snake_case)]
pub(crate) mod codegen {
    pub(crate) mod DomTypeHolder {
        include!(concat!(env!("OUT_DIR"), "/DomTypeHolder.rs"));
    }
    pub(crate) use script_bindings::codegen::GenericBindings;
    #[expect(dead_code)]
    pub(crate) mod Bindings {
        include!(concat!(env!("OUT_DIR"), "/ConcreteBindings/mod.rs"));
    }
    pub(crate) mod InterfaceObjectMap {
        include!(concat!(env!("OUT_DIR"), "/InterfaceObjectMap.rs"));
    }
    pub(crate) mod ConcreteInheritTypes {
        include!(concat!(env!("OUT_DIR"), "/ConcreteInheritTypes.rs"));
    }
    pub(crate) use script_bindings::codegen::{PrototypeList, RegisterBindings};
    #[expect(dead_code)]
    pub(crate) mod UnionTypes {
        include!(concat!(env!("OUT_DIR"), "/UnionTypes.rs"));
    }
}

/// Stub codegen module for Boa engine
/// When using Boa, we don't generate SpiderMonkey bindings
#[cfg(feature = "js-boa")]
#[allow(missing_docs, non_snake_case)]
pub(crate) mod codegen {
    pub(crate) mod DomTypeHolder {
        //! Stub DomTypeHolder for Boa
        use ::boa_bindings::reflector::DomTypes;
        
        /// The type holder for DOM types
        pub struct DomTypeHolder;
        
        impl DomTypes for DomTypeHolder {
            // Stub implementation - real types defined in boa_bindings
        }
    }
    
    pub(crate) use crate::script_bindings::codegen::GenericBindings;
    
    /// Stub bindings module - contains empty binding modules for Boa
    /// When using Boa, actual bindings are handled differently
    #[allow(non_snake_case)]
    pub(crate) mod Bindings {
        // Generate stub binding modules for all DOM interfaces
        // These are empty stubs - actual implementation is in boa_bindings::dom
        macro_rules! stub_binding {
            ($name:ident) => {
                pub mod $name {
                    #![allow(dead_code, non_upper_case_globals)]
                }
            };
        }
        
        // Core DOM bindings
        stub_binding!(EventBinding);
        stub_binding!(EventTargetBinding);
        stub_binding!(NodeBinding);
        stub_binding!(ElementBinding);
        stub_binding!(DocumentBinding);
        stub_binding!(WindowBinding);
        stub_binding!(HTMLElementBinding);
        
        // Events
        stub_binding!(AnimationEventBinding);
        stub_binding!(TransitionEventBinding);
        stub_binding!(EventListenerBinding);
        stub_binding!(CustomEventBinding);
        stub_binding!(MouseEventBinding);
        stub_binding!(KeyboardEventBinding);
        stub_binding!(UIEventBinding);
        stub_binding!(FocusEventBinding);
        stub_binding!(InputEventBinding);
        stub_binding!(WheelEventBinding);
        stub_binding!(PointerEventBinding);
        stub_binding!(TouchEventBinding);
        stub_binding!(ClipboardEventBinding);
        stub_binding!(DragEventBinding);
        stub_binding!(ProgressEventBinding);
        stub_binding!(ErrorEventBinding);
        stub_binding!(MessageEventBinding);
        stub_binding!(PageTransitionEventBinding);
        stub_binding!(HashChangeEventBinding);
        stub_binding!(PopStateEventBinding);
        stub_binding!(StorageEventBinding);
        stub_binding!(BeforeUnloadEventBinding);
        stub_binding!(SecurityPolicyViolationEventBinding);
        stub_binding!(CompositionEventBinding);
        stub_binding!(TextEventBinding);
        
        // AbortController
        stub_binding!(AbortControllerBinding);
        stub_binding!(AbortSignalBinding);
        
        // Ranges
        stub_binding!(AbstractRangeBinding);
        stub_binding!(RangeBinding);
        stub_binding!(StaticRangeBinding);
        
        // Collections
        stub_binding!(NodeListBinding);
        stub_binding!(HTMLCollectionBinding);
        stub_binding!(DOMTokenListBinding);
        stub_binding!(NamedNodeMapBinding);
        
        // DOM traversal
        stub_binding!(TreeWalkerBinding);
        stub_binding!(NodeIteratorBinding);
        stub_binding!(NodeFilterBinding);
        
        // Geometry
        stub_binding!(DOMRectBinding);
        stub_binding!(DOMRectReadOnlyBinding);
        stub_binding!(DOMRectListBinding);
        stub_binding!(DOMPointBinding);
        stub_binding!(DOMPointReadOnlyBinding);
        stub_binding!(DOMQuadBinding);
        stub_binding!(DOMMatrixBinding);
        stub_binding!(DOMMatrixReadOnlyBinding);
        
        // CSS
        stub_binding!(CSSStyleDeclarationBinding);
        stub_binding!(CSSRuleBinding);
        stub_binding!(CSSRuleListBinding);
        stub_binding!(CSSStyleRuleBinding);
        stub_binding!(CSSStyleSheetBinding);
        stub_binding!(StyleSheetBinding);
        stub_binding!(StyleSheetListBinding);
        stub_binding!(MediaListBinding);
        stub_binding!(MediaQueryListBinding);
        
        // HTML Elements
        stub_binding!(HTMLAnchorElementBinding);
        stub_binding!(HTMLAreaElementBinding);
        stub_binding!(HTMLAudioElementBinding);
        stub_binding!(HTMLBaseElementBinding);
        stub_binding!(HTMLBodyElementBinding);
        stub_binding!(HTMLBRElementBinding);
        stub_binding!(HTMLButtonElementBinding);
        stub_binding!(HTMLCanvasElementBinding);
        stub_binding!(HTMLDataElementBinding);
        stub_binding!(HTMLDataListElementBinding);
        stub_binding!(HTMLDetailsElementBinding);
        stub_binding!(HTMLDialogElementBinding);
        stub_binding!(HTMLDirectoryElementBinding);
        stub_binding!(HTMLDivElementBinding);
        stub_binding!(HTMLDListElementBinding);
        stub_binding!(HTMLEmbedElementBinding);
        stub_binding!(HTMLFieldSetElementBinding);
        stub_binding!(HTMLFontElementBinding);
        stub_binding!(HTMLFormElementBinding);
        stub_binding!(HTMLFrameElementBinding);
        stub_binding!(HTMLFrameSetElementBinding);
        stub_binding!(HTMLHeadElementBinding);
        stub_binding!(HTMLHeadingElementBinding);
        stub_binding!(HTMLHRElementBinding);
        stub_binding!(HTMLHtmlElementBinding);
        stub_binding!(HTMLIFrameElementBinding);
        stub_binding!(HTMLImageElementBinding);
        stub_binding!(HTMLInputElementBinding);
        stub_binding!(HTMLLabelElementBinding);
        stub_binding!(HTMLLegendElementBinding);
        stub_binding!(HTMLLIElementBinding);
        stub_binding!(HTMLLinkElementBinding);
        stub_binding!(HTMLMapElementBinding);
        stub_binding!(HTMLMediaElementBinding);
        stub_binding!(HTMLMenuElementBinding);
        stub_binding!(HTMLMetaElementBinding);
        stub_binding!(HTMLMeterElementBinding);
        stub_binding!(HTMLModElementBinding);
        stub_binding!(HTMLObjectElementBinding);
        stub_binding!(HTMLOListElementBinding);
        stub_binding!(HTMLOptGroupElementBinding);
        stub_binding!(HTMLOptionElementBinding);
        stub_binding!(HTMLOutputElementBinding);
        stub_binding!(HTMLParagraphElementBinding);
        stub_binding!(HTMLParamElementBinding);
        stub_binding!(HTMLPictureElementBinding);
        stub_binding!(HTMLPreElementBinding);
        stub_binding!(HTMLProgressElementBinding);
        stub_binding!(HTMLQuoteElementBinding);
        stub_binding!(HTMLScriptElementBinding);
        stub_binding!(HTMLSelectElementBinding);
        stub_binding!(HTMLSlotElementBinding);
        stub_binding!(HTMLSourceElementBinding);
        stub_binding!(HTMLSpanElementBinding);
        stub_binding!(HTMLStyleElementBinding);
        stub_binding!(HTMLTableCaptionElementBinding);
        stub_binding!(HTMLTableCellElementBinding);
        stub_binding!(HTMLTableColElementBinding);
        stub_binding!(HTMLTableElementBinding);
        stub_binding!(HTMLTableRowElementBinding);
        stub_binding!(HTMLTableSectionElementBinding);
        stub_binding!(HTMLTemplateElementBinding);
        stub_binding!(HTMLTextAreaElementBinding);
        stub_binding!(HTMLTimeElementBinding);
        stub_binding!(HTMLTitleElementBinding);
        stub_binding!(HTMLTrackElementBinding);
        stub_binding!(HTMLUListElementBinding);
        stub_binding!(HTMLUnknownElementBinding);
        stub_binding!(HTMLVideoElementBinding);
        
        // Form elements
        stub_binding!(FormDataBinding);
        stub_binding!(HTMLFormControlsCollectionBinding);
        stub_binding!(RadioNodeListBinding);
        stub_binding!(ValidityStateBinding);
        
        // XHR/Fetch
        stub_binding!(XMLHttpRequestBinding);
        stub_binding!(XMLHttpRequestEventTargetBinding);
        stub_binding!(XMLHttpRequestUploadBinding);
        stub_binding!(RequestBinding);
        stub_binding!(ResponseBinding);
        stub_binding!(HeadersBinding);
        stub_binding!(BodyBinding);
        
        // Blob/File
        stub_binding!(BlobBinding);
        stub_binding!(FileBinding);
        stub_binding!(FileListBinding);
        stub_binding!(FileReaderBinding);
        
        // URL
        stub_binding!(URLBinding);
        stub_binding!(URLSearchParamsBinding);
        
        // Promises
        stub_binding!(PromiseBinding);
        
        // Text
        stub_binding!(TextBinding);
        stub_binding!(CharacterDataBinding);
        stub_binding!(CommentBinding);
        stub_binding!(CDATASectionBinding);
        stub_binding!(ProcessingInstructionBinding);
        stub_binding!(DocumentTypeBinding);
        stub_binding!(DocumentFragmentBinding);
        
        // Selection
        stub_binding!(SelectionBinding);
        
        // Shadow DOM
        stub_binding!(ShadowRootBinding);
        
        // History
        stub_binding!(HistoryBinding);
        stub_binding!(LocationBinding);
        
        // Navigator
        stub_binding!(NavigatorBinding);
        stub_binding!(ScreenBinding);
        
        // Performance
        stub_binding!(PerformanceBinding);
        stub_binding!(PerformanceEntryBinding);
        stub_binding!(PerformanceMarkBinding);
        stub_binding!(PerformanceMeasureBinding);
        stub_binding!(PerformanceNavigationBinding);
        stub_binding!(PerformanceTimingBinding);
        
        // Console
        stub_binding!(ConsoleBinding);
        
        // Crypto
        stub_binding!(CryptoBinding);
        stub_binding!(SubtleCryptoBinding);
        stub_binding!(CryptoKeyBinding);
        
        // Storage
        stub_binding!(StorageBinding);
        
        // WebSocket
        stub_binding!(WebSocketBinding);
        stub_binding!(CloseEventBinding);
        
        // Workers
        stub_binding!(WorkerBinding);
        stub_binding!(DedicatedWorkerGlobalScopeBinding);
        stub_binding!(SharedWorkerBinding);
        stub_binding!(SharedWorkerGlobalScopeBinding);
        stub_binding!(ServiceWorkerBinding);
        stub_binding!(ServiceWorkerContainerBinding);
        stub_binding!(ServiceWorkerRegistrationBinding);
        stub_binding!(ServiceWorkerGlobalScopeBinding);
        
        // Canvas
        stub_binding!(CanvasRenderingContext2DBinding);
        stub_binding!(WebGLRenderingContextBinding);
        stub_binding!(WebGL2RenderingContextBinding);
        stub_binding!(ImageDataBinding);
        stub_binding!(Path2DBinding);
        stub_binding!(CanvasGradientBinding);
        stub_binding!(CanvasPatternBinding);
        stub_binding!(TextMetricsBinding);
        stub_binding!(ImageBitmapBinding);
        stub_binding!(OffscreenCanvasBinding);
        
        // Media
        stub_binding!(AudioContextBinding);
        stub_binding!(AudioNodeBinding);
        stub_binding!(MediaStreamBinding);
        stub_binding!(MediaRecorderBinding);
        stub_binding!(MediaSourceBinding);
        stub_binding!(SourceBufferBinding);
        stub_binding!(SourceBufferListBinding);
        
        // Mutation Observer
        stub_binding!(MutationObserverBinding);
        stub_binding!(MutationRecordBinding);
        
        // Intersection/Resize Observer
        stub_binding!(IntersectionObserverBinding);
        stub_binding!(IntersectionObserverEntryBinding);
        stub_binding!(ResizeObserverBinding);
        stub_binding!(ResizeObserverEntryBinding);
        
        // Custom Elements
        stub_binding!(CustomElementRegistryBinding);
        
        // Misc
        stub_binding!(DOMExceptionBinding);
        stub_binding!(DOMParserBinding);
        stub_binding!(XMLSerializerBinding);
        stub_binding!(XPathEvaluatorBinding);
        stub_binding!(XPathResultBinding);
        stub_binding!(AttrBinding);
    }
    
    pub(crate) mod InterfaceObjectMap {
        //! Interface object map stubs
        use std::collections::HashMap;
        
        /// Get the interface object map
        pub fn get() -> HashMap<&'static str, u16> {
            HashMap::new()
        }
    }
    
    pub(crate) mod ConcreteInheritTypes {
        //! Concrete inheritance types for Boa
    }
    
    pub(crate) use crate::script_bindings::codegen::{PrototypeList, RegisterBindings};
    
    // Re-export UnionTypes from script_bindings shim
    pub(crate) use crate::script_bindings::codegen::UnionTypes;}