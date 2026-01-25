// Copyright 2024-2026 Ferro Browser Contributors
// SPDX-License-Identifier: MIT
//
// Script Bindings Shim for Boa Engine
//
// This module provides compatibility with script_bindings API
// when using the Boa JavaScript engine instead of SpiderMonkey.
//
// Usage: When `js-boa` feature is enabled, this module is imported
// as `script_bindings` via `#[path]` directive in lib.rs.

// Import boa_bindings crate
extern crate boa_bindings;
extern crate boa_engine;
extern crate boa_gc;

// Re-export from boa_bindings where available
pub use ::boa_bindings::{
    // Root types
    Dom, DomRoot, Root, RootCollection, MaybeUnreflectedDom, assert_in_script, trace_roots,
    // Reflector types
    Reflector, DomObject, MutDomObject, DomRefCell,
    DomObjectWrap, Castable, DerivedFrom,
    // GC types
    Trace, Finalize, Gc, GcRefCell,
    // Error types
    JsResult, JsError, JsNativeError,
    // Settings stack
    settings_stack,
};

// Re-export DomTypes separately to avoid conflict
pub use ::boa_bindings::reflector::DomTypes;

// ============================================================================
// Stub modules that provide minimal API compatibility
// ============================================================================

pub mod root {
    //! Root types for DOM objects
    pub use ::boa_bindings::{
        Dom, DomRoot, Root, RootCollection, MaybeUnreflectedDom, 
        assert_in_script, trace_roots,
    };
    
    // Additional types that script expects
    pub use ::boa_bindings::root::{
        StableTraceObject, DomSlice, RootFromObject, DomExtractionError, DomExtractionResult,
    };
}

pub mod reflector {
    //! DOM object reflection
    pub use ::boa_bindings::reflector::*;
}

pub mod inheritance {
    //! DOM inheritance traits
    pub use ::boa_bindings::reflector::{Castable, DerivedFrom};
    
    /// Marker trait for parent relationship
    pub trait HasParent {
        type Parent;
    }
    
    /// Type ID for Node types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NodeTypeId {
        CharacterData(CharacterDataTypeId),
        Document,
        DocumentFragment,
        DocumentType,
        Element(ElementTypeId),
        Attr,
    }
    
    /// Type ID for CharacterData types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CharacterDataTypeId {
        Text(TextTypeId),
        Comment,
        ProcessingInstruction,
    }
    
    /// Type ID for Text types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TextTypeId {
        Text,
        CDATASection,
    }
    
    /// Type ID for Element types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ElementTypeId {
        HTMLElement(HTMLElementTypeId),
        SVGElement(SVGElementTypeId),
        Element,
    }
    
    /// Type ID for HTMLElement types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HTMLElementTypeId {
        HTMLElement,
        HTMLAnchorElement,
        HTMLAreaElement,
        HTMLAudioElement,
        HTMLBaseElement,
        HTMLBodyElement,
        HTMLBRElement,
        HTMLButtonElement,
        HTMLCanvasElement,
        HTMLDataElement,
        HTMLDataListElement,
        HTMLDetailsElement,
        HTMLDialogElement,
        HTMLDivElement,
        HTMLDListElement,
        HTMLEmbedElement,
        HTMLFieldSetElement,
        HTMLFontElement,
        HTMLFormElement,
        HTMLFrameElement,
        HTMLFrameSetElement,
        HTMLHeadElement,
        HTMLHeadingElement,
        HTMLHRElement,
        HTMLHtmlElement,
        HTMLIFrameElement,
        HTMLImageElement,
        HTMLInputElement,
        HTMLLabelElement,
        HTMLLegendElement,
        HTMLLIElement,
        HTMLLinkElement,
        HTMLMapElement,
        HTMLMediaElement,
        HTMLMetaElement,
        HTMLMeterElement,
        HTMLModElement,
        HTMLObjectElement,
        HTMLOListElement,
        HTMLOptGroupElement,
        HTMLOptionElement,
        HTMLOutputElement,
        HTMLParagraphElement,
        HTMLParamElement,
        HTMLPictureElement,
        HTMLPreElement,
        HTMLProgressElement,
        HTMLQuoteElement,
        HTMLScriptElement,
        HTMLSelectElement,
        HTMLSlotElement,
        HTMLSourceElement,
        HTMLSpanElement,
        HTMLStyleElement,
        HTMLTableElement,
        HTMLTableCaptionElement,
        HTMLTableCellElement,
        HTMLTableColElement,
        HTMLTableRowElement,
        HTMLTableSectionElement,
        HTMLTemplateElement,
        HTMLTextAreaElement,
        HTMLTimeElement,
        HTMLTitleElement,
        HTMLTrackElement,
        HTMLUListElement,
        HTMLVideoElement,
        HTMLUnknownElement,
    }
    
    /// Type ID for SVGElement types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SVGElementTypeId {
        SVGElement,
        SVGGraphicsElement(SVGGraphicsElementTypeId),
        SVGSVGElement,
    }
    
    /// Type ID for SVGGraphicsElement types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SVGGraphicsElementTypeId {
        SVGGraphicsElement,
        SVGSVGElement,
        SVGGElement,
        SVGImageElement,
        SVGTextContentElement,
        SVGUseElement,
    }
    
    /// Type ID for EventTarget types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EventTargetTypeId {
        EventTarget,
        Node(NodeTypeId),
        Window,
        Worker,
        WorkerGlobalScope,
        AudioNode,
        MediaQueryList,
        MessagePort,
        FileReader,
        XMLHttpRequest,
        EventSource,
        WebSocket,
        AbortSignal,
        Performance,
        PermissionStatus,
        Animation,
        FontFaceSet,
        MediaDevices,
        ServiceWorker,
        ServiceWorkerContainer,
        ServiceWorkerRegistration,
        BroadcastChannel,
        MediaRecorder,
        ScreenOrientation,
        RemotePlayback,
        SourceBuffer,
        SourceBufferList,
        TextTrack,
        TextTrackCue,
        TextTrackList,
        VideoTrackList,
        AudioTrackList,
        MediaSource,
        AudioScheduledSourceNode,
        MediaElementAudioSourceNode,
        AudioContext,
        BaseAudioContext,
        OfflineAudioContext,
        GPUDevice,
    }
    
    /// Type ID for DocumentFragment types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DocumentFragmentTypeId {
        DocumentFragment,
        ShadowRoot,
    }
    
    /// Type ID for HTMLMediaElement types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HTMLMediaElementTypeId {
        HTMLMediaElement,
        HTMLAudioElement,
        HTMLVideoElement,
    }
    
    /// Type ID for WorkerGlobalScope types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WorkerGlobalScopeTypeId {
        DedicatedWorkerGlobalScope,
        ServiceWorkerGlobalScope,
    }
    
    /// Type ID for GlobalScope types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum GlobalScopeTypeId {
        Window,
        WorkerGlobalScope(WorkerGlobalScopeTypeId),
    }
}

