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
        use script_bindings::DomTypes;
        use script_bindings::DomObjectPlaceholder;
        
        // Placeholder types for Boa - these will be replaced with real DOM types when implemented
        macro_rules! define_placeholder {
            ($name:ident) => {
                pub struct $name;
                impl DomObjectPlaceholder for $name {}
            };
        }
        
        define_placeholder!(GlobalScopePlaceholder);
        define_placeholder!(WindowPlaceholder);
        define_placeholder!(WindowProxyPlaceholder);
        define_placeholder!(DocumentPlaceholder);
        define_placeholder!(DocumentFragmentPlaceholder);
        define_placeholder!(DocumentTypePlaceholder);
        define_placeholder!(ElementPlaceholder);
        define_placeholder!(NodePlaceholder);
        define_placeholder!(CommentPlaceholder);
        define_placeholder!(TextPlaceholder);
        define_placeholder!(CharacterDataPlaceholder);
        define_placeholder!(AttrPlaceholder);
        define_placeholder!(EventPlaceholder);
        define_placeholder!(EventTargetPlaceholder);
        define_placeholder!(ShadowRootPlaceholder);
        define_placeholder!(NodeListPlaceholder);
        define_placeholder!(HTMLCollectionPlaceholder);
        define_placeholder!(HTMLOptionsCollectionPlaceholder);
        define_placeholder!(HTMLFormControlsCollectionPlaceholder);
        define_placeholder!(DOMTokenListPlaceholder);
        define_placeholder!(NamedNodeMapPlaceholder);
        define_placeholder!(FileListPlaceholder);
        define_placeholder!(FilePlaceholder);
        define_placeholder!(BlobPlaceholder);
        define_placeholder!(CanvasRenderingContext2DPlaceholder);
        define_placeholder!(WebGLRenderingContextPlaceholder);
        define_placeholder!(WebGL2RenderingContextPlaceholder);
        define_placeholder!(ImageDataPlaceholder);
        define_placeholder!(URLPlaceholder);
        define_placeholder!(LocationPlaceholder);
        define_placeholder!(HistoryPlaceholder);
        define_placeholder!(NavigatorPlaceholder);
        define_placeholder!(StoragePlaceholder);
        define_placeholder!(XMLHttpRequestPlaceholder);
        define_placeholder!(RequestPlaceholder);
        define_placeholder!(ResponsePlaceholder);
        define_placeholder!(HeadersPlaceholder);
        define_placeholder!(KeyboardEventPlaceholder);
        define_placeholder!(MouseEventPlaceholder);
        define_placeholder!(UIEventPlaceholder);
        define_placeholder!(FocusEventPlaceholder);
        define_placeholder!(WheelEventPlaceholder);
        define_placeholder!(InputEventPlaceholder);
        define_placeholder!(TouchEventPlaceholder);
        define_placeholder!(PointerEventPlaceholder);
        define_placeholder!(CompositionEventPlaceholder);
        define_placeholder!(ClipboardEventPlaceholder);
        define_placeholder!(DragEventPlaceholder);
        define_placeholder!(AnimationEventPlaceholder);
        define_placeholder!(TransitionEventPlaceholder);
        define_placeholder!(MessageEventPlaceholder);
        define_placeholder!(ErrorEventPlaceholder);
        define_placeholder!(ProgressEventPlaceholder);
        define_placeholder!(CustomEventPlaceholder);
        define_placeholder!(HTMLElementPlaceholder);
        define_placeholder!(HTMLFormElementPlaceholder);
        define_placeholder!(HTMLInputElementPlaceholder);
        define_placeholder!(HTMLButtonElementPlaceholder);
        define_placeholder!(HTMLSelectElementPlaceholder);
        define_placeholder!(HTMLTextAreaElementPlaceholder);
        define_placeholder!(HTMLAnchorElementPlaceholder);
        define_placeholder!(HTMLImageElementPlaceholder);
        define_placeholder!(HTMLScriptElementPlaceholder);
        define_placeholder!(HTMLStyleElementPlaceholder);
        define_placeholder!(HTMLLinkElementPlaceholder);
        define_placeholder!(HTMLDivElementPlaceholder);
        define_placeholder!(HTMLSpanElementPlaceholder);
        define_placeholder!(HTMLParagraphElementPlaceholder);
        define_placeholder!(HTMLHeadingElementPlaceholder);
        define_placeholder!(HTMLBodyElementPlaceholder);
        define_placeholder!(HTMLHtmlElementPlaceholder);
        define_placeholder!(HTMLHeadElementPlaceholder);
        define_placeholder!(HTMLTableElementPlaceholder);
        define_placeholder!(HTMLTableRowElementPlaceholder);
        define_placeholder!(HTMLTableCellElementPlaceholder);
        define_placeholder!(HTMLCanvasElementPlaceholder);
        define_placeholder!(HTMLVideoElementPlaceholder);
        define_placeholder!(HTMLAudioElementPlaceholder);
        define_placeholder!(HTMLMediaElementPlaceholder);
        define_placeholder!(HTMLIFrameElementPlaceholder);
        define_placeholder!(HTMLTemplateElementPlaceholder);
        define_placeholder!(HTMLSlotElementPlaceholder);
        define_placeholder!(CSSStyleDeclarationPlaceholder);
        define_placeholder!(StyleSheetPlaceholder);
        define_placeholder!(CSSStyleSheetPlaceholder);
        define_placeholder!(CSSRulePlaceholder);
        define_placeholder!(CSSStyleRulePlaceholder);
        define_placeholder!(RangePlaceholder);
        define_placeholder!(SelectionPlaceholder);
        define_placeholder!(WorkerPlaceholder);
        define_placeholder!(WorkerGlobalScopePlaceholder);
        define_placeholder!(DedicatedWorkerGlobalScopePlaceholder);
        define_placeholder!(ServiceWorkerGlobalScopePlaceholder);
        define_placeholder!(DOMParserPlaceholder);
        define_placeholder!(XMLSerializerPlaceholder);
        define_placeholder!(TreeWalkerPlaceholder);
        define_placeholder!(NodeIteratorPlaceholder);
        define_placeholder!(MutationObserverPlaceholder);
        define_placeholder!(IntersectionObserverPlaceholder);
        define_placeholder!(ResizeObserverPlaceholder);
        define_placeholder!(PerformancePlaceholder);
        define_placeholder!(PerformanceEntryPlaceholder);
        define_placeholder!(CustomElementRegistryPlaceholder);
        define_placeholder!(AnimationPlaceholder);
        define_placeholder!(DOMExceptionPlaceholder);
        define_placeholder!(MessageChannelPlaceholder);
        define_placeholder!(MessagePortPlaceholder);
        define_placeholder!(AbortControllerPlaceholder);
        define_placeholder!(AbortSignalPlaceholder);
        define_placeholder!(PromisePlaceholder);
        
        /// The type holder for DOM types
        pub struct DomTypeHolder;
        
        impl DomTypes for DomTypeHolder {
            type GlobalScope = GlobalScopePlaceholder;
            type Window = WindowPlaceholder;
            type WindowProxy = WindowProxyPlaceholder;
            type Document = DocumentPlaceholder;
            type DocumentFragment = DocumentFragmentPlaceholder;
            type DocumentType = DocumentTypePlaceholder;
            type Element = ElementPlaceholder;
            type Node = NodePlaceholder;
            type Comment = CommentPlaceholder;
            type Text = TextPlaceholder;
            type CharacterData = CharacterDataPlaceholder;
            type Attr = AttrPlaceholder;
            type Event = EventPlaceholder;
            type EventTarget = EventTargetPlaceholder;
            type ShadowRoot = ShadowRootPlaceholder;
            type NodeList = NodeListPlaceholder;
            type HTMLCollection = HTMLCollectionPlaceholder;
            type HTMLOptionsCollection = HTMLOptionsCollectionPlaceholder;
            type HTMLFormControlsCollection = HTMLFormControlsCollectionPlaceholder;
            type DOMTokenList = DOMTokenListPlaceholder;
            type NamedNodeMap = NamedNodeMapPlaceholder;
            type FileList = FileListPlaceholder;
            type File = FilePlaceholder;
            type Blob = BlobPlaceholder;
            type CanvasRenderingContext2D = CanvasRenderingContext2DPlaceholder;
            type WebGLRenderingContext = WebGLRenderingContextPlaceholder;
            type WebGL2RenderingContext = WebGL2RenderingContextPlaceholder;
            type ImageData = ImageDataPlaceholder;
            type URL = URLPlaceholder;
            type Location = LocationPlaceholder;
            type History = HistoryPlaceholder;
            type Navigator = NavigatorPlaceholder;
            type Storage = StoragePlaceholder;
            type XMLHttpRequest = XMLHttpRequestPlaceholder;
            type Request = RequestPlaceholder;
            type Response = ResponsePlaceholder;
            type Headers = HeadersPlaceholder;
            type KeyboardEvent = KeyboardEventPlaceholder;
            type MouseEvent = MouseEventPlaceholder;
            type UIEvent = UIEventPlaceholder;
            type FocusEvent = FocusEventPlaceholder;
            type WheelEvent = WheelEventPlaceholder;
            type InputEvent = InputEventPlaceholder;
            type TouchEvent = TouchEventPlaceholder;
            type PointerEvent = PointerEventPlaceholder;
            type CompositionEvent = CompositionEventPlaceholder;
            type ClipboardEvent = ClipboardEventPlaceholder;
            type DragEvent = DragEventPlaceholder;
            type AnimationEvent = AnimationEventPlaceholder;
            type TransitionEvent = TransitionEventPlaceholder;
            type MessageEvent = MessageEventPlaceholder;
            type ErrorEvent = ErrorEventPlaceholder;
            type ProgressEvent = ProgressEventPlaceholder;
            type CustomEvent = CustomEventPlaceholder;
            type HTMLElement = HTMLElementPlaceholder;
            type HTMLFormElement = HTMLFormElementPlaceholder;
            type HTMLInputElement = HTMLInputElementPlaceholder;
            type HTMLButtonElement = HTMLButtonElementPlaceholder;
            type HTMLSelectElement = HTMLSelectElementPlaceholder;
            type HTMLTextAreaElement = HTMLTextAreaElementPlaceholder;
            type HTMLAnchorElement = HTMLAnchorElementPlaceholder;
            type HTMLImageElement = HTMLImageElementPlaceholder;
            type HTMLScriptElement = HTMLScriptElementPlaceholder;
            type HTMLStyleElement = HTMLStyleElementPlaceholder;
            type HTMLLinkElement = HTMLLinkElementPlaceholder;
            type HTMLDivElement = HTMLDivElementPlaceholder;
            type HTMLSpanElement = HTMLSpanElementPlaceholder;
            type HTMLParagraphElement = HTMLParagraphElementPlaceholder;
            type HTMLHeadingElement = HTMLHeadingElementPlaceholder;
            type HTMLBodyElement = HTMLBodyElementPlaceholder;
            type HTMLHtmlElement = HTMLHtmlElementPlaceholder;
            type HTMLHeadElement = HTMLHeadElementPlaceholder;
            type HTMLTableElement = HTMLTableElementPlaceholder;
            type HTMLTableRowElement = HTMLTableRowElementPlaceholder;
            type HTMLTableCellElement = HTMLTableCellElementPlaceholder;
            type HTMLCanvasElement = HTMLCanvasElementPlaceholder;
            type HTMLVideoElement = HTMLVideoElementPlaceholder;
            type HTMLAudioElement = HTMLAudioElementPlaceholder;
            type HTMLMediaElement = HTMLMediaElementPlaceholder;
            type HTMLIFrameElement = HTMLIFrameElementPlaceholder;
            type HTMLTemplateElement = HTMLTemplateElementPlaceholder;
            type HTMLSlotElement = HTMLSlotElementPlaceholder;
            type CSSStyleDeclaration = CSSStyleDeclarationPlaceholder;
            type StyleSheet = StyleSheetPlaceholder;
            type CSSStyleSheet = CSSStyleSheetPlaceholder;
            type CSSRule = CSSRulePlaceholder;
            type CSSStyleRule = CSSStyleRulePlaceholder;
            type Range = RangePlaceholder;
            type Selection = SelectionPlaceholder;
            type Worker = WorkerPlaceholder;
            type WorkerGlobalScope = WorkerGlobalScopePlaceholder;
            type DedicatedWorkerGlobalScope = DedicatedWorkerGlobalScopePlaceholder;
            type ServiceWorkerGlobalScope = ServiceWorkerGlobalScopePlaceholder;
            type DOMParser = DOMParserPlaceholder;
            type XMLSerializer = XMLSerializerPlaceholder;
            type TreeWalker = TreeWalkerPlaceholder;
            type NodeIterator = NodeIteratorPlaceholder;
            type MutationObserver = MutationObserverPlaceholder;
            type IntersectionObserver = IntersectionObserverPlaceholder;
            type ResizeObserver = ResizeObserverPlaceholder;
            type Performance = PerformancePlaceholder;
            type PerformanceEntry = PerformanceEntryPlaceholder;
            type CustomElementRegistry = CustomElementRegistryPlaceholder;
            type Animation = AnimationPlaceholder;
            type DOMException = DOMExceptionPlaceholder;
            type MessageChannel = MessageChannelPlaceholder;
            type MessagePort = MessagePortPlaceholder;
            type AbortController = AbortControllerPlaceholder;
            type AbortSignal = AbortSignalPlaceholder;
            type Promise = PromisePlaceholder;
        }
    }
    
    pub(crate) use crate::script_bindings::codegen::GenericBindings;
    
    // Re-export GenericBindings as Bindings so existing code continues to work
    pub(crate) use crate::script_bindings::codegen::GenericBindings as Bindings;
    
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
    pub(crate) use crate::script_bindings::codegen::UnionTypes;
}