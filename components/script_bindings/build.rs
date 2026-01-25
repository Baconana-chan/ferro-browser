/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use std::{env, fmt};

use phf_shared::{self, FmtConst};
use serde_json::{self, Value};

fn main() {
    let start = Instant::now();

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:out_dir={}", out_dir.display());

    // Check if we're building for Boa (skip SpiderMonkey codegen)
    let is_boa = env::var("CARGO_FEATURE_JS_BOA").is_ok();
    
    if is_boa {
        // For Boa, generate minimal stubs instead of full SpiderMonkey bindings
        println!("cargo:warning=Building script_bindings for Boa - skipping SpiderMonkey codegen");
        generate_boa_stubs(&out_dir);
        println!("Boa stub generation completed in {:?}", start.elapsed());
        return;
    }

    let style_out_dir = PathBuf::from(env::var_os("DEP_SERVO_STYLE_CRATE_OUT_DIR").unwrap());
    let css_properties_json = style_out_dir.join("css-properties.json");

    println!("cargo::rerun-if-changed=webidls");
    println!("cargo::rerun-if-changed=codegen");
    println!("cargo::rerun-if-changed={}", css_properties_json.display());
    println!("cargo::rerun-if-changed=../../third_party/WebIDL/WebIDL.py");
    // NB: We aren't handling changes in `third_party/ply` here.

    let status = find_python()
        .arg("codegen/run.py")
        .arg(&css_properties_json)
        .arg(&out_dir)
        .status()
        .unwrap();
    if !status.success() {
        std::process::exit(1)
    }

    println!("Binding generation completed in {:?}", start.elapsed());

    let json = out_dir.join("InterfaceObjectMapData.json");
    let json: Value = serde_json::from_reader(File::open(json).unwrap()).unwrap();
    let mut map = phf_codegen::Map::new();
    for (key, value) in json.as_object().unwrap() {
        let parts = value.as_array().unwrap();
        map.entry(
            Bytes(key),
            format!(
                "Interface {{ define: {}, enabled: {} }}",
                parts[0].as_str().unwrap(),
                parts[1].as_str().unwrap()
            ),
        );
    }
    let phf = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("InterfaceObjectMapPhf.rs");
    let mut phf = File::create(phf).unwrap();
    writeln!(
        &mut phf,
        "pub(crate) static MAP: phf::Map<&'static [u8], Interface> = {};",
        map.build(),
    )
    .unwrap();
}

#[derive(Eq, Hash, PartialEq)]
struct Bytes<'a>(&'a str);

impl FmtConst for Bytes<'_> {
    fn fmt_const(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "b\"{}\"", self.0)
    }
}

impl phf_shared::PhfHash for Bytes<'_> {
    fn phf_hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.0.as_bytes().phf_hash(hasher)
    }
}

/// Tries to find a suitable python, which in Servo is always `uv run python` unless we are running
/// as a descendant of `uv run python`. In that case, we can use either `uv run python` or `python`
/// (uv does not provide a `python3` on Windows).
///
/// More details: <https://book.servo.org/hacking/setting-up-your-environment.html#check-tools>
///
/// Note: This function should be kept in sync with the version in `components/script/build.rs`
fn find_python() -> Command {
    let mut command = Command::new("uv");
    command.args(["run", "--no-project", "python"]);

    if command.output().is_ok_and(|out| out.status.success()) {
        return command;
    }

    panic!("Can't find python (tried `{command:?}`)! Is uv installed and in PATH?")
}