pub mod trace {
    //! GC tracing
    pub use ::boa_bindings::trace::*;
    pub use ::boa_bindings::{Trace, Finalize};
}

pub mod weakref {
    //! Weak references
    pub use ::boa_bindings::weakref::*;
}

pub mod conversions {
    //! Type conversions between Rust and JavaScript
    use ::boa_engine::{Context, JsResult, JsValue, JsString};
    use ::boa_engine::object::JsObject;
    
    pub use ::boa_bindings::js_compat::conversions::*;
    pub use ::boa_bindings::dom_conversions::{
        NativeFromObjectError, NativeFromObjectExt,
    };
    
    /// Trait for safe conversion to JS values
    pub trait SafeToJSValConvertible {
        /// Convert to a JS value safely
        fn safe_to_jsval(&self, cx: &mut Context) -> JsResult<JsValue>;
    }
    
    /// Trait for safe conversion from JS values
    pub trait SafeFromJSValConvertible: Sized {
        /// Convert from a JS value safely
        fn safe_from_jsval(cx: &mut Context, val: JsValue) -> JsResult<Self>;
    }
    
    /// Trait for IDL interface identification
    pub trait IDLInterface {
        /// Get the interface name
        fn derives(_prototypes: &[u16]) -> bool { false }
    }
    
    /// Check if value is a DOM proxy
    pub fn is_dom_proxy(_obj: &JsObject) -> bool {
        // TODO: implement when proxy support is added
        false
    }
    
    // Implementations for common types
    impl SafeToJSValConvertible for String {
        fn safe_to_jsval(&self, _cx: &mut Context) -> JsResult<JsValue> {
            Ok(JsValue::from(JsString::from(self.as_str())))
        }
    }
    
    impl SafeToJSValConvertible for &str {
        fn safe_to_jsval(&self, _cx: &mut Context) -> JsResult<JsValue> {
            Ok(JsValue::from(JsString::from(*self)))
        }
    }
    
    impl SafeToJSValConvertible for bool {
        fn safe_to_jsval(&self, _cx: &mut Context) -> JsResult<JsValue> {
            Ok(JsValue::from(*self))
        }
    }
    
    impl SafeToJSValConvertible for i32 {
        fn safe_to_jsval(&self, _cx: &mut Context) -> JsResult<JsValue> {
            Ok(JsValue::from(*self))
        }
    }
    
    impl SafeToJSValConvertible for u32 {
        fn safe_to_jsval(&self, _cx: &mut Context) -> JsResult<JsValue> {
            Ok(JsValue::from(*self))
        }
    }
    
    impl SafeToJSValConvertible for f64 {
        fn safe_to_jsval(&self, _cx: &mut Context) -> JsResult<JsValue> {
            Ok(JsValue::from(*self))
        }
    }
    
    impl SafeFromJSValConvertible for String {
        fn safe_from_jsval(cx: &mut Context, val: JsValue) -> JsResult<Self> {
            val.to_string(cx).map(|s| s.to_std_string_escaped())
        }
    }
    
    impl SafeFromJSValConvertible for bool {
        fn safe_from_jsval(_cx: &mut Context, val: JsValue) -> JsResult<Self> {
            Ok(val.to_boolean())
        }
    }
    
    impl SafeFromJSValConvertible for i32 {
        fn safe_from_jsval(cx: &mut Context, val: JsValue) -> JsResult<Self> {
            val.to_i32(cx)
        }
    }
    
    impl SafeFromJSValConvertible for f64 {
        fn safe_from_jsval(cx: &mut Context, val: JsValue) -> JsResult<Self> {
            val.to_number(cx)
        }
    }
    
    /// DerivedFrom trait - marker for DOM inheritance relationships
    pub trait DerivedFrom<T>: Sized {}
    
    /// Root from object - extract DOM root from JS object
    pub fn root_from_object<T>(_obj: *mut ::boa_bindings::js_compat::jsapi::JSObject) -> Option<::boa_bindings::DomRoot<T>> 
    where 
        T: ::boa_bindings::reflector::DomObject,
    {
        // Stub - returns None until proper implementation
        None
    }
}

pub mod error {
    //! Error handling
    use std::fmt;
    
    pub use ::boa_bindings::error::*;
    
    /// DOM exception error type
    #[derive(Debug, Clone)]
    pub enum Error {
        /// Index out of bounds
        IndexSize,
        /// Invalid hierarchy
        HierarchyRequest,
        /// Wrong document
        WrongDocument,
        /// Invalid character
        InvalidCharacter,
        /// No modification allowed
        NoModificationAllowed,
        /// Not found
        NotFound,
        /// Not supported
        NotSupported,
        /// Invalid state
        InvalidState,
        /// Syntax error
        Syntax,
        /// Invalid modification
        InvalidModification,
        /// Namespace error
        Namespace,
        /// Invalid access
        InvalidAccess,
        /// Security error
        Security,
        /// Network error
        Network,
        /// Abort error
        Abort,
        /// URL mismatch
        URLMismatch,
        /// Quota exceeded
        QuotaExceeded,
        /// Timeout
        Timeout,
        /// Invalid node type
        InvalidNodeType,
        /// Data clone error
        DataClone,
        /// Not readable
        NotReadable,
        /// Encoding error
        EncodingError,
        /// Type error with message
        Type(String),
        /// Range error with message
        Range(String),
        /// JS exception
        JSFailed,
    }
    
    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Error::IndexSize => write!(f, "IndexSizeError"),
                Error::HierarchyRequest => write!(f, "HierarchyRequestError"),
                Error::WrongDocument => write!(f, "WrongDocumentError"),
                Error::InvalidCharacter => write!(f, "InvalidCharacterError"),
                Error::NoModificationAllowed => write!(f, "NoModificationAllowedError"),
                Error::NotFound => write!(f, "NotFoundError"),
                Error::NotSupported => write!(f, "NotSupportedError"),
                Error::InvalidState => write!(f, "InvalidStateError"),
                Error::Syntax => write!(f, "SyntaxError"),
                Error::InvalidModification => write!(f, "InvalidModificationError"),
                Error::Namespace => write!(f, "NamespaceError"),
                Error::InvalidAccess => write!(f, "InvalidAccessError"),
                Error::Security => write!(f, "SecurityError"),
                Error::Network => write!(f, "NetworkError"),
                Error::Abort => write!(f, "AbortError"),
                Error::URLMismatch => write!(f, "URLMismatchError"),
                Error::QuotaExceeded => write!(f, "QuotaExceededError"),
                Error::Timeout => write!(f, "TimeoutError"),
                Error::InvalidNodeType => write!(f, "InvalidNodeTypeError"),
                Error::DataClone => write!(f, "DataCloneError"),
                Error::NotReadable => write!(f, "NotReadableError"),
                Error::EncodingError => write!(f, "EncodingError"),
                Error::Type(msg) => write!(f, "TypeError: {}", msg),
                Error::Range(msg) => write!(f, "RangeError: {}", msg),
                Error::JSFailed => write!(f, "JSError"),
            }
        }
    }
    
    impl std::error::Error for Error {}
    
    /// Result type for fallible DOM operations
    pub type Fallible<T> = Result<T, Error>;
    
    /// Result type for error results
    pub type ErrorResult = Fallible<()>;
}

pub mod str {
    //! DOM string types
    use std::borrow::Cow;
    use std::ops::Deref;
    
    /// DOM string type
    #[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
    pub struct DOMString(String);
    
    impl DOMString {
        /// Create a new DOMString
        pub fn new() -> Self {
            DOMString(String::new())
        }
        
        /// Create from a string
        pub fn from_string(s: String) -> Self {
            DOMString(s)
        }
        
        /// Get as str
        pub fn as_str(&self) -> &str {
            &self.0
        }
        
        /// Check if empty
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
        
        /// Get length
        pub fn len(&self) -> usize {
            self.0.len()
        }
        
        /// Convert to lowercase
        pub fn to_lowercase(&self) -> DOMString {
            DOMString(self.0.to_lowercase())
        }
        
        /// Push a char
        pub fn push(&mut self, c: char) {
            self.0.push(c);
        }
        
        /// Push a string
        pub fn push_str(&mut self, s: &str) {
            self.0.push_str(s);
        }
        
        /// Trim whitespace
        pub fn trim(&self) -> &str {
            self.0.trim()
        }
    }
    
    impl From<String> for DOMString {
        fn from(s: String) -> Self {
            DOMString(s)
        }
    }
    
    impl From<&str> for DOMString {
        fn from(s: &str) -> Self {
            DOMString(s.to_string())
        }
    }
    