/// Generate minimal stubs for Boa engine (no SpiderMonkey bindings needed)
fn generate_boa_stubs(out_dir: &PathBuf) {
    use std::fs;
    
    // Create Bindings directory
    let bindings_dir = out_dir.join("Bindings");
    fs::create_dir_all(&bindings_dir).unwrap();
    
    // Create mod.rs for Bindings with submodules
    let mod_rs = bindings_dir.join("mod.rs");
    fs::write(&mod_rs, r#"
// Auto-generated stub bindings for Boa engine
// These are placeholder modules - actual DOM bindings use boa_bindings

pub mod WindowBinding;
pub mod EventModifierInitBinding;
pub mod IterableIteratorBinding;
"#).unwrap();

    // Create WindowBinding.rs stub
    let window_binding = bindings_dir.join("WindowBinding.rs");
    fs::write(&window_binding, r#"
// Auto-generated WindowBinding stub for Boa engine
pub struct WindowMethods;
"#).unwrap();

    // Create EventModifierInitBinding.rs stub  
    let event_modifier_binding = bindings_dir.join("EventModifierInitBinding.rs");
    fs::write(&event_modifier_binding, r#"
// Auto-generated EventModifierInitBinding stub for Boa engine

/// EventModifierInit dictionary
#[derive(Clone, Debug, Default)]
pub struct EventModifierInit {
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
    pub modifier_alt_graph: bool,
    pub modifier_caps_lock: bool,
    pub modifier_fn: bool,
    pub modifier_fn_lock: bool,
    pub modifier_hyper: bool,
    pub modifier_num_lock: bool,
    pub modifier_scroll_lock: bool,
    pub modifier_super: bool,
    pub modifier_symbol: bool,
    pub modifier_symbol_lock: bool,
}
"#).unwrap();

    // Create IterableIteratorBinding.rs stub
    let iterable_iterator_binding = bindings_dir.join("IterableIteratorBinding.rs");
    fs::write(&iterable_iterator_binding, r#"
// Auto-generated IterableIteratorBinding stub for Boa engine
pub struct IterableIteratorMethods;
"#).unwrap();

    // Create InterfaceObjectMapPhf.rs with empty map
    let phf_rs = out_dir.join("InterfaceObjectMapPhf.rs");
    fs::write(&phf_rs, r#"
// Auto-generated for Boa engine
pub(crate) static MAP: phf::Map<&'static [u8], Interface> = phf::phf_map! {};
"#).unwrap();

    // Create InterfaceObjectMapData.json stub
    let json_path = out_dir.join("InterfaceObjectMapData.json");
    fs::write(&json_path, "{}").unwrap();

    // Create Globals.rs stub
    let globals_rs = out_dir.join("Globals.rs");
    fs::write(&globals_rs, r#"
// Auto-generated stub for Boa engine
// Window globals are handled by boa_bindings

use bitflags::bitflags;

bitflags! {
    /// Flags for global objects
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Globals: u32 {
        const EMPTY = 0;
        const WINDOW = 1 << 0;
        const WORKERGLOBALSCOPE = 1 << 1;
        const DEDICATEDWORKERGLOBALSCOPE = 1 << 2;
        const SERVICEWORKERGLOBALSCOPE = 1 << 3;
        const SHAREDWORKERGLOBALSCOPE = 1 << 4;
        const PAINTWORKLETGLOBALSCOPE = 1 << 5;
        const AUDIOWORKLETGLOBALSCOPE = 1 << 6;
        const TESTWORKLETGLOBALSCOPE = 1 << 7;
        const DISSIMILAR_ORIGIN_WINDOW = 1 << 8;
    }
}

/// Initialize window globals - stub for Boa
pub fn initialize_window(_global: &crate::reflector::Reflector) {
    // TODO: Initialize using boa_bindings
}
"#).unwrap();

    // Create InheritTypes.rs stub
    let inherit_types_rs = out_dir.join("InheritTypes.rs");
    fs::write(&inherit_types_rs, r#"
// Auto-generated stub for Boa engine
// Inheritance types for DOM elements

/// TopTypeId - top level type classification
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopTypeId {
    Node,
    Event,
    Other,
}

/// NodeTypeId - type IDs for Node types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeTypeId {
    CharacterData(CharacterDataTypeId),
    Document,
    DocumentFragment(DocumentFragmentTypeId),
    DocumentType,
    Element(ElementTypeId),
    Attr,
}

/// CharacterDataTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterDataTypeId {
    Text(TextTypeId),
    Comment,
    ProcessingInstruction,
}

/// TextTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextTypeId {
    Text,
    CDATASection,
}

/// DocumentFragmentTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentFragmentTypeId {
    DocumentFragment,
    ShadowRoot,
}

/// ElementTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementTypeId {
    HTMLElement(HTMLElementTypeId),
    SVGElement(SVGElementTypeId),
    MathMLElement,
    Element,
}

/// HTMLElementTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    HTMLDirectoryElement,
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
    HTMLMarqueeElement,
    HTMLMediaElement(HTMLMediaElementTypeId),
    HTMLMenuElement,
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
    HTMLTableCaptionElement,
    HTMLTableCellElement,
    HTMLTableColElement,
    HTMLTableElement,
    HTMLTableRowElement,
    HTMLTableSectionElement,
    HTMLTemplateElement,
    HTMLTextAreaElement,
    HTMLTimeElement,
    HTMLTitleElement,
    HTMLTrackElement,
    HTMLUListElement,
    HTMLUnknownElement,
    HTMLVideoElement,
}

/// HTMLMediaElementTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HTMLMediaElementTypeId {
    HTMLAudioElement,
    HTMLVideoElement,
}

/// SVGElementTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SVGElementTypeId {
    SVGElement,
    SVGGraphicsElement(SVGGraphicsElementTypeId),
    SVGSVGElement,
}

/// SVGGraphicsElementTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SVGGraphicsElementTypeId {
    SVGGraphicsElement,
}

/// EventTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventTypeId {
    Event,
    UIEvent(UIEventTypeId),
    CustomEvent,
    ErrorEvent,
    MessageEvent,
    ProgressEvent,
    CloseEvent,
}

/// UIEventTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UIEventTypeId {
    UIEvent,
    MouseEvent,
    KeyboardEvent,
    FocusEvent,
    WheelEvent,
    TouchEvent,
    CompositionEvent,
    InputEvent,
}

/// EventTargetTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventTargetTypeId {
    EventTarget,
    Node(NodeTypeId),
    Window,
    Worker,
    WorkerGlobalScope(WorkerGlobalScopeTypeId),
    XMLHttpRequest,
    XMLHttpRequestEventTarget,
    FileReader,
    MessagePort,
    DedicatedWorkerGlobalScope,
    ServiceWorkerGlobalScope,
    Performance,
}

/// WorkerGlobalScopeTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerGlobalScopeTypeId {
    DedicatedWorkerGlobalScope,
    ServiceWorkerGlobalScope,
}

/// GlobalScopeTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlobalScopeTypeId {
    Window,
    WorkerGlobalScope(WorkerGlobalScopeTypeId),
}
"#).unwrap();

    // Create PrototypeList.rs stub
    let prototype_list_rs = out_dir.join("PrototypeList.rs");
    fs::write(&prototype_list_rs, r#"
// Auto-generated stub for Boa engine

/// Maximum number of prototypes
pub const PROTO_OR_IFACE_LENGTH: usize = 500;

/// Maximum prototype chain length
pub const MAX_PROTO_CHAIN_LENGTH: usize = 10;

/// Prototype ID type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ID {
    Window = 0,
    Document = 1,
    Element = 2,
    Node = 3,
    EventTarget = 4,
    HTMLElement = 5,
    HTMLDivElement = 6,
    HTMLSpanElement = 7,
    HTMLParagraphElement = 8,
    HTMLAnchorElement = 9,
    // Add more as needed
    Last = 499,
}