    impl From<Cow<'_, str>> for DOMString {
        fn from(s: Cow<'_, str>) -> Self {
            DOMString(s.into_owned())
        }
    }
    
    impl Deref for DOMString {
        type Target = str;
        
        fn deref(&self) -> &str {
            &self.0
        }
    }
    
    impl AsRef<str> for DOMString {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }
    
    impl std::fmt::Display for DOMString {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    
    /// Byte string type
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct ByteString(Vec<u8>);
    
    impl ByteString {
        /// Create new empty ByteString
        pub fn new() -> Self {
            ByteString(Vec::new())
        }
        
        /// Get as bytes
        pub fn as_bytes(&self) -> &[u8] {
            &self.0
        }
    }
    
    impl From<Vec<u8>> for ByteString {
        fn from(v: Vec<u8>) -> Self {
            ByteString(v)
        }
    }
    
    /// USV string type (valid Unicode scalar values)
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct USVString(String);
    
    impl USVString {
        /// Create from string
        pub fn new(s: String) -> Self {
            USVString(s)
        }
        
        /// Get as str
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }
    
    impl From<String> for USVString {
        fn from(s: String) -> Self {
            USVString(s)
        }
    }
    
    impl Deref for USVString {
        type Target = str;
        
        fn deref(&self) -> &str {
            &self.0
        }
    }
}

pub mod script_runtime {
    //! Script runtime types
    use ::boa_engine::Context;
    
    /// JSContext type alias
    pub type JSContext = Context;
    
    /// Marker type for operations that can trigger GC
    #[derive(Debug, Clone, Copy)]
    pub struct CanGc;
    
    impl CanGc {
        /// Create a new CanGc marker
        pub fn note() -> Self {
            CanGc
        }
    }
}

pub mod callback {
    //! Callback handling
    
    /// Exception handling mode for callbacks
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ExceptionHandling {
        /// Report exceptions to console
        Report,
        /// Rethrow exceptions
        Rethrow,
    }
}

pub mod num {
    //! Numeric types for WebIDL
    
    /// A finite floating point number
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Finite<T>(T);
    
    impl<T: std::fmt::Debug + Copy> Finite<T> {
        /// Create a new finite number (panics if not finite for f32/f64)
        pub fn new(value: T) -> Option<Self> {
            Some(Finite(value))
        }
        
        /// Get the wrapped value
        pub fn get(self) -> T {
            self.0
        }
    }
    
    impl Finite<f64> {
        /// Wrap a finite f64
        pub fn wrap(value: f64) -> Option<Self> {
            if value.is_finite() {
                Some(Finite(value))
            } else {
                None
            }
        }
    }
    
    impl<T> std::ops::Deref for Finite<T> {
        type Target = T;
        
        fn deref(&self) -> &T {
            &self.0
        }
    }
}

pub mod iterable {
    //! Iterable interface support
    use std::marker::PhantomData;
    
    /// Marker for iterable interfaces
    pub trait Iterable {}
    
    /// Iterable iterator - generic iterator for WebIDL iterables
    pub struct IterableIterator<T, I> {
        _inner: PhantomData<(T, I)>,
    }
    
    impl<T, I> IterableIterator<T, I> {
        /// Create a new iterator
        pub fn new(_iterable: &T, _kind: IteratorKind) -> Self {
            Self { _inner: PhantomData }
        }
    }
    
    /// Iterator kind (keys, values, or entries)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum IteratorKind {
        Keys,
        Values,
        Entries,
    }
}

pub mod record {
    //! WebIDL record type
    use std::collections::HashMap;
    
    /// A WebIDL record (like a HashMap)
    #[derive(Debug, Clone, Default)]
    pub struct Record<K, V>(HashMap<K, V>);
    
    impl<K: std::hash::Hash + Eq, V> Record<K, V> {
        /// Create empty record
        pub fn new() -> Self {
            Record(HashMap::new())
        }
        
        /// Get value by key
        pub fn get(&self, key: &K) -> Option<&V> {
            self.0.get(key)
        }
        
        /// Insert value
        pub fn insert(&mut self, key: K, value: V) {
            self.0.insert(key, value);
        }
        
        /// Iterate over entries
        pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
            self.0.iter()
        }
    }
}

pub mod principals {
    //! Security principals
    
    /// JavaScript principals for security checks
    #[derive(Debug)]
    pub struct ServoJSPrincipals;
    
    /// Reference to principals
    pub type ServoJSPrincipalsRef = Option<&'static ServoJSPrincipals>;
}

pub mod proxyhandler {
    //! Proxy handler support (stub)
    use std::ffi::c_void;
    
    /// Set property descriptor
    pub unsafe fn set_property_descriptor(
        _desc: *mut crate::js::glue::PropertyDescriptor,
        _value: crate::js::jsapi::Value,
        _attrs: u32,
        _getter: *mut c_void,
        _setter: *mut c_void,
    ) {
        // Stub implementation
    }
}

pub mod structuredclone {
    //! Structured clone algorithm
    
    /// Marker trait for types that can be serialized
    pub trait MarkedAsSerializableInIdl {}
    
    /// Marker trait for types that can be transferred
    pub trait MarkedAsTransferableInIdl {}
}

pub mod domstring {
    //! DOM string utilities
    
    /// View into bytes
    pub struct BytesView<'a>(&'a [u8]);
    
    impl<'a> BytesView<'a> {
        /// Create from slice
        pub fn new(bytes: &'a [u8]) -> Self {
            BytesView(bytes)
        }
        
        /// Get as slice
        pub fn as_slice(&self) -> &[u8] {
            self.0
        }
    }
    
    /// Parse a floating point number from string
    pub fn parse_floating_point_number(s: &str) -> Option<f64> {
        s.trim().parse::<f64>().ok()
    }
}

pub mod codegen {
    //! Generated bindings (stubs for Boa)
    
    pub use ::boa_bindings::codegen::*;
    
    /// Prototype list (stub)
    pub mod PrototypeList {
        /// Maximum prototype ID
        pub const PROTO_ID_MAX: u16 = 1000;
        
        /// Prototype IDs
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct ID(pub u16);
    }
    
    /// Generic bindings stubs
    pub mod GenericBindings {
        pub mod CharacterDataBinding {
            pub trait CharacterDataMethods<D> {}
            pub mod CharacterData_Binding {
                pub trait CharacterDataMethods<D> {}
            }
        }
        pub mod DOMRectBinding {
            pub trait DOMRectMethods<D> {}
            pub mod DOMRect_Binding {
                pub trait DOMRectMethods<D> {}
            }
        }
        pub mod DocumentBinding {
            pub trait DocumentMethods<D> {}
            pub mod Document_Binding {
                pub trait DocumentMethods<D> {}
            }
        }
        pub mod ElementBinding {
            pub trait ElementMethods<D> {}
            pub mod Element_Binding {
                pub trait ElementMethods<D> {}
            }
        }
        pub mod MouseEventBinding {
            pub trait MouseEventMethods<D> {}
            pub mod MouseEvent_Binding {
                pub trait MouseEventMethods<D> {}
            }
        }
        pub mod UIEventBinding {
            pub trait UIEventMethods<D> {}
            pub mod UIEvent_Binding {
                pub trait UIEventMethods<D> {}
            }
        }
        pub mod MediaSourceBinding {
            pub trait MediaSourceMethods<D> {}
            pub mod MediaSource_Binding {
                pub trait MediaSourceMethods<D> {}
            }
            pub mod MediaSourceReadyState {
                pub const Closed: u8 = 0;
                pub const Open: u8 = 1;
                pub const Ended: u8 = 2;
            }
            pub mod EndOfStreamError {
                pub const Network: u8 = 0;
                pub const Decode: u8 = 1;
            }
        }
        pub mod SourceBufferListBinding {
            pub trait SourceBufferListMethods<D> {}
            pub mod SourceBufferList_Binding {
                pub trait SourceBufferListMethods<D> {}
            }
        }
        pub mod SourceBufferBinding {
            pub trait SourceBufferMethods<D> {}
            pub mod SourceBuffer_Binding {
                pub trait SourceBufferMethods<D> {}
            }
        }
        pub mod WindowBinding {
            pub trait WindowMethods<D> {}
            pub mod Window_Binding {
                pub trait WindowMethods<D> {}
            }
        }
        pub mod CSSRuleListBinding {
            pub trait CSSRuleListMethods<D> {}
            pub mod CSSRuleList_Binding {
                pub trait CSSRuleListMethods<D> {}
            }
        }
        pub mod CSSStyleSheetBinding {
            pub trait CSSStyleSheetMethods<D> {}
            pub mod CSSStyleSheet_Binding {
                pub trait CSSStyleSheetMethods<D> {}
            }
        }
        pub mod CSSRuleBinding {
            pub trait CSSRuleMethods<D> {}
            pub mod CSSRule_Binding {
                pub trait CSSRuleMethods<D> {}
            }
        }
        pub mod NodeBinding {
            pub trait NodeMethods<D> {}
            pub mod Node_Binding {
                pub trait NodeMethods<D> {}
            }
        }
        pub mod HTMLOptGroupElementBinding {
            pub trait HTMLOptGroupElementMethods<D> {}
            pub mod HTMLOptGroupElement_Binding {
                pub trait HTMLOptGroupElementMethods<D> {}
            }
        }
        pub mod WritableStreamDefaultWriterBinding {
            pub trait WritableStreamDefaultWriterMethods<D> {}
            pub mod WritableStreamDefaultWriter_Binding {
                pub trait WritableStreamDefaultWriterMethods<D> {}
            }
        }
        pub mod ShadowRootBinding {
            pub trait ShadowRootMethods<D> {}
            pub mod ShadowRoot_Binding {
                pub trait ShadowRootMethods<D> {}
            }
        }
    }
    
    /// Generic union types
    pub mod GenericUnionTypes {
        /// String or string sequence union
        pub enum StringOrStringSequence {
            String(String),
            StringSequence(Vec<String>),
        }
    }
    
    /// WebIDL union types for DOM APIs
    pub mod UnionTypes {
        /// TrustedHTML or TrustedScript or TrustedScriptURL or String union
        pub enum TrustedHTMLOrTrustedScriptOrTrustedScriptURLOrString {
            TrustedHTML(()),
            TrustedScript(()),
            TrustedScriptURL(()),
            String(String),
        }
        
        /// AudioContextLatencyCategory or Double union
        pub enum AudioContextLatencyCategoryOrDouble {
            AudioContextLatencyCategory(u8),
            Double(f64),
        }
        
        /// ArrayBuffer or ArrayBufferView or Blob or String union
        pub enum ArrayBufferOrArrayBufferViewOrBlobOrString {
            ArrayBuffer(Vec<u8>),
            ArrayBufferView(Vec<u8>),
            Blob(()),
            String(String),
        }
        
        /// Generic unions for various DOM types
        pub enum StringOrUnsignedLong {
            String(String),
            UnsignedLong(u32),
        }
        