impl ID {
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Convert prototype ID to string name
pub fn proto_id_to_name(id: ID) -> &'static str {
    match id {
        ID::Window => "Window",
        ID::Document => "Document",
        ID::Element => "Element",
        ID::Node => "Node",
        ID::EventTarget => "EventTarget",
        ID::HTMLElement => "HTMLElement",
        ID::HTMLDivElement => "HTMLDivElement",
        ID::HTMLSpanElement => "HTMLSpanElement",
        ID::HTMLParagraphElement => "HTMLParagraphElement",
        ID::HTMLAnchorElement => "HTMLAnchorElement",
        ID::Last => "Unknown",
    }
}
"#).unwrap();

    // Create DomTypes.rs stub with all required associated types
    let dom_types_rs = out_dir.join("DomTypes.rs");
    fs::write(&dom_types_rs, r#"
// Auto-generated stub for Boa engine

use crate::reflector::DomObject;

/// DomTypes marker trait with all required associated types for DOM interfaces
pub trait DomTypes: Sized + 'static {
    // Core DOM types
    type GlobalScope: DomObject;
    type Window: DomObject;
    type WindowProxy;
    type Document: DomObject;
    type DocumentFragment: DomObject;
    type DocumentType: DomObject;
    type Element: DomObject;
    type Node: DomObject;
    type Comment: DomObject;
    type Text: DomObject;
    type CharacterData: DomObject;
    type Attr: DomObject;
    type Event: DomObject;
    type EventTarget: DomObject;

    // HTML Element types
    type HTMLElement: DomObject;
    type HTMLFormElement: DomObject;
    type HTMLInputElement: DomObject;
    type HTMLButtonElement: DomObject;
    type HTMLSelectElement: DomObject;
    type HTMLTextAreaElement: DomObject;
    type HTMLAnchorElement: DomObject;
    type HTMLImageElement: DomObject;
    type HTMLScriptElement: DomObject;
    type HTMLStyleElement: DomObject;
    type HTMLLinkElement: DomObject;
    type HTMLDivElement: DomObject;
    type HTMLSpanElement: DomObject;
    type HTMLParagraphElement: DomObject;
    type HTMLHeadingElement: DomObject;
    type HTMLBodyElement: DomObject;
    type HTMLHtmlElement: DomObject;
    type HTMLHeadElement: DomObject;
    type HTMLTableElement: DomObject;
    type HTMLTableRowElement: DomObject;
    type HTMLTableCellElement: DomObject;
    type HTMLCanvasElement: DomObject;
    type HTMLVideoElement: DomObject;
    type HTMLAudioElement: DomObject;
    type HTMLMediaElement: DomObject;
    type HTMLIFrameElement: DomObject;
    type HTMLTemplateElement: DomObject;
    type HTMLSlotElement: DomObject;

    // Collections and lists
    type NodeList: DomObject;
    type HTMLCollection: DomObject;
    type HTMLOptionsCollection: DomObject;
    type HTMLFormControlsCollection: DomObject;
    type DOMTokenList: DomObject;
    type NamedNodeMap: DomObject;
    type FileList: DomObject;
    type File: DomObject;
    type Blob: DomObject;

    // CSS and styling
    type CSSStyleDeclaration: DomObject;
    type StyleSheet: DomObject;
    type CSSStyleSheet: DomObject;
    type CSSRule: DomObject;
    type CSSStyleRule: DomObject;

    // Ranges and selections
    type Range: DomObject;
    type Selection: DomObject;

    // Canvas and graphics
    type CanvasRenderingContext2D: DomObject;
    type WebGLRenderingContext: DomObject;
    type WebGL2RenderingContext: DomObject;
    type ImageData: DomObject;

    // URL and Location
    type URL: DomObject;
    type Location: DomObject;
    type History: DomObject;
    type Navigator: DomObject;

    // Storage
    type Storage: DomObject;

    // XMLHttpRequest and Fetch
    type XMLHttpRequest: DomObject;
    type Request: DomObject;
    type Response: DomObject;
    type Headers: DomObject;

    // Promises and async
    type Promise: DomObject;

    // Workers
    type Worker: DomObject;
    type WorkerGlobalScope: DomObject;
    type DedicatedWorkerGlobalScope: DomObject;
    type ServiceWorkerGlobalScope: DomObject;

    // Shadow DOM
    type ShadowRoot: DomObject;

    // Custom elements
    type CustomElementRegistry: DomObject;

    // Performance
    type Performance: DomObject;
    type PerformanceEntry: DomObject;

    // Other common types
    type DOMParser: DomObject;
    type XMLSerializer: DomObject;
    type TreeWalker: DomObject;
    type NodeIterator: DomObject;
    type MutationObserver: DomObject;
    type IntersectionObserver: DomObject;
    type ResizeObserver: DomObject;

    // Keyboard and input
    type KeyboardEvent: DomObject;
    type MouseEvent: DomObject;
    type TouchEvent: DomObject;
    type WheelEvent: DomObject;
    type PointerEvent: DomObject;
    type FocusEvent: DomObject;
    type InputEvent: DomObject;
    type UIEvent: DomObject;
    type CompositionEvent: DomObject;
    type ClipboardEvent: DomObject;
    type DragEvent: DomObject;

    // Animation
    type Animation: DomObject;
    type AnimationEvent: DomObject;
    type TransitionEvent: DomObject;

    // Error handling
    type DOMException: DomObject;
    type ErrorEvent: DomObject;

    // Message
    type MessageEvent: DomObject;
    type MessageChannel: DomObject;
    type MessagePort: DomObject;

    // AbortController
    type AbortController: DomObject;
    type AbortSignal: DomObject;
}
"#).unwrap();

    // Create GenericUnionTypes.rs stub
    let generic_union_types_rs = out_dir.join("GenericUnionTypes.rs");
    fs::write(&generic_union_types_rs, r#"
// Auto-generated stub for Boa engine
// Union types for DOM APIs

/// String or unsigned long union
pub enum StringOrUnsignedLong {
    String(String),
    UnsignedLong(u32),
}

/// Boolean or string union
pub enum BooleanOrString {
    Boolean(bool),
    String(String),
}

/// String or string sequence
pub enum StringOrStringSequence {
    String(String),
    StringSequence(Vec<String>),
}

/// Double or string union
pub enum DoubleOrString {
    Double(f64),
    String(String),
}

/// ArrayBuffer or string
pub enum ArrayBufferOrString {
    ArrayBuffer(Vec<u8>),
    String(String),
}

/// Blob or string
pub enum BlobOrString<D: crate::codegen::DomTypes::DomTypes> {
    Blob(std::marker::PhantomData<D>),
    String(String),
}

/// Node or string
pub enum NodeOrString<D: crate::codegen::DomTypes::DomTypes> {
    Node(std::marker::PhantomData<D>),
    String(String),
}

/// Element or string
pub enum ElementOrString<D: crate::codegen::DomTypes::DomTypes> {
    Element(std::marker::PhantomData<D>),
    String(String),
}
"#).unwrap();

    // Create RegisterBindings.rs stub  
    let register_bindings_rs = out_dir.join("RegisterBindings.rs");
    fs::write(&register_bindings_rs, r#"
// Auto-generated stub for Boa engine

/// Register all DOM bindings - stub for Boa
pub fn register<D: crate::codegen::DomTypes::DomTypes>() {
    // TODO: Register DOM constructors using boa_bindings
}
"#).unwrap();
}