        pub enum StringOrBoolean {
            String(String),
            Boolean(bool),
        }
        
        pub enum StringOrDocument {
            String(String),
            Document(()),
        }
        
        pub enum BlobOrString {
            Blob(()),
            String(String),
        }
        
        pub enum FileOrString {
            File(()),
            String(String),
        }
        
        pub enum StringOrCanvasGradientOrCanvasPattern {
            String(String),
            CanvasGradient(()),
            CanvasPattern(()),
        }
        
        pub enum HTMLCanvasElementOrOffscreenCanvas {
            HTMLCanvasElement(()),
            OffscreenCanvas(()),
        }
        
        pub enum NodeOrString {
            Node(()),
            String(String),
        }
        
        pub enum ArrayBufferViewOrArrayBuffer {
            ArrayBufferView(Vec<u8>),
            ArrayBuffer(Vec<u8>),
        }
        
        pub enum StringOrArrayBufferViewOrArrayBuffer {
            String(String),
            ArrayBufferView(Vec<u8>),
            ArrayBuffer(Vec<u8>),
        }
        
        pub enum MediaListOrString {
            MediaList(()),
            String(String),
        }
        
        pub enum StringOrElementCreationOptions {
            String(String),
            ElementCreationOptions(()),
        }
        
        pub enum TrustedHTMLOrString {
            TrustedHTML(()),
            String(String),
        }
        
        pub enum TrustedScriptOrString {
            TrustedScript(()),
            String(String),
        }
        
        pub enum TrustedScriptURLOrString {
            TrustedScriptURL(()),
            String(String),
        }
        
        pub enum StringOrUnrestrictedDoubleSequence {
            String(String),
            UnrestrictedDoubleSequence(Vec<f64>),
        }
        
        pub enum BooleanOrScrollIntoViewOptions {
            Boolean(bool),
            ScrollIntoViewOptions(()),
        }
        
        pub enum TrustedHTMLOrNullIsEmptyString {
            TrustedHTML(()),
            NullIsEmptyString(String),
        }
        
        pub enum TrustedScriptURLOrUSVString {
            TrustedScriptURL(()),
            USVString(String),
        }
        
        pub enum FileOrUSVStringOrFormData {
            File(()),
            USVString(String),
            FormData(()),
        }
        
        pub enum FileOrUSVString {
            File(()),
            USVString(String),
        }
        
        pub enum AddEventListenerOptionsOrBoolean {
            AddEventListenerOptions(()),
            Boolean(bool),
        }
        
        pub enum EventListenerOptionsOrBoolean {
            EventListenerOptions(()),
            Boolean(bool),
        }
        
        pub enum EventOrString {
            Event(()),
            String(String),
        }
        
        pub enum StringOrObject {
            String(String),
            Object(()),
        }
        
        pub enum ElementOrText {
            Element(()),
            Text(()),
        }
        
        pub enum RadioNodeListOrElement {
            RadioNodeList(()),
            Element(()),
        }
        
        pub enum DoubleOrAutoKeyword {
            Double(f64),
            AutoKeyword(()),
        }
        
        pub enum Float32ArrayOrUnrestrictedDoubleSequence {
            Float32Array(()),
            UnrestrictedDoubleSequence(Vec<f64>),
        }
        
        pub enum ElementOrRadioNodeList {
            Element(()),
            RadioNodeList(()),
        }
        
        pub enum HTMLCollectionOrElement {
            HTMLCollection(()),
            Element(()),
        }
        
        pub enum DocumentOrXMLHttpRequestBodyInit {
            Document(()),
            XMLHttpRequestBodyInit(()),
        }
        
        pub enum BlobOrBufferSourceOrFormDataOrURLSearchParamsOrString {
            Blob(()),
            BufferSource(()),
            FormData(()),
            URLSearchParams(()),
            String(String),
        }
        
        pub enum USVStringOrURLSearchParams {
            USVString(String),
            URLSearchParams(()),
        }
        
        pub enum GPUColorOrSequence {
            GPUColor(()),
            DoubleSequence(Vec<f64>),
        }
        
        pub enum GPUOrigin2DOrSequence {
            GPUOrigin2D(()),
            GPUIntegerCoordinateSequence(Vec<u32>),
        }
        
        pub enum GPUOrigin3DOrSequence {
            GPUOrigin3D(()),
            GPUIntegerCoordinateSequence(Vec<u32>),
        }
        
        pub enum GPUExtent3DOrSequence {
            GPUExtent3D(()),
            GPUIntegerCoordinateSequence(Vec<u32>),
        }
        
        pub enum RequestInfoOrURL {
            Request(()),
            URL(String),
        }
        
        pub enum RequestOrUSVString {
            Request(()),
            USVString(String),
        }
        
        pub enum BodyInit {
            Blob(()),
            BufferSource(()),
            FormData(()),
            URLSearchParams(()),
            String(String),
            ReadableStream(()),
        }
        
        pub enum HTMLOrSVGScriptElement {
            HTMLScriptElement(()),
            SVGScriptElement(()),
        }
        
        pub enum HTMLOrSVGImageElement {
            HTMLImageElement(()),
            SVGImageElement(()),
        }
        
        pub enum CanvasImageSource {
            HTMLImageElement(()),
            HTMLCanvasElement(()),
            HTMLVideoElement(()),
            ImageBitmap(()),
            OffscreenCanvas(()),
            VideoFrame(()),
        }
        
        pub enum TexImageSource {
            ImageBitmap(()),
            ImageData(()),
            HTMLImageElement(()),
            HTMLCanvasElement(()),
            HTMLVideoElement(()),
            OffscreenCanvas(()),
            VideoFrame(()),
        }
        
        pub enum ImageBitmapSource {
            Blob(()),
            ImageData(()),
            ImageBitmap(()),
            HTMLImageElement(()),
            HTMLCanvasElement(()),
            HTMLVideoElement(()),
            OffscreenCanvas(()),
            VideoFrame(()),
        }
        
        pub enum ClipboardItemData {
            String(String),
            Blob(()),
        }
        
        pub enum ClipboardItems {
            Items(Vec<()>),
        }
        
        pub enum Transferable {
            ArrayBuffer(()),
            MessagePort(()),
            ImageBitmap(()),
            OffscreenCanvas(()),
        }
        
        pub enum MessageEventSource {
            Window(()),
            MessagePort(()),
            ServiceWorker(()),
        }
        
        pub enum HTMLElementOrLong {
            HTMLElement(()),
            Long(i32),
        }
        
        pub enum NodeOrLong {
            Node(()),
            Long(i32),
        }
        
        pub enum MediaStreamOrMediaSource {
            MediaStream(()),
            MediaSource(()),
        }
        
        pub enum MediaStreamOrBlob {
            MediaStream(()),
            Blob(()),
        }
        
        pub enum VideoTrackOrAudioTrackOrTextTrack {
            VideoTrack(()),
            AudioTrack(()),
            TextTrack(()),
        }
        
        pub enum HTMLOptionElementOrHTMLOptGroupElement {
            HTMLOptionElement(()),
            HTMLOptGroupElement(()),
        }
        
        pub enum IDBObjectStoreOrIDBIndex {
            IDBObjectStore(()),
            IDBIndex(()),
        }
        
        pub enum StringOrStringSequence {
            String(String),
            StringSequence(Vec<String>),
        }
        
        pub enum DoubleOrDoubleSequence {
            Double(f64),
            DoubleSequence(Vec<f64>),
        }
        
        pub enum ElementOrDocument {
            Element(()),
            Document(()),
        }
        
        pub enum BooleanOrMediaTrackConstraints {
            Boolean(bool),
            MediaTrackConstraints(()),
        }
        
        pub enum ClampedUnsignedLongOrConstrainULongRange {
            ClampedUnsignedLong(u32),
            ConstrainULongRange(()),
        }
        
        pub enum DoubleOrConstrainDoubleRange {
            Double(f64),
            ConstrainDoubleRange(()),
        }
        
        pub enum WindowProxyOrMessagePortOrServiceWorker {
            WindowProxy(()),
            MessagePort(()),
            ServiceWorker(()),
        }
        
        pub enum UnsignedLongOrUnsignedLongSequence {
            UnsignedLong(u32),
            UnsignedLongSequence(Vec<u32>),
        }
        
        pub enum StringOrDouble {
            String(String),
            Double(f64),
        }
        
        pub enum StringOrPerformanceMeasureOptions {
            String(String),
            PerformanceMeasureOptions(()),
        }
        
        pub enum ReadableStreamDefaultControllerOrReadableByteStreamController {
            ReadableStreamDefaultController(()),
            ReadableByteStreamController(()),
        }
        
        pub enum ReadableStreamDefaultReaderOrReadableStreamBYOBReader {
            ReadableStreamDefaultReader(()),
            ReadableStreamBYOBReader(()),
        }
        
        pub enum ArrayBufferViewOrArrayBufferOrJsonWebKey {
            ArrayBufferView(Vec<u8>),
            ArrayBuffer(Vec<u8>),
            JsonWebKey(()),
        }
        
        pub enum ObjectOrString {
            Object(()),
            String(String),
        }
        
        pub enum USVStringOrURLPatternInit {
            USVString(String),
            URLPatternInit(()),
        }
        
        pub enum USVStringSequenceSequenceOrUSVStringUSVStringRecordOrUSVString {
            USVStringSequenceSequence(Vec<Vec<String>>),
            USVStringUSVStringRecord(()),
            USVString(String),
        }
        
        pub enum USVStringOrUndefined {
            USVString(String),
            Undefined,
        }
        
        pub enum LongOrLongSequence {
            Long(i32),
            LongSequence(Vec<i32>),
        }
        
        pub enum StringOrLong {
            String(String),
            Long(i32),
        }
        
        pub enum NodeFilterOrFunction {
            NodeFilter(()),
            Function(()),
        }
        
        pub enum ByteStringOrByteStringSequence {
            ByteString(Vec<u8>),
            ByteStringSequence(Vec<Vec<u8>>),
        }
        
        pub enum DOMStringOrArrayBuffer {
            DOMString(String),
            ArrayBuffer(Vec<u8>),
        }
        
        pub enum HTMLImageElementOrSVGImageElement {
            HTMLImageElement(()),
            SVGImageElement(()),
        }
        
        pub enum HTMLScriptElementOrSVGScriptElement {
            HTMLScriptElement(()),
            SVGScriptElement(()),
        }
        
        pub enum SequenceOfElementsOrElement {
            SequenceOfElements(Vec<()>),
            Element(()),
        }
        
        pub enum ArrayBufferOrUSVString {
            ArrayBuffer(Vec<u8>),
            USVString(String),
        }
        
        pub enum FormDataOrURLSearchParams {
            FormData(()),
            URLSearchParams(()),
        }
        
        pub enum BlobOrMediaSource {
            Blob(()),
            MediaSource(()),
        }
        
        pub enum AudioBufferOrMediaStream {
            AudioBuffer(()),
            MediaStream(()),
        }
        
        pub enum HTMLMediaElement {
            HTMLVideoElement(()),
            HTMLAudioElement(()),
        }
        
        pub enum StringOrBool {
            String(String),
            Bool(bool),
        }
        
        pub enum LongOrBoolean {
            Long(i32),
            Boolean(bool),
        }
        
        pub enum DoubleOrString {
            Double(f64),
            String(String),
        }
        
        pub enum RequestOrUSVStringSequence {
            Request(()),
            USVStringSequence(Vec<String>),
        }
        
        pub enum ElementOrHTMLCollection {
            Element(()),
            HTMLCollection(()),
        }
        
        pub enum DOMMatrixInit {
            DOMMatrix2DInit(()),
            DOMMatrixInit(()),
        }
        
        pub enum BlobPart {
            ArrayBuffer(Vec<u8>),
            ArrayBufferView(Vec<u8>),
            Blob(()),
            String(String),
        }
        
        pub enum ConstrainBoolean {
            Boolean(bool),
            ConstrainBooleanParameters(()),
        }
        
        pub enum ConstrainDOMString {
            String(String),
            StringSequence(Vec<String>),
            ConstrainDOMStringParameters(()),
        }
        
        pub enum ConstrainDouble {
            Double(f64),
            ConstrainDoubleRange(()),
        }
        
        pub enum ConstrainULong {
            UnsignedLong(u32),
            ConstrainULongRange(()),
        }
        
        pub enum AudioContextLatencyCategory {
            Category(String),
            Seconds(f64),
        }
        
        pub enum Float32ArrayOrUnrestrictedFloatSequence {
            Float32Array(()),
            UnrestrictedFloatSequence(Vec<f32>),
        }
        
        pub enum Int32ArrayOrLongSequence {
            Int32Array(()),
            LongSequence(Vec<i32>),
        }
        
        pub enum Uint32ArrayOrUnsignedLongSequence {
            Uint32Array(()),
            UnsignedLongSequence(Vec<u32>),
        }
        
        pub enum MediaStreamTrackOrString {
            MediaStreamTrack(()),
            String(String),
        }
        
        pub enum TrustedScriptOrStringOrFunction {
            TrustedScript(()),
            String(String),
            Function(()),
        }
        
        pub enum DocumentOrBlobOrArrayBufferViewOrArrayBufferOrFormDataOrStringOrURLSearchParams {
            Document(()),
            Blob(()),
            ArrayBufferView(Vec<u8>),
            ArrayBuffer(Vec<u8>),
            FormData(()),
            String(String),
            URLSearchParams(()),
        }
    }
    
    /// Inheritance types (stub)
    pub mod InheritTypes {}
    
    /// Register bindings function
    pub mod RegisterBindings {
        use ::boa_engine::Context;
        
        /// Register all DOM bindings
        pub fn register(_cx: &mut Context) {
            // TODO: register DOM constructors
        }
    }
}

pub mod interfaces {
    //! Interface helpers
    use ::boa_engine::{Context, JsObject, JsResult};
    
    /// Trait for DOM helpers
    pub trait DomHelpers {
        /// Get the global object
        fn global_object(cx: &mut Context) -> JsResult<JsObject>;
    }
    
    /// Trait for global scope helpers
    pub trait GlobalScopeHelpers {
        /// Get the global scope
        fn from_context(cx: &mut Context) -> JsResult<JsObject>;
    }
    
    /// Interface marker trait
    pub trait Interface {
        /// Get the interface name
        fn interface_name() -> &'static str;
    }
}

pub mod interface {
    //! Interface utilities
    use ::boa_engine::{Context, JsObject, JsResult};
    
    /// Get the desired prototype for a constructor
    pub fn get_desired_proto(
        _cx: &mut Context,
        _args: &[boa_engine::JsValue],
        _proto: &JsObject,
    ) -> JsResult<Option<JsObject>> {
        Ok(None)
    }
}

pub mod utils {
    //! Utility types
    
    /// DOM class descriptor
    #[repr(C)]
    pub struct DOMClass {
        /// Interface chain
        pub interface_chain: [u16; 8],
        /// Depth in prototype chain
        pub depth: u16,
        /// Type ID
        pub type_id: std::any::TypeId,
    }
}

// Note: DomTypes is exported at line 33, not here to avoid duplicate export

/// Macro for matching DOM strings (ASCII case-insensitive)
#[macro_export]
macro_rules! match_domstring_ascii {
    ($s:expr, { $($pattern:literal => $result:expr),* $(,)? }) => {
        match $s.to_ascii_lowercase().as_str() {
            $($pattern => $result,)*
            _ => {}
        }
    };
}

pub use match_domstring_ascii;
/// Like module - provides pattern matching utilities (SpiderMonkey-like API)
pub mod like {
    /// Marker type for "like" pattern matching
    pub struct Like<T>(pub T);
    
    /// Marker for pattern matching operations
    pub trait LikePattern<T> {
        fn matches(&self, value: &T) -> bool;
    }
}