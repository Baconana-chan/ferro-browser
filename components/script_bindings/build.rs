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
        // For Boa, we still run codegen/run.py to generate proper trait definitions
        // (Methods traits, InheritTypes, UnionTypes, etc.) from WebIDL files.
        // Then we overwrite SpiderMonkey-specific files with Boa stubs.
        println!("cargo:warning=Building script_bindings for Boa - running WebIDL codegen then applying Boa stubs");
        
        let style_out_dir = PathBuf::from(env::var_os("DEP_SERVO_STYLE_CRATE_OUT_DIR").unwrap());
        let css_properties_json = style_out_dir.join("css-properties.json");
        
        println!("cargo::rerun-if-changed=webidls");
        println!("cargo::rerun-if-changed=codegen");
        println!("cargo::rerun-if-changed={}", css_properties_json.display());
        
        // Step 1: Run codegen/run.py to generate all bindings (traits, types, etc.)
        let status = find_python().and_then(|mut python| {
            python
                .arg("codegen/run.py")
                .arg(&css_properties_json)
                .arg(&out_dir)
                .status()
                .ok()
        });
        if !status.is_some_and(|status| status.success()) {
            println!("cargo:warning=WebIDL codegen failed, falling back to minimal Boa stubs");
            generate_boa_stubs(&out_dir);
        } else {
            println!("cargo:warning=WebIDL codegen succeeded, applying Boa-specific overrides");
            // Step 2: Override SpiderMonkey-specific generated files with Boa stubs
            apply_boa_overrides(&out_dir);
        }
        
        // Step 3: Generate PHF map from InterfaceObjectMapData.json
        let json_path = out_dir.join("InterfaceObjectMapData.json");
        if json_path.exists() {
            let json: Value = serde_json::from_reader(File::open(&json_path).unwrap()).unwrap();
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
            let phf = out_dir.join("InterfaceObjectMapPhf.rs");
            let mut phf = File::create(phf).unwrap();
            writeln!(
                &mut phf,
                "pub(crate) static MAP: phf::Map<&'static [u8], Interface> = {};",
                map.build(),
            )
            .unwrap();
        }
        
        println!("Boa binding generation completed in {:?}", start.elapsed());
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
        .expect("Can't find python interpreter for WebIDL codegen")
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
fn find_python() -> Option<Command> {
    let candidates = [
        ("uv", &["run", "--no-project", "python"][..]),
        ("python", &[][..]),
        ("py", &["-3"][..]),
    ];

    for (program, args) in candidates {
        let mut probe = Command::new(program);
        probe.args(args).arg("--version");
        if probe.output().is_ok_and(|out| out.status.success()) {
            let mut command = Command::new(program);
            command.args(args);
            return Some(command);
        }
    }

    None
}

/// Apply Boa-specific overrides on top of the codegen-generated files.
/// The codegen generates proper trait definitions from WebIDL, but some files
/// need to be overridden for Boa compatibility (e.g., DomTypeHolder, ConcreteBindings).
fn apply_boa_overrides(out_dir: &PathBuf) {
    use std::fs;
    use std::path::Path;

    fn patch_generated_js_paths(dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    patch_generated_js_paths(&path);
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                    continue;
                }
                let Ok(contents) = fs::read_to_string(&path) else {
                    continue;
                };
                let mut patched = contents.clone();
                patched = patched.replace("use js::", "use crate::js::");
                patched = patched.replace("(js::", "(crate::js::");
                patched = patched.replace(" js::", " crate::js::");
                patched = patched.replace(",js::", ",crate::js::");
                patched = patched.replace("[js::", "[crate::js::");
                patched = patched.replace("= js::", "= crate::js::");
                patched = patched.replace("rooted!(", "crate::js::rooted!(");
                patched = patched.replace("auto_root!(", "crate::js::auto_root!(");
                patched = patched.replace("rooted_vec!(", "crate::js::rooted_vec!(");
                patched = patched.replace(
                    "JSClassOps {\n        addProperty: None,\n        delProperty: None,\n        enumerate: None,",
                    "JSClassOps {\n        addProperty: None,\n        delProperty: None,\n        getProperty: None,\n        setProperty: None,\n        enumerate: None,",
                );
                patched = patched.replace(
                    "        finalize: None,\n        call: ",
                    "        finalize: None,\n        call: ",
                );
                patched = patched.replace(
                    "        call: None,\n        construct: None,",
                    "        call: None,\n        hasInstance: None,\n        construct: None,",
                );
                patched = patched.replace(
                    "        call: Some(invalid_constructor),\n        construct: Some(invalid_constructor),",
                    "        call: Some(invalid_constructor),\n        hasInstance: None,\n        construct: Some(invalid_constructor),",
                );
                patched = patched.replace(
                    "        call: Some(non_new_constructor),\n        construct: Some(hook),",
                    "        call: Some(non_new_constructor),\n        hasInstance: None,\n        construct: Some(hook),",
                );
                patched = patched.replace("selfHostedName", "self_hosted_name");
                patched = patched.replace(
                    "JSFunctionSpec {\n        name: JSPropertySpec_Name {",
                    "JSFunctionSpec {\n        name: JSFunctionSpec_Name {",
                );
                if patched != contents {
                    fs::write(&path, patched).unwrap();
                }
            }
        }
    }

    // Override GenericUnionTypes.rs to add Boa-specific union type stubs
    // The codegen generates union types that reference SpiderMonkey's JSVal,
    // so we need to override some that the script crate uses directly.
    // NOTE: We keep the codegen-generated GenericUnionTypes.rs as it already
    // has proper type definitions. We only extend it if needed.

    // Override ConcreteBindings to be empty stubs (SpiderMonkey-specific)
    let concrete_dir = out_dir.join("ConcreteBindings");
    fs::create_dir_all(&concrete_dir).unwrap();
    // ConcreteBindings/mod.rs - stub it since it's SpiderMonkey-specific
    fs::write(concrete_dir.join("mod.rs"), r#"
// Auto-generated stub for Boa engine
// Concrete bindings are SpiderMonkey-specific and not needed for Boa
"#).unwrap();

    // Override ConcreteInheritTypes.rs - stub it
    fs::write(out_dir.join("ConcreteInheritTypes.rs"), r#"
// Auto-generated stub for Boa engine  
// Concrete inherit types are SpiderMonkey-specific
"#).unwrap();

    // Override DomTypeHolder.rs with Boa-specific placeholder version
    // The codegen generates one that references SpiderMonkey-bound types
    // We leave the codegen version as-is since it should work with DomTypes trait
    // Actually, the DomTypeHolder needs concrete types which we don't have in Boa yet
    // So we stub it
    // NOTE: DomTypeHolder.rs from codegen should be fine as long as it only
    // declares the struct and impl DomTypes - keeping codegen version

    patch_generated_js_paths(out_dir);

    println!("cargo:warning=Boa overrides applied successfully");
}

/// Generate minimal stubs for Boa engine (no SpiderMonkey bindings needed)
fn generate_boa_stubs(out_dir: &PathBuf) {
    use std::fs;
    
    // Create Bindings directory
    let bindings_dir = out_dir.join("Bindings");
    fs::create_dir_all(&bindings_dir).unwrap();
    
    // List of all binding modules needed (auto-generated from error analysis)
    let binding_modules = [
        // Core DOM
        "AbortControllerBinding", "AbortSignalBinding", "AttrBinding",
        "CharacterDataBinding", "CommentBinding", "DocumentBinding",
        "DocumentFragmentBinding", "DocumentTypeBinding", "ElementBinding",
        "EventBinding", "EventTargetBinding", "NodeBinding", "NodeIteratorBinding",
        "NodeListBinding", "NamedNodeMapBinding", "ProcessingInstructionBinding",
        "RangeBinding", "SelectionBinding", "ShadowRootBinding", "TextBinding",
        "TreeWalkerBinding", "WindowBinding",
        
        // HTML Elements
        "HTMLAnchorElementBinding", "HTMLAreaElementBinding", "HTMLAudioElementBinding",
        "HTMLBaseElementBinding", "HTMLBodyElementBinding", "HTMLBRElementBinding",
        "HTMLButtonElementBinding", "HTMLCanvasElementBinding", "HTMLCollectionBinding",
        "HTMLDataElementBinding", "HTMLDataListElementBinding", "HTMLDetailsElementBinding",
        "HTMLDialogElementBinding", "HTMLDirectoryElementBinding", "HTMLDivElementBinding",
        "HTMLDListElementBinding", "HTMLElementBinding", "HTMLEmbedElementBinding",
        "HTMLFieldSetElementBinding", "HTMLFontElementBinding", "HTMLFormElementBinding",
        "HTMLFrameElementBinding", "HTMLFrameSetElementBinding", "HTMLHeadElementBinding",
        "HTMLHeadingElementBinding", "HTMLHRElementBinding", "HTMLHtmlElementBinding",
        "HTMLIFrameElementBinding", "HTMLImageElementBinding", "HTMLInputElementBinding",
        "HTMLLabelElementBinding", "HTMLLegendElementBinding", "HTMLLIElementBinding",
        "HTMLLinkElementBinding", "HTMLMapElementBinding", "HTMLMarqueeElementBinding",
        "HTMLMediaElementBinding", "HTMLMenuElementBinding", "HTMLMetaElementBinding",
        "HTMLMeterElementBinding", "HTMLModElementBinding", "HTMLObjectElementBinding",
        "HTMLOListElementBinding", "HTMLOptGroupElementBinding", "HTMLOptionElementBinding",
        "HTMLOutputElementBinding", "HTMLParagraphElementBinding", "HTMLParamElementBinding",
        "HTMLPictureElementBinding", "HTMLPreElementBinding", "HTMLProgressElementBinding",
        "HTMLQuoteElementBinding", "HTMLScriptElementBinding", "HTMLSelectElementBinding",
        "HTMLSlotElementBinding", "HTMLSourceElementBinding", "HTMLSpanElementBinding",
        "HTMLStyleElementBinding", "HTMLTableCaptionElementBinding", "HTMLTableCellElementBinding",
        "HTMLTableColElementBinding", "HTMLTableElementBinding", "HTMLTableRowElementBinding",
        "HTMLTableSectionElementBinding", "HTMLTemplateElementBinding", "HTMLTextAreaElementBinding",
        "HTMLTimeElementBinding", "HTMLTitleElementBinding", "HTMLTrackElementBinding",
        "HTMLUListElementBinding", "HTMLUnknownElementBinding", "HTMLVideoElementBinding",
        "HTMLDocumentBinding",
        
        // SVG and MathML
        "SVGElementBinding", "SVGSVGElementBinding", "SVGGraphicsElementBinding",
        "MathMLElementBinding",
        
        // DOM manipulation
        "DOMExceptionBinding", "DOMMatrixBinding", "DOMParserBinding",
        "DOMPointBinding", "DOMQuadBinding", "DOMRectBinding", "DOMTokenListBinding",
        "DOMStringListBinding", "DOMImplementationBinding",
        
        // CSS
        "CSSBinding", "CSSRuleBinding", "CSSRuleListBinding", "CSSStyleDeclarationBinding",
        "CSSStyleRuleBinding", "CSSStyleSheetBinding", "StyleSheetBinding", "StyleSheetListBinding",
        
        // Events
        "AnimationEventBinding", "BeforeUnloadEventBinding", "ClipboardEventBinding",
        "CompositionEventBinding", "CustomEventBinding", "DragEventBinding",
        "ErrorEventBinding", "FocusEventBinding", "HashChangeEventBinding",
        "InputEventBinding", "KeyboardEventBinding", "MessageEventBinding",
        "MouseEventBinding", "MutationEventBinding", "PageTransitionEventBinding",
        "PointerEventBinding", "PopStateEventBinding", "ProgressEventBinding",
        "StorageEventBinding", "TouchBinding", "TouchEventBinding", "TouchListBinding",
        "TransitionEventBinding", "UIEventBinding", "WheelEventBinding",
        
        // XHR and Fetch
        "XMLHttpRequestBinding", "XMLHttpRequestEventTargetBinding", "XMLHttpRequestUploadBinding",
        "FetchBinding", "RequestBinding", "ResponseBinding", "HeadersBinding", "FormDataBinding",
        
        // URL
        "URLBinding", "URLPatternBinding", "URLSearchParamsBinding",
        
        // File
        "BlobBinding", "FileBinding", "FileListBinding", "FileReaderBinding",
        
        // Location and History
        "HistoryBinding", "LocationBinding", "NavigatorBinding", "ScreenBinding",
        
        // Storage
        "StorageBinding",
        
        // Performance
        "PerformanceBinding", "PerformanceEntryBinding", "PerformanceTimingBinding",
        
        // Canvas and Graphics
        "CanvasGradientBinding", "CanvasPatternBinding", "CanvasRenderingContext2DBinding",
        "ImageDataBinding", "Path2DBinding", "TextMetricsBinding",
        "WebGL2RenderingContextBinding", "WebGLRenderingContextBinding",
        
        // Audio
        "AudioContextBinding", "AudioNodeBinding", "AnalyserNodeBinding",
        "GainNodeBinding", "OscillatorNodeBinding", "AudioBufferBinding",
        
        // Workers
        "DedicatedWorkerGlobalScopeBinding", "GlobalScopeBinding",
        "ServiceWorkerBinding", "ServiceWorkerContainerBinding",
        "ServiceWorkerGlobalScopeBinding", "ServiceWorkerRegistrationBinding",
        "SharedWorkerGlobalScopeBinding", "WorkerBinding", "WorkerGlobalScopeBinding",
        
        // Communication
        "BroadcastChannelBinding", "MessagePortBinding", "WebSocketBinding",
        
        // Crypto
        "CryptoBinding", "CryptoKeyBinding", "SubtleCryptoBinding",
        
        // Credentials
        "CredentialsContainerBinding", "PasswordCredentialBinding", "PublicKeyCredentialBinding",
        
        // Geolocation
        "GeolocationBinding", "GeolocationCoordinatesBinding",
        "GeolocationPositionBinding", "GeolocationPositionErrorBinding",
        
        // Media
        "MediaSourceBinding", "SourceBufferBinding", "SourceBufferListBinding",
        
        // IndexedDB
        "IDBCursorBinding", "IDBDatabaseBinding", "IDBIndexBinding",
        "IDBKeyRangeBinding", "IDBObjectStoreBinding", "IDBRequestBinding", "IDBTransactionBinding",
        
        // Observers
        "IntersectionObserverBinding", "MutationObserverBinding", "ResizeObserverBinding",
        
        // Misc
        "ClientBinding", "ClipboardBinding", "CloseEventBinding", "CommandEventBinding",
        "CompressionStreamBinding", "ConsoleBinding", "CookieStoreBinding", "CredentialBinding",
        "CSSConditionRuleBinding", "CSSGroupingRuleBinding", "CSSImportRuleBinding",
        "CSSKeyframeRuleBinding", "CSSKeyframesRuleBinding", "CSSLayerBlockRuleBinding",
        "CSSLayerStatementRuleBinding", "CSSMediaRuleBinding", "CSSNamespaceRuleBinding",
        "CSSNestedDeclarationsBinding", "CSSStyleValueBinding",
        "CustomElementRegistryBinding", "DataTransferBinding", "DataTransferItemBinding",
        "DataTransferItemListBinding", "DebuggerAddDebuggeeEventBinding",
        "DebuggerGetPossibleBreakpointsEventBinding", "DebuggerGlobalScopeBinding",
        "DecompressionStreamBinding", "DissimilarOriginLocationBinding",
        "DissimilarOriginWindowBinding", "DOMMatrixReadOnlyBinding", "DOMPointReadOnlyBinding",
        "DOMRectListBinding", "DOMRectReadOnlyBinding", "DOMStringMapBinding",
        "DynamicModuleOwnerBinding", "ElementInternalsBinding",
        "EventHandlerBinding", "EventListenerBinding", "EventModifierInitBinding",
        "EventSourceBinding", "ExtendableEventBinding", "ExtendableMessageEventBinding",
        "FetchLaterResultBinding", "FileReaderSyncBinding", "FontFaceBinding", "FontFaceSetBinding",
        "FormDataEventBinding", "FunctionBinding", "GamepadBinding", "GamepadButtonBinding",
        "GamepadButtonListBinding", "GamepadEventBinding", "GamepadHapticActuatorBinding",
        "GamepadPoseBinding", "GPUCanvasContextBinding",
        "HTMLFormControlsCollectionBinding", "HTMLOptionsCollectionBinding",
        "HTMLOrSVGElementBinding", "ImageBitmapBinding", "ImageBitmapRenderingContextBinding",
        "IDBCursorWithValueBinding", "IDBFactoryBinding", "IDBOpenDBRequestBinding",
        "IDBVersionChangeEventBinding", "IIRFilterNodeBinding",
        "IntersectionObserverEntryBinding", "IterableIteratorBinding",
        "LargestContentfulPaintBinding",
        "MediaDeviceInfoBinding", "MediaDevicesBinding", "MediaElementAudioSourceNodeBinding",
        "MediaErrorBinding", "MediaListBinding", "MediaMetadataBinding", "MediaQueryListBinding",
        "MediaQueryListEventBinding", "MediaSessionBinding",
        "MediaStreamAudioDestinationNodeBinding", "MediaStreamAudioSourceNodeBinding",
        "MediaStreamBinding", "MediaStreamTrackAudioSourceNodeBinding", "MediaStreamTrackBinding",
        "MessageChannelBinding", "MimeTypeArrayBinding", "MimeTypeBinding", "MutationRecordBinding",
        "NavigationPreloadManagerBinding", "NodeFilterBinding", "NotificationBinding",
        "OESStandardDerivativesBinding", "OESTextureHalfFloatBinding", "OESVertexArrayObjectBinding",
        "OfflineAudioCompletionEventBinding", "OfflineAudioContextBinding",
        "OffscreenCanvasBinding", "OffscreenCanvasRenderingContext2DBinding",
        "PaintRenderingContext2DBinding", "PaintSizeBinding", "PaintWorkletGlobalScopeBinding",
        "PannerNodeBinding", "PerformanceMarkBinding", "PerformanceMeasureBinding",
        "PerformanceNavigationBinding", "PerformanceNavigationTimingBinding",
        "PerformanceObserverBinding", "PerformanceObserverEntryListBinding",
        "PerformanceResourceTimingBinding", "PermissionsBinding", "PermissionStatusBinding",
        "PluginArrayBinding", "PluginBinding", "PromiseBinding", "PromiseRejectionEventBinding",
        "QueuingStrategyBinding", "QuotaExceededErrorBinding",
        "RadioNodeListBinding", "ReadableByteStreamControllerBinding",
        "ReadableStreamBinding", "ReadableStreamBYOBReaderBinding",
        "ReadableStreamBYOBRequestBinding", "ReadableStreamDefaultControllerBinding",
        "ReadableStreamDefaultReaderBinding", "ReportingObserverBinding",
        "ResizeObserverEntryBinding", "ResizeObserverSizeBinding",
        "RTCDataChannelBinding", "RTCDataChannelEventBinding", "RTCErrorBinding",
        "RTCErrorEventBinding", "RTCIceCandidateBinding", "RTCPeerConnectionBinding",
        "RTCPeerConnectionIceEventBinding", "RTCRtpSenderBinding", "RTCRtpTransceiverBinding",
        "RTCSessionDescriptionBinding", "RTCTrackEventBinding",
        "SecurityPolicyViolationEventBinding", "ServoInternalsBinding", "StaticRangeBinding",
        "StereoPannerNodeBinding", "StylePropertyMapReadOnlyBinding", "SubmitEventBinding",
        "TextDecoderBinding", "TextDecoderStreamBinding", "TextEncoderBinding",
        "TextEncoderStreamBinding", "TextTrackBinding", "TextTrackCueBinding",
        "TextTrackCueListBinding", "TextTrackListBinding", "TimeRangesBinding",
        "ToggleEventBinding", "TrackEventBinding", "TransformerBinding", "TransformStreamBinding",
        "TransformStreamDefaultControllerBinding",
        "TrustedHTMLBinding", "TrustedScriptBinding", "TrustedScriptURLBinding",
        "TrustedTypePolicyBinding", "TrustedTypePolicyFactoryBinding",
        "UnderlyingSinkBinding", "UnderlyingSourceBinding", "ValidityStateBinding",
        "VideoTrackBinding", "VideoTrackListBinding", "VisibilityStateEntryBinding",
        "VoidFunctionBinding", "VTTCueBinding", "VTTRegionBinding",
        "WebGLActiveInfoBinding", "WEBGLColorBufferFloatBinding", "WebGLContextEventBinding",
        "WebGLObjectBinding", "WebGLShaderPrecisionFormatBinding",
        "WorkerLocationBinding", "WorkerNavigatorBinding", "WorkletBinding",
        "WritableStreamBinding", "WritableStreamDefaultControllerBinding",
        "WritableStreamDefaultWriterBinding",
        "XMLDocumentBinding", "XMLSerializerBinding",
        "XPathEvaluatorBinding", "XPathNSResolverBinding", "XPathResultBinding",
        // Audio nodes
        "AbstractRangeBinding", "ANGLEInstancedArraysBinding",
        "AudioBufferSourceNodeBinding", "AudioDestinationNodeBinding", "AudioListenerBinding",
        "AudioParamBinding", "AudioScheduledSourceNodeBinding", "AudioTrackBinding",
        "AudioTrackListBinding", "BaseAudioContextBinding", "BiquadFilterNodeBinding",
        "ChannelMergerNodeBinding", "ChannelSplitterNodeBinding", "ConstantSourceNodeBinding",
        "CSPViolationReportBodyBinding",
        // WebGL extensions
        "EXTBlendMinmaxBinding", "EXTColorBufferHalfFloatBinding", "EXTTextureFilterAnisotropicBinding",
        // XPath
        "XPathExpressionBinding",
    ];
    
    // Create mod.rs for Bindings with all submodules
    let mut mod_content = String::from("// Auto-generated stub bindings for Boa engine\n\n");
    for module in &binding_modules {
        mod_content.push_str(&format!("pub mod {};\n", module));
    }
    let mod_rs = bindings_dir.join("mod.rs");
    fs::write(&mod_rs, &mod_content).unwrap();
    
    // Create stub file for each binding
    // Each binding module contains:
    // 1. A *Methods trait with a generic parameter D
    // 2. A nested *_Binding module (e.g. NodeBinding::Node_Binding) with re-exports
    // This matches the SpiderMonkey generated bindings structure
    for module in &binding_modules {
        let binding_file = bindings_dir.join(format!("{}.rs", module));
        let element_name = module.trim_end_matches("Binding");
        fs::write(&binding_file, format!(r#"
// Auto-generated stub for Boa engine: {}

use crate::codegen::DomTypes::DomTypes;

/// {} methods trait - stub for Boa engine
pub trait {}Methods<D: DomTypes> {{}}

/// Nested binding module for compatibility with SpiderMonkey-style imports
/// e.g. NodeBinding::Node_Binding::NodeMethods
pub mod {}_Binding {{
    use super::*;
    pub use super::{}Methods;
}}
"#, module, element_name, element_name, element_name, element_name)).unwrap();
    }

    // Create WindowBinding.rs stub with additional content
    let window_binding = bindings_dir.join("WindowBinding.rs");
    fs::write(&window_binding, r#"
// Auto-generated WindowBinding stub for Boa engine

pub struct WindowMethods;

/// Window binding constants
pub mod Window_Binding {
    pub const MAX_PROTO_CHAIN_LENGTH: usize = 3;
    
    pub trait WindowMethods {
        // Window method stubs - add as needed
    }
}
"#).unwrap();

    // Create EventModifierInitBinding.rs stub  
    let event_modifier_binding = bindings_dir.join("EventModifierInitBinding.rs");
    fs::write(&event_modifier_binding, r#"
// Auto-generated EventModifierInitBinding stub for Boa engine

use std::marker::PhantomData;

/// EventModifierInit dictionary
#[derive(Clone, Debug, Default)]
pub struct EventModifierInit<D> {
    pub altKey: bool,
    pub ctrlKey: bool,
    pub metaKey: bool,
    pub shiftKey: bool,
    pub keyModifierStateAltGraph: bool,
    pub keyModifierStateCapsLock: bool,
    pub keyModifierStateFn: bool,
    pub keyModifierStateFnLock: bool,
    pub keyModifierStateHyper: bool,
    pub keyModifierStateNumLock: bool,
    pub keyModifierStateScrollLock: bool,
    pub keyModifierStateSuper: bool,
    pub keyModifierStateSymbol: bool,
    pub keyModifierStateSymbolLock: bool,
    pub _marker: PhantomData<D>,
}
"#).unwrap();

    // Create IterableIteratorBinding.rs stub
    let iterable_iterator_binding = bindings_dir.join("IterableIteratorBinding.rs");
    fs::write(&iterable_iterator_binding, r#"
// Auto-generated IterableIteratorBinding stub for Boa engine

pub struct IterableIteratorMethods;

/// Iterable key or value result wrapper
#[derive(Clone, Debug)]
pub struct IterableKeyOrValueResult {
    pub done: bool,
    pub value: Option<crate::js::jsapi::Value>,
}

impl IterableKeyOrValueResult {
    pub fn empty() -> Self {
        Self::default()
    }
    
    pub fn to_jsval(&self, _cx: *mut crate::js::jsapi::JSContext, _rval: crate::js::rust::MutableHandleValue<'_>) {
        // Stub: convert to JSVal
    }
}

impl Default for IterableKeyOrValueResult {
    fn default() -> Self {
        Self { done: false, value: None }
    }
}

/// Iterable key and value result wrapper
#[derive(Clone, Debug)]
pub struct IterableKeyAndValueResult {
    pub done: bool,
    pub value: Option<(crate::js::jsapi::Value, crate::js::jsapi::Value)>,
}

impl IterableKeyAndValueResult {
    pub fn empty() -> Self {
        Self::default()
    }
    
    pub fn to_jsval(&self, _cx: *mut crate::js::jsapi::JSContext, _rval: crate::js::rust::MutableHandleValue<'_>) {
        // Stub: convert to JSVal
    }
}

impl Default for IterableKeyAndValueResult {
    fn default() -> Self {
        Self { done: false, value: None }
    }
}
"#).unwrap();

    // Create extended CanvasRenderingContext2DBinding with needed types
    let canvas_binding = bindings_dir.join("CanvasRenderingContext2DBinding.rs");
    fs::write(&canvas_binding, r#"
// Extended stub for CanvasRenderingContext2DBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

/// Canvas methods trait
pub trait CanvasRenderingContext2DMethods<D: DomTypes> {}

/// Nested binding module
pub mod CanvasRenderingContext2D_Binding {
    use super::*;
    pub use super::CanvasRenderingContext2DMethods;
}

// Enums
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasDirection { Inherit, Ltr, Rtl }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]  
pub enum CanvasFillRule { Nonzero, Evenodd }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasLineCap { Butt, Round, Square }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasLineJoin { Round, Bevel, Miter }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasTextAlign { Start, End, Left, Right, Center }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasTextBaseline { Top, Hanging, Middle, Alphabetic, Ideographic, Bottom }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredefinedColorSpace { Srgb, DisplayP3 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageDataPixelFormat { Rgba8 }

// Structs
#[derive(Clone, Debug, Default)]
pub struct ImageDataSettings {
    pub colorSpace: Option<PredefinedColorSpace>,
}

// Union stubs (as enums)
pub enum CanvasImageSource<D: DomTypes> {
    _Marker(PhantomData<D>),
}

// Additional Methods traits
pub trait CanvasGradientMethods<D: DomTypes> {}
pub trait CanvasPatternMethods<D: DomTypes> {}
pub trait Path2DMethods<D: DomTypes> {}
pub trait ImageDataMethods<D: DomTypes> {}
"#).unwrap();

    // Create DOMMatrixBinding with needed types
    let dom_matrix_binding = bindings_dir.join("DOMMatrixBinding.rs");
    fs::write(&dom_matrix_binding, r#"
// Extended stub for DOMMatrixBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DOMMatrixMethods<D: DomTypes> {}

pub mod DOMMatrix_Binding {
    use super::*;
    pub use super::DOMMatrixMethods;
}

#[derive(Clone, Debug, Default)]
pub struct DOMMatrix2DInit {
    pub a: Option<f64>,
    pub b: Option<f64>,
    pub c: Option<f64>,
    pub d: Option<f64>,
    pub e: Option<f64>,
    pub f: Option<f64>,
    pub m11: Option<f64>,
    pub m12: Option<f64>,
    pub m21: Option<f64>,
    pub m22: Option<f64>,
    pub m41: Option<f64>,
    pub m42: Option<f64>,
}
"#).unwrap();

    // Create FunctionBinding with Function type
    let function_binding = bindings_dir.join("FunctionBinding.rs");
    fs::write(&function_binding, r#"
// Extended stub for FunctionBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait FunctionMethods<D: DomTypes> {}

pub mod Function_Binding {
    use super::*;
    pub use super::FunctionMethods;
}

/// Callback function type
pub struct Function<D: DomTypes> {
    _marker: PhantomData<D>,
}

impl<D: DomTypes> Function<D> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }
}
"#).unwrap();

    // Create QueuingStrategyBinding
    let queuing_binding = bindings_dir.join("QueuingStrategyBinding.rs");
    fs::write(&queuing_binding, r#"
// Extended stub for QueuingStrategyBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ByteLengthQueuingStrategyMethods<D: DomTypes> {}
pub trait CountQueuingStrategyMethods<D: DomTypes> {}
pub trait QueuingStrategyMethods<D: DomTypes> {}

pub mod QueuingStrategy_Binding {
    use super::*;
    pub use super::QueuingStrategyMethods;
}

#[derive(Clone, Debug, Default)]
pub struct QueuingStrategyInit {
    pub highWaterMark: f64,
}

pub struct QueuingStrategy;
pub struct QueuingStrategySize;
"#).unwrap();

    // Create ClipboardBinding with needed types
    let clipboard_binding = bindings_dir.join("ClipboardBinding.rs");
    fs::write(&clipboard_binding, r#"
// Extended stub for ClipboardBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ClipboardMethods<D: DomTypes> {}
pub trait ClipboardItemMethods<D: DomTypes> {}

pub mod Clipboard_Binding {
    use super::*;
    pub use super::ClipboardMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationStyle { Unspecified, Inline, Attachment }

#[derive(Clone, Debug, Default)]
pub struct ClipboardItemOptions {
    pub presentationStyle: Option<PresentationStyle>,
}
"#).unwrap();

    // Create ClientBinding
    let client_binding = bindings_dir.join("ClientBinding.rs");
    fs::write(&client_binding, r#"
// Extended stub for ClientBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ClientMethods<D: DomTypes> {}

pub mod Client_Binding {
    use super::*;
    pub use super::ClientMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameType { Auxiliary, TopLevel, Nested, None }
"#).unwrap();

    // Create ClipboardEventBinding
    let clipboard_event_binding = bindings_dir.join("ClipboardEventBinding.rs");
    fs::write(&clipboard_event_binding, r#"
// Extended stub for ClipboardEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ClipboardEventMethods<D: DomTypes> {}

pub mod ClipboardEvent_Binding {
    use super::*;
    pub use super::ClipboardEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct ClipboardEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
}
"#).unwrap();

    // Create CloseEventBinding
    let close_event_binding = bindings_dir.join("CloseEventBinding.rs");
    fs::write(&close_event_binding, r#"
// Extended stub for CloseEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait CloseEventMethods<D: DomTypes> {}

pub mod CloseEvent_Binding {
    use super::*;
    pub use super::CloseEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct CloseEventInit {
    pub wasClean: bool,
    pub code: u16,
    pub reason: String,
}
"#).unwrap();

    // Create CompressionStreamBinding
    let compression_binding = bindings_dir.join("CompressionStreamBinding.rs");
    fs::write(&compression_binding, r#"
// Extended stub for CompressionStreamBinding

use crate::codegen::DomTypes::DomTypes;

pub trait CompressionStreamMethods<D: DomTypes> {}

pub mod CompressionStream_Binding {
    use super::*;
    pub use super::CompressionStreamMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompressionFormat { Deflate, DeflateRaw, Gzip }
"#).unwrap();

    // Create ConsoleBinding
    let console_binding = bindings_dir.join("ConsoleBinding.rs");
    fs::write(&console_binding, r#"
// Extended stub for ConsoleBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ConsoleMethods<D: DomTypes> {}
pub trait consoleMethods<D: DomTypes> {}

pub mod Console_Binding {
    use super::*;
    pub use super::ConsoleMethods;
}
"#).unwrap();

    // Create CryptoKeyBinding
    let crypto_key_binding = bindings_dir.join("CryptoKeyBinding.rs");
    fs::write(&crypto_key_binding, r#"
// Extended stub for CryptoKeyBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait CryptoKeyMethods<D: DomTypes> {}

pub mod CryptoKey_Binding {
    use super::*;
    pub use super::CryptoKeyMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyType { Public, Private, Secret }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyUsage { Encrypt, Decrypt, Sign, Verify, DeriveKey, DeriveBits, WrapKey, UnwrapKey }

pub struct CryptoKeyPair<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create CSSBinding
    let css_binding = bindings_dir.join("CSSBinding.rs");
    fs::write(&css_binding, r#"
// Extended stub for CSSBinding

use crate::codegen::DomTypes::DomTypes;

pub trait CSSMethods<D: DomTypes> {}

pub mod CSS_Binding {
    use super::*;
    pub use super::CSSMethods;
}

#[derive(Clone, Debug, Default)]
pub struct PropertyDefinition {
    pub name: String,
    pub syntax: Option<String>,
    pub inherits: bool,
    pub initialValue: Option<String>,
}
"#).unwrap();

    // Create CSSStyleSheetBinding
    let css_stylesheet_binding = bindings_dir.join("CSSStyleSheetBinding.rs");
    fs::write(&css_stylesheet_binding, r#"
// Extended stub for CSSStyleSheetBinding

use crate::codegen::DomTypes::DomTypes;

pub trait CSSStyleSheetMethods<D: DomTypes> {}

pub mod CSSStyleSheet_Binding {
    use super::*;
    pub use super::CSSStyleSheetMethods;
}

#[derive(Clone, Debug, Default)]
pub struct CSSStyleSheetInit {
    pub disabled: bool,
    pub baseURL: Option<String>,
}
"#).unwrap();

    // Create CredentialsContainerBinding
    let credentials_binding = bindings_dir.join("CredentialsContainerBinding.rs");
    fs::write(&credentials_binding, r#"
// Extended stub for CredentialsContainerBinding

use crate::codegen::DomTypes::DomTypes;

pub trait CredentialsContainerMethods<D: DomTypes> {}

pub mod CredentialsContainer_Binding {
    use super::*;
    pub use super::CredentialsContainerMethods;
}

#[derive(Clone, Debug, Default)]
pub struct CredentialCreationOptions;

#[derive(Clone, Debug, Default)]
pub struct CredentialRequestOptions;
"#).unwrap();

    // Create PasswordCredentialBinding
    let password_cred_binding = bindings_dir.join("PasswordCredentialBinding.rs");
    fs::write(&password_cred_binding, r#"
// Extended stub for PasswordCredentialBinding

use crate::codegen::DomTypes::DomTypes;

pub trait PasswordCredentialMethods<D: DomTypes> {}

pub mod PasswordCredential_Binding {
    use super::*;
    pub use super::PasswordCredentialMethods;
}

#[derive(Clone, Debug, Default)]
pub struct PasswordCredentialData {
    pub id: String,
    pub name: Option<String>,
    pub iconURL: Option<String>,
    pub password: String,
}
"#).unwrap();

    // Create ImageBitmapBinding
    let image_bitmap_binding = bindings_dir.join("ImageBitmapBinding.rs");
    fs::write(&image_bitmap_binding, r#"
// Extended stub for ImageBitmapBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait ImageBitmapMethods<D: DomTypes> {}

pub mod ImageBitmap_Binding {
    use super::*;
    pub use super::ImageBitmapMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageOrientation { FromImage, FlipY }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PremultiplyAlpha { Default, Premultiply, None }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeQuality { Pixelated, Low, Medium, High }

#[derive(Clone, Debug, Default)]
pub struct ImageBitmapOptions {
    pub imageOrientation: Option<ImageOrientation>,
    pub premultiplyAlpha: Option<PremultiplyAlpha>,
    pub resizeQuality: Option<ResizeQuality>,
    pub resizeWidth: Option<u32>,
    pub resizeHeight: Option<u32>,
}

pub enum ImageBitmapSource<D: DomTypes> {
    _Marker(PhantomData<D>),
}
"#).unwrap();

    // Create OffscreenCanvasBinding
    let offscreen_canvas_binding = bindings_dir.join("OffscreenCanvasBinding.rs");
    fs::write(&offscreen_canvas_binding, r#"
// Extended stub for OffscreenCanvasBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait OffscreenCanvasMethods<D: DomTypes> {}

pub mod OffscreenCanvas_Binding {
    use super::*;
    pub use super::OffscreenCanvasMethods;
}

#[derive(Clone, Debug, Default)]
pub struct ImageEncodeOptions {
    pub type_: Option<String>,
    pub quality: Option<f64>,
}

pub enum OffscreenRenderingContext<D: DomTypes> {
    _Marker(PhantomData<D>),
}
"#).unwrap();

    // Create CustomElementRegistryBinding
    let custom_element_binding = bindings_dir.join("CustomElementRegistryBinding.rs");
    fs::write(&custom_element_binding, r#"
// Extended stub for CustomElementRegistryBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait CustomElementRegistryMethods<D: DomTypes> {}

pub mod CustomElementRegistry_Binding {
    use super::*;
    pub use super::CustomElementRegistryMethods;
}

pub struct CustomElementConstructor<D: DomTypes> {
    _marker: PhantomData<D>,
}

#[derive(Clone, Debug, Default)]
pub struct ElementDefinitionOptions {
    pub extends: Option<String>,
}
"#).unwrap();

    // Create DataTransferItemBinding
    let data_transfer_item_binding = bindings_dir.join("DataTransferItemBinding.rs");
    fs::write(&data_transfer_item_binding, r#"
// Extended stub for DataTransferItemBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait DataTransferItemMethods<D: DomTypes> {}

pub mod DataTransferItem_Binding {
    use super::*;
    pub use super::DataTransferItemMethods;
}

pub struct FunctionStringCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create FontFaceBinding
    let font_face_binding = bindings_dir.join("FontFaceBinding.rs");
    fs::write(&font_face_binding, r#"
// Extended stub for FontFaceBinding

use crate::codegen::DomTypes::DomTypes;

pub trait FontFaceMethods<D: DomTypes> {}

pub mod FontFace_Binding {
    use super::*;
    pub use super::FontFaceMethods;
}

#[derive(Clone, Debug, Default)]
pub struct FontFaceDescriptors {
    pub style: Option<String>,
    pub weight: Option<String>,
    pub stretch: Option<String>,
    pub unicodeRange: Option<String>,
    pub variant: Option<String>,
    pub featureSettings: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontFaceLoadStatus { Unloaded, Loading, Loaded, Error }
"#).unwrap();

    // Create CookieStoreBinding
    let cookie_store_binding = bindings_dir.join("CookieStoreBinding.rs");
    fs::write(&cookie_store_binding, r#"
// Extended stub for CookieStoreBinding

use crate::codegen::DomTypes::DomTypes;

pub trait CookieStoreMethods<D: DomTypes> {}

pub mod CookieStore_Binding {
    use super::*;
    pub use super::CookieStoreMethods;
}

#[derive(Clone, Debug, Default)]
pub struct CookieInit {
    pub name: String,
    pub value: String,
    pub expires: Option<f64>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: Option<bool>,
    pub sameSite: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CookieListItem {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Default)]
pub struct CookieStoreDeleteOptions {
    pub name: String,
    pub domain: Option<String>,
    pub path: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CookieStoreGetOptions {
    pub name: Option<String>,
    pub url: Option<String>,
}
"#).unwrap();

    // Create ElementInternalsBinding
    let element_internals_binding = bindings_dir.join("ElementInternalsBinding.rs");
    fs::write(&element_internals_binding, r#"
// Extended stub for ElementInternalsBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ElementInternalsMethods<D: DomTypes> {}
pub trait CustomStateSetMethods<D: DomTypes> {}

pub mod ElementInternals_Binding {
    use super::*;
    pub use super::ElementInternalsMethods;
}
"#).unwrap();

    // Create DebuggerGlobalScopeBinding
    let debugger_binding = bindings_dir.join("DebuggerGlobalScopeBinding.rs");
    fs::write(&debugger_binding, r#"
// Extended stub for DebuggerGlobalScopeBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait DebuggerGlobalScopeMethods<D: DomTypes> {}

pub mod DebuggerGlobalScope_Binding {
    use super::*;
    pub use super::DebuggerGlobalScopeMethods;
}

pub struct NotifyNewSource<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create DebuggerAddDebuggeeEventBinding
    let debugger_add_debuggee_binding = bindings_dir.join("DebuggerAddDebuggeeEventBinding.rs");
    fs::write(&debugger_add_debuggee_binding, r#"
// Extended stub for DebuggerAddDebuggeeEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DebuggerAddDebuggeeEventMethods<D: DomTypes> {}

pub mod DebuggerAddDebuggeeEvent_Binding {
    use super::*;
    pub use super::DebuggerAddDebuggeeEventMethods;
}

/// PipelineIdMethods trait for pipeline identification
pub trait PipelineIdMethods<D: DomTypes> {
    fn namespace_id(&self) -> u32 { 0 }
    fn index(&self) -> u32 { 0 }
}
"#).unwrap();

    // Create DebuggerGetPossibleBreakpointsEventBinding
    let debugger_bp_binding = bindings_dir.join("DebuggerGetPossibleBreakpointsEventBinding.rs");
    fs::write(&debugger_bp_binding, r#"
// Extended stub for DebuggerGetPossibleBreakpointsEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DebuggerGetPossibleBreakpointsEventMethods<D: DomTypes> {}

pub mod DebuggerGetPossibleBreakpointsEvent_Binding {
    use super::*;
    pub use super::DebuggerGetPossibleBreakpointsEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct RecommendedBreakpointLocation {
    pub line: u32,
    pub column: u32,
}
"#).unwrap();

    // Create DOMParserBinding with SupportedType
    let dom_parser_binding = bindings_dir.join("DOMParserBinding.rs");
    fs::write(&dom_parser_binding, r#"
// Extended stub for DOMParserBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DOMParserMethods<D: DomTypes> {}

pub mod DOMParser_Binding {
    use super::*;
    pub use super::DOMParserMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportedType {
    Text_html,
    Text_xml,
    Application_xml,
    Application_xhtml_xml,
    Image_svg_xml,
}
"#).unwrap();

    // Create EventListenerBinding
    let event_listener_binding = bindings_dir.join("EventListenerBinding.rs");
    fs::write(&event_listener_binding, r#"
// Extended stub for EventListenerBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait EventListenerMethods<D: DomTypes> {}

pub mod EventListener_Binding {
    use super::*;
    pub use super::EventListenerMethods;
}

pub struct EventListener<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create EventHandlerBinding
    let event_handler_binding = bindings_dir.join("EventHandlerBinding.rs");
    fs::write(&event_handler_binding, r#"
// Extended stub for EventHandlerBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait EventHandlerMethods<D: DomTypes> {}

pub mod EventHandler_Binding {
    use super::*;
    pub use super::EventHandlerMethods;
}

pub struct EventHandlerNonNull<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create GamepadBinding
    let gamepad_binding = bindings_dir.join("GamepadBinding.rs");
    fs::write(&gamepad_binding, r#"
// Extended stub for GamepadBinding

use crate::codegen::DomTypes::DomTypes;

pub trait GamepadMethods<D: DomTypes> {}

pub mod Gamepad_Binding {
    use super::*;
    pub use super::GamepadMethods;
}
"#).unwrap();

    // Create NodeFilterBinding
    let node_filter_binding = bindings_dir.join("NodeFilterBinding.rs");
    fs::write(&node_filter_binding, r#"
// Extended stub for NodeFilterBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait NodeFilterMethods<D: DomTypes> {}

pub mod NodeFilter_Binding {
    use super::*;
    pub use super::NodeFilterMethods;
}

pub struct NodeFilter<D: DomTypes> {
    _marker: PhantomData<D>,
}

// Constants
pub const FILTER_ACCEPT: u16 = 1;
pub const FILTER_REJECT: u16 = 2;
pub const FILTER_SKIP: u16 = 3;
pub const SHOW_ALL: u32 = 0xFFFFFFFF;
pub const SHOW_ELEMENT: u32 = 0x1;
pub const SHOW_TEXT: u32 = 0x4;
pub const SHOW_COMMENT: u32 = 0x80;
pub const SHOW_DOCUMENT: u32 = 0x100;

/// NodeFilter constants module
pub mod NodeFilterConstants {
    pub const FILTER_ACCEPT: u16 = 1;
    pub const FILTER_REJECT: u16 = 2;
    pub const FILTER_SKIP: u16 = 3;
    pub const SHOW_ALL: u32 = 0xFFFFFFFF;
    pub const SHOW_ELEMENT: u32 = 0x1;
    pub const SHOW_ATTRIBUTE: u32 = 0x2;
    pub const SHOW_TEXT: u32 = 0x4;
    pub const SHOW_CDATA_SECTION: u32 = 0x8;
    pub const SHOW_ENTITY_REFERENCE: u32 = 0x10;
    pub const SHOW_ENTITY: u32 = 0x20;
    pub const SHOW_PROCESSING_INSTRUCTION: u32 = 0x40;
    pub const SHOW_COMMENT: u32 = 0x80;
    pub const SHOW_DOCUMENT: u32 = 0x100;
    pub const SHOW_DOCUMENT_TYPE: u32 = 0x200;
    pub const SHOW_DOCUMENT_FRAGMENT: u32 = 0x400;
    pub const SHOW_NOTATION: u32 = 0x800;
}
"#).unwrap();

    // Create extended DocumentBinding with needed types
    let document_binding = bindings_dir.join("DocumentBinding.rs");
    fs::write(&document_binding, r#"
// Extended stub for DocumentBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DocumentMethods<D: DomTypes> {}

pub mod Document_Binding {
    use super::*;
    pub use super::DocumentMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentReadyState { Loading, Interactive, Complete }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentVisibilityState { Visible, Hidden }

#[derive(Clone, Debug)]
pub enum NamedPropertyValue {
    Element(()),
    HTMLCollection(()),
}

#[derive(Clone, Debug, Default)]
pub struct ElementCreationOptions {
    pub is: Option<String>,
}
"#).unwrap();

    // Create extended WindowBinding with needed types
    let window_binding_ext = bindings_dir.join("WindowBinding.rs");
    fs::write(&window_binding_ext, r#"
// Extended stub for WindowBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait WindowMethods<D: DomTypes> {}

pub mod Window_Binding {
    use super::*;
    pub use super::WindowMethods;
    pub const MAX_PROTO_CHAIN_LENGTH: usize = 3;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollBehavior { Auto, Smooth, Instant }

#[derive(Clone, Debug, Default)]
pub struct ScrollToOptions {
    pub top: Option<f64>,
    pub left: Option<f64>,
    pub behavior: Option<ScrollBehavior>,
}

#[derive(Clone, Debug, Default)]
pub struct ScrollOptions {
    pub behavior: Option<ScrollBehavior>,
}

#[derive(Clone, Debug, Default)]
pub struct WindowPostMessageOptions {
    pub targetOrigin: String,
    pub transfer: Vec<()>,
}

#[derive(Clone, Debug, Default)]
pub struct DeferredRequestInit;

pub struct FrameRequestCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create extended ElementBinding with needed types
    let element_binding = bindings_dir.join("ElementBinding.rs");
    fs::write(&element_binding, r#"
// Extended stub for ElementBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ElementMethods<D: DomTypes> {}

pub mod Element_Binding {
    use super::*;
    pub use super::ElementMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollLogicalPosition { Start, Center, End, Nearest }

#[derive(Clone, Debug, Default)]
pub struct ScrollIntoViewOptions {
    pub behavior: Option<super::WindowBinding::ScrollBehavior>,
    pub block: Option<ScrollLogicalPosition>,
    pub inline: Option<ScrollLogicalPosition>,
}

pub enum ScrollIntoViewContainer {
    Boolean(bool),
    ScrollIntoViewOptions(ScrollIntoViewOptions),
}

#[derive(Clone, Debug, Default)]
pub struct ShadowRootInit {
    pub mode: Option<super::ShadowRootBinding::ShadowRootMode>,
    pub delegatesFocus: Option<bool>,
    pub slotAssignment: Option<super::ShadowRootBinding::SlotAssignmentMode>,
}

#[derive(Clone, Debug, Default)]
pub struct GetHTMLOptions {
    pub serializableShadowRoots: Option<bool>,
    pub shadowRoots: Option<Vec<()>>,
}
"#).unwrap();

    // Create extended ShadowRootBinding with needed types
    let shadow_root_binding = bindings_dir.join("ShadowRootBinding.rs");
    fs::write(&shadow_root_binding, r#"
// Extended stub for ShadowRootBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ShadowRootMethods<D: DomTypes> {}

pub mod ShadowRoot_Binding {
    use super::*;
    pub use super::ShadowRootMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowRootMode { Open, Closed }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotAssignmentMode { Manual, Named }
"#).unwrap();

    // Create extended EventBinding with needed types
    let event_binding = bindings_dir.join("EventBinding.rs");
    fs::write(&event_binding, r#"
// Extended stub for EventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait EventMethods<D: DomTypes> {}

pub mod Event_Binding {
    use super::*;
    pub use super::EventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct EventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
}

// Event constants
pub struct EventConstants;
impl EventConstants {
    pub const NONE: u16 = 0;
    pub const CAPTURING_PHASE: u16 = 1;
    pub const AT_TARGET: u16 = 2;
    pub const BUBBLING_PHASE: u16 = 3;
}
"#).unwrap();

    // Create extended EventTargetBinding with needed types
    let event_target_binding = bindings_dir.join("EventTargetBinding.rs");
    fs::write(&event_target_binding, r#"
// Extended stub for EventTargetBinding

use crate::codegen::DomTypes::DomTypes;

pub trait EventTargetMethods<D: DomTypes> {}

pub mod EventTarget_Binding {
    use super::*;
    pub use super::EventTargetMethods;
}

#[derive(Clone, Debug, Default)]
pub struct EventListenerOptions {
    pub capture: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AddEventListenerOptions {
    pub capture: bool,
    pub passive: Option<bool>,
    pub once: bool,
    pub signal: Option<()>,
}
"#).unwrap();

    // Create extended NodeBinding with needed types
    let node_binding = bindings_dir.join("NodeBinding.rs");
    fs::write(&node_binding, r#"
// Extended stub for NodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait NodeMethods<D: DomTypes> {}

pub mod Node_Binding {
    use super::*;
    pub use super::NodeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct GetRootNodeOptions {
    pub composed: bool,
}

// Node constants
pub struct NodeConstants;
impl NodeConstants {
    pub const ELEMENT_NODE: u16 = 1;
    pub const ATTRIBUTE_NODE: u16 = 2;
    pub const TEXT_NODE: u16 = 3;
    pub const CDATA_SECTION_NODE: u16 = 4;
    pub const ENTITY_REFERENCE_NODE: u16 = 5;
    pub const ENTITY_NODE: u16 = 6;
    pub const PROCESSING_INSTRUCTION_NODE: u16 = 7;
    pub const COMMENT_NODE: u16 = 8;
    pub const DOCUMENT_NODE: u16 = 9;
    pub const DOCUMENT_TYPE_NODE: u16 = 10;
    pub const DOCUMENT_FRAGMENT_NODE: u16 = 11;
    pub const NOTATION_NODE: u16 = 12;
    pub const DOCUMENT_POSITION_DISCONNECTED: u16 = 0x01;
    pub const DOCUMENT_POSITION_PRECEDING: u16 = 0x02;
    pub const DOCUMENT_POSITION_FOLLOWING: u16 = 0x04;
    pub const DOCUMENT_POSITION_CONTAINS: u16 = 0x08;
    pub const DOCUMENT_POSITION_CONTAINED_BY: u16 = 0x10;
    pub const DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: u16 = 0x20;
}
"#).unwrap();

    // Create extended EventHandlerBinding with callbacks
    let event_handler_binding_ext = bindings_dir.join("EventHandlerBinding.rs");
    fs::write(&event_handler_binding_ext, r#"
// Extended stub for EventHandlerBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait EventHandlerMethods<D: DomTypes> {}

pub mod EventHandler_Binding {
    use super::*;
    pub use super::EventHandlerMethods;
}

pub struct EventHandlerNonNull<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct OnBeforeUnloadEventHandlerNonNull<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct OnErrorEventHandlerNonNull<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create extended PerformanceBinding with types
    let performance_binding = bindings_dir.join("PerformanceBinding.rs");
    fs::write(&performance_binding, r#"
// Extended stub for PerformanceBinding

use crate::codegen::DomTypes::DomTypes;

pub trait PerformanceMethods<D: DomTypes> {}

pub mod Performance_Binding {
    use super::*;
    pub use super::PerformanceMethods;
}

pub type DOMHighResTimeStamp = f64;
pub type PerformanceEntryList = Vec<()>;
"#).unwrap();

    // Create extended HTMLOrSVGElementBinding with types
    let html_or_svg_binding = bindings_dir.join("HTMLOrSVGElementBinding.rs");
    fs::write(&html_or_svg_binding, r#"
// Extended stub for HTMLOrSVGElementBinding

use crate::codegen::DomTypes::DomTypes;

pub trait HTMLOrSVGElementMethods<D: DomTypes> {}

pub mod HTMLOrSVGElement_Binding {
    use super::*;
    pub use super::HTMLOrSVGElementMethods;
}

#[derive(Clone, Debug, Default)]
pub struct FocusOptions {
    pub preventScroll: bool,
}
"#).unwrap();

    // Create extended PermissionStatusBinding with types
    let permission_status_binding = bindings_dir.join("PermissionStatusBinding.rs");
    fs::write(&permission_status_binding, r#"
// Extended stub for PermissionStatusBinding

use crate::codegen::DomTypes::DomTypes;

pub trait PermissionStatusMethods<D: DomTypes> {}

pub mod PermissionStatus_Binding {
    use super::*;
    pub use super::PermissionStatusMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionState {
    Granted,
    Denied,
    Prompt,
}

impl Default for PermissionState {
    fn default() -> Self {
        PermissionState::Prompt
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionName {
    Geolocation,
    Notifications,
    Push,
    Midi,
    Camera,
    Microphone,
    BackgroundSync,
    AmbientLightSensor,
    Accelerometer,
    Gyroscope,
    Magnetometer,
    ClipboardRead,
    ClipboardWrite,
    ScreenWakeLock,
    Nfc,
    DisplayCapture,
    Bluetooth,
    PersistentStorage,
}

#[derive(Clone, Debug, Default)]
pub struct PermissionDescriptor {
    pub name: Option<PermissionName>,
}
"#).unwrap();

    // Create extended XPathNSResolverBinding
    let xpath_resolver_binding = bindings_dir.join("XPathNSResolverBinding.rs");
    fs::write(&xpath_resolver_binding, r#"
// Extended stub for XPathNSResolverBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait XPathNSResolverMethods<D: DomTypes> {}

pub mod XPathNSResolver_Binding {
    use super::*;
    pub use super::XPathNSResolverMethods;
}

pub struct XPathNSResolver<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create extended DOMExceptionBinding with constants
    let dom_exception_binding = bindings_dir.join("DOMExceptionBinding.rs");
    fs::write(&dom_exception_binding, r#"
// Extended stub for DOMExceptionBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DOMExceptionMethods<D: DomTypes> {}

pub mod DOMException_Binding {
    use super::*;
    pub use super::DOMExceptionMethods;
}

pub struct DOMExceptionConstants;
impl DOMExceptionConstants {
    pub const INDEX_SIZE_ERR: u16 = 1;
    pub const DOMSTRING_SIZE_ERR: u16 = 2;
    pub const HIERARCHY_REQUEST_ERR: u16 = 3;
    pub const WRONG_DOCUMENT_ERR: u16 = 4;
    pub const INVALID_CHARACTER_ERR: u16 = 5;
    pub const NO_DATA_ALLOWED_ERR: u16 = 6;
    pub const NO_MODIFICATION_ALLOWED_ERR: u16 = 7;
    pub const NOT_FOUND_ERR: u16 = 8;
    pub const NOT_SUPPORTED_ERR: u16 = 9;
    pub const INUSE_ATTRIBUTE_ERR: u16 = 10;
    pub const INVALID_STATE_ERR: u16 = 11;
    pub const SYNTAX_ERR: u16 = 12;
    pub const INVALID_MODIFICATION_ERR: u16 = 13;
    pub const NAMESPACE_ERR: u16 = 14;
    pub const INVALID_ACCESS_ERR: u16 = 15;
    pub const VALIDATION_ERR: u16 = 16;
    pub const TYPE_MISMATCH_ERR: u16 = 17;
    pub const SECURITY_ERR: u16 = 18;
    pub const NETWORK_ERR: u16 = 19;
    pub const ABORT_ERR: u16 = 20;
    pub const URL_MISMATCH_ERR: u16 = 21;
    pub const QUOTA_EXCEEDED_ERR: u16 = 22;
    pub const TIMEOUT_ERR: u16 = 23;
    pub const INVALID_NODE_TYPE_ERR: u16 = 24;
    pub const DATA_CLONE_ERR: u16 = 25;
}
"#).unwrap();

    // Create extended RangeBinding with constants
    let range_binding = bindings_dir.join("RangeBinding.rs");
    fs::write(&range_binding, r#"
// Extended stub for RangeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RangeMethods<D: DomTypes> {}

pub mod Range_Binding {
    use super::*;
    pub use super::RangeMethods;
}

pub struct RangeConstants;
impl RangeConstants {
    pub const START_TO_START: u16 = 0;
    pub const START_TO_END: u16 = 1;
    pub const END_TO_END: u16 = 2;
    pub const END_TO_START: u16 = 3;
}
"#).unwrap();

    // Create extended DOMPointBinding with types
    let dom_point_binding = bindings_dir.join("DOMPointBinding.rs");
    fs::write(&dom_point_binding, r#"
// Extended stub for DOMPointBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DOMPointMethods<D: DomTypes> {}

pub mod DOMPoint_Binding {
    use super::*;
    pub use super::DOMPointMethods;
}

#[derive(Clone, Debug, Default)]
pub struct DOMPointInit {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}
"#).unwrap();

    // Create extended DOMQuadBinding with types
    let dom_quad_binding = bindings_dir.join("DOMQuadBinding.rs");
    fs::write(&dom_quad_binding, r#"
// Extended stub for DOMQuadBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DOMQuadMethods<D: DomTypes> {}

pub mod DOMQuad_Binding {
    use super::*;
    pub use super::DOMQuadMethods;
}

#[derive(Clone, Debug, Default)]
pub struct DOMQuadInit {
    pub p1: Option<super::DOMPointBinding::DOMPointInit>,
    pub p2: Option<super::DOMPointBinding::DOMPointInit>,
    pub p3: Option<super::DOMPointBinding::DOMPointInit>,
    pub p4: Option<super::DOMPointBinding::DOMPointInit>,
}
"#).unwrap();

    // Create extended DOMRectReadOnlyBinding with types
    let dom_rect_readonly_binding = bindings_dir.join("DOMRectReadOnlyBinding.rs");
    fs::write(&dom_rect_readonly_binding, r#"
// Extended stub for DOMRectReadOnlyBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DOMRectReadOnlyMethods<D: DomTypes> {}

pub mod DOMRectReadOnly_Binding {
    use super::*;
    pub use super::DOMRectReadOnlyMethods;
}

#[derive(Clone, Debug, Default)]
pub struct DOMRectInit {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
"#).unwrap();

    // Create extended DOMMatrixBinding with full types
    let dom_matrix_binding_ext = bindings_dir.join("DOMMatrixBinding.rs");
    fs::write(&dom_matrix_binding_ext, r#"
// Extended stub for DOMMatrixBinding

use crate::codegen::DomTypes::DomTypes;

pub trait DOMMatrixMethods<D: DomTypes> {}

pub mod DOMMatrix_Binding {
    use super::*;
    pub use super::DOMMatrixMethods;
}

#[derive(Clone, Debug, Default)]
pub struct DOMMatrix2DInit {
    pub a: Option<f64>,
    pub b: Option<f64>,
    pub c: Option<f64>,
    pub d: Option<f64>,
    pub e: Option<f64>,
    pub f: Option<f64>,
    pub m11: Option<f64>,
    pub m12: Option<f64>,
    pub m21: Option<f64>,
    pub m22: Option<f64>,
    pub m41: Option<f64>,
    pub m42: Option<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct DOMMatrixInit {
    pub a: Option<f64>,
    pub b: Option<f64>,
    pub c: Option<f64>,
    pub d: Option<f64>,
    pub e: Option<f64>,
    pub f: Option<f64>,
    pub m11: Option<f64>,
    pub m12: Option<f64>,
    pub m13: Option<f64>,
    pub m14: Option<f64>,
    pub m21: Option<f64>,
    pub m22: Option<f64>,
    pub m23: Option<f64>,
    pub m24: Option<f64>,
    pub m31: Option<f64>,
    pub m32: Option<f64>,
    pub m33: Option<f64>,
    pub m34: Option<f64>,
    pub m41: Option<f64>,
    pub m42: Option<f64>,
    pub m43: Option<f64>,
    pub m44: Option<f64>,
    pub is2D: Option<bool>,
}
"#).unwrap();

    // Create extended FileReaderBinding with constants
    let file_reader_binding = bindings_dir.join("FileReaderBinding.rs");
    fs::write(&file_reader_binding, r#"
// Extended stub for FileReaderBinding

use crate::codegen::DomTypes::DomTypes;

pub trait FileReaderMethods<D: DomTypes> {}

pub mod FileReader_Binding {
    use super::*;
    pub use super::FileReaderMethods;
}

pub struct FileReaderConstants;
impl FileReaderConstants {
    pub const EMPTY: u16 = 0;
    pub const LOADING: u16 = 1;
    pub const DONE: u16 = 2;
}
"#).unwrap();

    // Create extended XMLHttpRequestBinding with types
    let xhr_binding = bindings_dir.join("XMLHttpRequestBinding.rs");
    fs::write(&xhr_binding, r#"
// Extended stub for XMLHttpRequestBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait XMLHttpRequestMethods<D: DomTypes> {}

pub mod XMLHttpRequest_Binding {
    use super::*;
    pub use super::XMLHttpRequestMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XMLHttpRequestResponseType {
    Empty,
    Arraybuffer,
    Blob,
    Document,
    Json,
    Text,
}

pub enum BodyInit<D: DomTypes> {
    Blob(PhantomData<D>),
    FormData(PhantomData<D>),
    URLSearchParams(PhantomData<D>),
    String(String),
    ArrayBuffer(Vec<u8>),
    ArrayBufferView(Vec<u8>),
}
"#).unwrap();

    // Create extended HeadersBinding with types
    let headers_binding = bindings_dir.join("HeadersBinding.rs");
    fs::write(&headers_binding, r#"
// Extended stub for HeadersBinding

use crate::codegen::DomTypes::DomTypes;

pub trait HeadersMethods<D: DomTypes> {}

pub mod Headers_Binding {
    use super::*;
    pub use super::HeadersMethods;
}

pub enum HeadersInit {
    Sequence(Vec<(String, String)>),
    Record(Vec<(String, String)>),
}
"#).unwrap();

    // Create extended RequestBinding with types
    let request_binding = bindings_dir.join("RequestBinding.rs");
    fs::write(&request_binding, r#"
// Extended stub for RequestBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait RequestMethods<D: DomTypes> {}

pub mod Request_Binding {
    use super::*;
    pub use super::RequestMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestCredentials { Omit, SameOrigin, Include }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferrerPolicy {
    Empty,
    NoReferrer,
    NoReferrerWhenDowngrade,
    SameOrigin,
    Origin,
    StrictOrigin,
    OriginWhenCrossOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestCache { Default, NoStore, Reload, NoCache, ForceCache, OnlyIfCached }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestDestination {
    Empty, Audio, Audioworklet, Document, Embed, Font, Frame, Iframe, Image, Json,
    Manifest, Object, Paintworklet, Report, Script, Serviceworker, Sharedworker,
    Style, Track, Video, Webidentity, Worker, Xslt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestMode { Navigate, SameOrigin, NoCors, Cors }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestRedirect { Follow, Error, Manual }

#[derive(Clone, Debug, Default)]
pub struct RequestInit {
    pub method: Option<String>,
    pub headers: Option<()>,
    pub body: Option<()>,
    pub referrer: Option<String>,
    pub referrerPolicy: Option<ReferrerPolicy>,
    pub mode: Option<()>,
    pub credentials: Option<RequestCredentials>,
    pub cache: Option<()>,
    pub redirect: Option<()>,
    pub integrity: Option<String>,
    pub keepalive: Option<bool>,
    pub signal: Option<()>,
}

pub enum RequestInfo<D: DomTypes> {
    Request(PhantomData<D>),
    USVString(String),
}
"#).unwrap();

    // Create extended ResponseBinding with types
    let response_binding = bindings_dir.join("ResponseBinding.rs");
    fs::write(&response_binding, r#"
// Extended stub for ResponseBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ResponseMethods<D: DomTypes> {}

pub mod Response_Binding {
    use super::*;
    pub use super::ResponseMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseType { Basic, Cors, Default, Error, Opaque, OpaqueRedirect }
"#).unwrap();

    // Create extended HTMLMediaElementBinding with types
    let html_media_binding = bindings_dir.join("HTMLMediaElementBinding.rs");
    fs::write(&html_media_binding, r#"
// Extended stub for HTMLMediaElementBinding

use crate::codegen::DomTypes::DomTypes;

pub trait HTMLMediaElementMethods<D: DomTypes> {}

pub mod HTMLMediaElement_Binding {
    use super::*;
    pub use super::HTMLMediaElementMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanPlayTypeResult { Empty, Maybe, Probably }

/// Constants for HTMLMediaElement
pub mod HTMLMediaElementConstants {
    pub const NETWORK_EMPTY: u16 = 0;
    pub const NETWORK_IDLE: u16 = 1;
    pub const NETWORK_LOADING: u16 = 2;
    pub const NETWORK_NO_SOURCE: u16 = 3;
    pub const HAVE_NOTHING: u16 = 0;
    pub const HAVE_METADATA: u16 = 1;
    pub const HAVE_CURRENT_DATA: u16 = 2;
    pub const HAVE_FUTURE_DATA: u16 = 3;
    pub const HAVE_ENOUGH_DATA: u16 = 4;
}
"#).unwrap();

    // Create extended HTMLFormElementBinding with types
    let html_form_binding = bindings_dir.join("HTMLFormElementBinding.rs");
    fs::write(&html_form_binding, r#"
// Extended stub for HTMLFormElementBinding

use crate::codegen::DomTypes::DomTypes;

pub trait HTMLFormElementMethods<D: DomTypes> {}

pub mod HTMLFormElement_Binding {
    use super::*;
    pub use super::HTMLFormElementMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMode { Select, Start, End, Preserve }
"#).unwrap();

    // Create extended HTMLSlotElementBinding with types
    let html_slot_binding = bindings_dir.join("HTMLSlotElementBinding.rs");
    fs::write(&html_slot_binding, r#"
// Extended stub for HTMLSlotElementBinding

use crate::codegen::DomTypes::DomTypes;

pub trait HTMLSlotElementMethods<D: DomTypes> {}

pub mod HTMLSlotElement_Binding {
    use super::*;
    pub use super::HTMLSlotElementMethods;
}

#[derive(Clone, Debug, Default)]
pub struct AssignedNodesOptions {
    pub flatten: bool,
}
"#).unwrap();

    // Create extended HTMLTrackElementBinding with constants
    let html_track_binding = bindings_dir.join("HTMLTrackElementBinding.rs");
    fs::write(&html_track_binding, r#"
// Extended stub for HTMLTrackElementBinding

use crate::codegen::DomTypes::DomTypes;

pub trait HTMLTrackElementMethods<D: DomTypes> {}

pub mod HTMLTrackElement_Binding {
    use super::*;
    pub use super::HTMLTrackElementMethods;
}

pub struct HTMLTrackElementConstants;
impl HTMLTrackElementConstants {
    pub const NONE: u16 = 0;
    pub const LOADING: u16 = 1;
    pub const LOADED: u16 = 2;
    pub const ERROR: u16 = 3;
}
"#).unwrap();

    // Create extended HTMLCanvasElementBinding with callback
    let html_canvas_binding = bindings_dir.join("HTMLCanvasElementBinding.rs");
    fs::write(&html_canvas_binding, r#"
// Extended stub for HTMLCanvasElementBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait HTMLCanvasElementMethods<D: DomTypes> {}

pub mod HTMLCanvasElement_Binding {
    use super::*;
    pub use super::HTMLCanvasElementMethods;
}

pub struct BlobCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

/// RenderingContext union type for canvas
#[derive(Clone)]
pub enum RenderingContext<D: DomTypes> {
    CanvasRenderingContext2D(PhantomData<D>),
    WebGLRenderingContext(PhantomData<D>),
    WebGL2RenderingContext(PhantomData<D>),
}
"#).unwrap();

    // Create extended ElementInternalsBinding with ValidityStateFlags
    let element_internals_binding_ext = bindings_dir.join("ElementInternalsBinding.rs");
    fs::write(&element_internals_binding_ext, r#"
// Extended stub for ElementInternalsBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ElementInternalsMethods<D: DomTypes> {}
pub trait CustomStateSetMethods<D: DomTypes> {}

pub mod ElementInternals_Binding {
    use super::*;
    pub use super::ElementInternalsMethods;
}

#[derive(Clone, Debug, Default)]
pub struct ValidityStateFlags {
    pub valueMissing: bool,
    pub typeMismatch: bool,
    pub patternMismatch: bool,
    pub tooLong: bool,
    pub tooShort: bool,
    pub rangeUnderflow: bool,
    pub rangeOverflow: bool,
    pub stepMismatch: bool,
    pub badInput: bool,
    pub customError: bool,
}
"#).unwrap();

    // Create extended GamepadBinding with types
    let gamepad_binding_ext = bindings_dir.join("GamepadBinding.rs");
    fs::write(&gamepad_binding_ext, r#"
// Extended stub for GamepadBinding

use crate::codegen::DomTypes::DomTypes;

pub trait GamepadMethods<D: DomTypes> {}

pub mod Gamepad_Binding {
    use super::*;
    pub use super::GamepadMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadHand { Empty, Left, Right }
"#).unwrap();

    // Create extended GamepadHapticActuatorBinding with types
    let gamepad_haptic_binding = bindings_dir.join("GamepadHapticActuatorBinding.rs");
    fs::write(&gamepad_haptic_binding, r#"
// Extended stub for GamepadHapticActuatorBinding

use crate::codegen::DomTypes::DomTypes;

pub trait GamepadHapticActuatorMethods<D: DomTypes> {}

pub mod GamepadHapticActuator_Binding {
    use super::*;
    pub use super::GamepadHapticActuatorMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadHapticEffectType {
    DualRumble,
}

#[derive(Clone, Debug, Default)]
pub struct GamepadEffectParameters {
    pub duration: Option<f64>,
    pub startDelay: Option<f64>,
    pub strongMagnitude: Option<f64>,
    pub weakMagnitude: Option<f64>,
}
"#).unwrap();

    // Create extended GeolocationBinding with callback
    let geolocation_binding = bindings_dir.join("GeolocationBinding.rs");
    fs::write(&geolocation_binding, r#"
// Extended stub for GeolocationBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait GeolocationMethods<D: DomTypes> {}

pub mod Geolocation_Binding {
    use super::*;
    pub use super::GeolocationMethods;
}

pub struct PositionCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct PositionErrorCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

#[derive(Clone, Debug, Default)]
pub struct PositionOptions {
    pub enableHighAccuracy: bool,
    pub timeout: Option<u32>,
    pub maximumAge: Option<u32>,
}
"#).unwrap();

    // Create extended MutationObserverBinding with callback
    let mutation_observer_binding = bindings_dir.join("MutationObserverBinding.rs");
    fs::write(&mutation_observer_binding, r#"
// Extended stub for MutationObserverBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait MutationObserverMethods<D: DomTypes> {}

pub mod MutationObserver_Binding {
    use super::*;
    pub use super::MutationObserverMethods;
}

pub struct MutationCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

#[derive(Clone, Debug, Default)]
pub struct MutationObserverInit {
    pub childList: bool,
    pub attributes: Option<bool>,
    pub characterData: Option<bool>,
    pub subtree: bool,
    pub attributeOldValue: Option<bool>,
    pub characterDataOldValue: Option<bool>,
    pub attributeFilter: Option<Vec<String>>,
}
"#).unwrap();

    // Create extended IntersectionObserverBinding with callback
    let intersection_observer_binding = bindings_dir.join("IntersectionObserverBinding.rs");
    fs::write(&intersection_observer_binding, r#"
// Extended stub for IntersectionObserverBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait IntersectionObserverMethods<D: DomTypes> {}

pub mod IntersectionObserver_Binding {
    use super::*;
    pub use super::IntersectionObserverMethods;
}

pub struct IntersectionObserverCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

#[derive(Clone, Debug, Default)]
pub struct IntersectionObserverInit {
    pub root: Option<()>,
    pub rootMargin: Option<String>,
    pub threshold: Option<f64>,
}
"#).unwrap();

    // Create extended IntersectionObserverEntryBinding with Init
    let intersection_observer_entry_binding = bindings_dir.join("IntersectionObserverEntryBinding.rs");
    fs::write(&intersection_observer_entry_binding, r#"
// Extended stub for IntersectionObserverEntryBinding

use crate::codegen::DomTypes::DomTypes;

pub trait IntersectionObserverEntryMethods<D: DomTypes> {}

pub mod IntersectionObserverEntry_Binding {
    use super::*;
    pub use super::IntersectionObserverEntryMethods;
}

#[derive(Clone, Debug, Default)]
pub struct IntersectionObserverEntryInit {
    pub time: f64,
    pub rootBounds: Option<()>,
    pub boundingClientRect: (),
    pub intersectionRect: (),
    pub isIntersecting: bool,
    pub intersectionRatio: f64,
    pub target: (),
}
"#).unwrap();

    // Create extended ResizeObserverBinding with types
    let resize_observer_binding = bindings_dir.join("ResizeObserverBinding.rs");
    fs::write(&resize_observer_binding, r#"
// Extended stub for ResizeObserverBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait ResizeObserverMethods<D: DomTypes> {}

pub mod ResizeObserver_Binding {
    use super::*;
    pub use super::ResizeObserverMethods;
}

pub struct ResizeObserverCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeObserverBoxOptions { ContentBox, BorderBox, DevicePixelContentBox }

#[derive(Clone, Debug, Default)]
pub struct ResizeObserverOptions {
    pub box_: Option<ResizeObserverBoxOptions>,
}
"#).unwrap();

    // Create extended PerformanceObserverBinding with callback
    let performance_observer_binding = bindings_dir.join("PerformanceObserverBinding.rs");
    fs::write(&performance_observer_binding, r#"
// Extended stub for PerformanceObserverBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait PerformanceObserverMethods<D: DomTypes> {}

pub mod PerformanceObserver_Binding {
    use super::*;
    pub use super::PerformanceObserverMethods;
}

pub struct PerformanceObserverCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

#[derive(Clone, Debug, Default)]
pub struct PerformanceObserverInit {
    pub entryTypes: Option<Vec<String>>,
    pub type_: Option<String>,
    pub buffered: Option<bool>,
}
"#).unwrap();

    // Create extended PerformanceNavigationBinding with constants
    let performance_navigation_binding = bindings_dir.join("PerformanceNavigationBinding.rs");
    fs::write(&performance_navigation_binding, r#"
// Extended stub for PerformanceNavigationBinding

use crate::codegen::DomTypes::DomTypes;

pub trait PerformanceNavigationMethods<D: DomTypes> {}

pub mod PerformanceNavigation_Binding {
    use super::*;
    pub use super::PerformanceNavigationMethods;
}

pub struct PerformanceNavigationConstants;
impl PerformanceNavigationConstants {
    pub const TYPE_NAVIGATE: u16 = 0;
    pub const TYPE_RELOAD: u16 = 1;
    pub const TYPE_BACK_FORWARD: u16 = 2;
    pub const TYPE_RESERVED: u16 = 255;
}
"#).unwrap();

    // Create extended PerformanceNavigationTimingBinding with types
    let perf_nav_timing_binding = bindings_dir.join("PerformanceNavigationTimingBinding.rs");
    fs::write(&perf_nav_timing_binding, r#"
// Extended stub for PerformanceNavigationTimingBinding

use crate::codegen::DomTypes::DomTypes;

pub trait PerformanceNavigationTimingMethods<D: DomTypes> {}

pub mod PerformanceNavigationTiming_Binding {
    use super::*;
    pub use super::PerformanceNavigationTimingMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationTimingType { Navigate, Reload, BackForward, Prerender }
"#).unwrap();

    // Create extended EventSourceBinding with Init
    let event_source_binding = bindings_dir.join("EventSourceBinding.rs");
    fs::write(&event_source_binding, r#"
// Extended stub for EventSourceBinding

use crate::codegen::DomTypes::DomTypes;

pub trait EventSourceMethods<D: DomTypes> {}

pub mod EventSource_Binding {
    use super::*;
    pub use super::EventSourceMethods;
}

#[derive(Clone, Debug, Default)]
pub struct EventSourceInit {
    pub withCredentials: bool,
}
"#).unwrap();

    // Create extended ExtendableEventBinding with Init
    let extendable_event_binding = bindings_dir.join("ExtendableEventBinding.rs");
    fs::write(&extendable_event_binding, r#"
// Extended stub for ExtendableEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ExtendableEventMethods<D: DomTypes> {}

pub mod ExtendableEvent_Binding {
    use super::*;
    pub use super::ExtendableEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct ExtendableEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
}
"#).unwrap();

    // Create extended AnimationEventBinding with Init
    let animation_event_binding = bindings_dir.join("AnimationEventBinding.rs");
    fs::write(&animation_event_binding, r#"
// Extended stub for AnimationEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait AnimationEventMethods<D: DomTypes> {}

pub mod AnimationEvent_Binding {
    use super::*;
    pub use super::AnimationEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct AnimationEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub animationName: String,
    pub elapsedTime: f64,
    pub pseudoElement: String,
}
"#).unwrap();

    // Create extended TransitionEventBinding with Init
    let transition_event_binding = bindings_dir.join("TransitionEventBinding.rs");
    fs::write(&transition_event_binding, r#"
// Extended stub for TransitionEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait TransitionEventMethods<D: DomTypes> {}

pub mod TransitionEvent_Binding {
    use super::*;
    pub use super::TransitionEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct TransitionEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub propertyName: String,
    pub elapsedTime: f64,
    pub pseudoElement: String,
}
"#).unwrap();

    // Create extended PointerEventBinding with Init
    let pointer_event_binding = bindings_dir.join("PointerEventBinding.rs");
    fs::write(&pointer_event_binding, r#"
// Extended stub for PointerEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait PointerEventMethods<D: DomTypes> {}

pub mod PointerEvent_Binding {
    use super::*;
    pub use super::PointerEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct PointerEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub pointerId: i32,
    pub width: f64,
    pub height: f64,
    pub pressure: f32,
    pub tangentialPressure: f32,
    pub tiltX: i32,
    pub tiltY: i32,
    pub twist: i32,
    pub pointerType: String,
    pub isPrimary: bool,
}
"#).unwrap();

    // Create extended Audio bindings
    let audio_context_binding = bindings_dir.join("AudioContextBinding.rs");
    fs::write(&audio_context_binding, r#"
// Extended stub for AudioContextBinding

use crate::codegen::DomTypes::DomTypes;

pub trait AudioContextMethods<D: DomTypes> {}

pub mod AudioContext_Binding {
    use super::*;
    pub use super::AudioContextMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioContextLatencyCategory { Balanced, Interactive, Playback }

#[derive(Clone, Debug, Default)]
pub struct AudioContextOptions {
    pub latencyHint: Option<AudioContextLatencyCategory>,
    pub sampleRate: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct AudioTimestamp {
    pub contextTime: Option<f64>,
    pub performanceTime: Option<f64>,
}
"#).unwrap();

    let base_audio_context_binding = bindings_dir.join("BaseAudioContextBinding.rs");
    fs::write(&base_audio_context_binding, r#"
// Extended stub for BaseAudioContextBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait BaseAudioContextMethods<D: DomTypes> {}

pub mod BaseAudioContext_Binding {
    use super::*;
    pub use super::BaseAudioContextMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioContextState { Suspended, Running, Closed }

/// DecodeSuccessCallback type
pub struct DecodeSuccessCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

/// DecodeErrorCallback type
pub struct DecodeErrorCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    let audio_buffer_source_node_binding = bindings_dir.join("AudioBufferSourceNodeBinding.rs");
    fs::write(&audio_buffer_source_node_binding, r#"
// Extended stub for AudioBufferSourceNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait AudioBufferSourceNodeMethods<D: DomTypes> {}

pub mod AudioBufferSourceNode_Binding {
    use super::*;
    pub use super::AudioBufferSourceNodeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct AudioBufferSourceOptions {
    pub buffer: Option<()>,
    pub detune: Option<f32>,
    pub loop_: Option<bool>,
    pub loopEnd: Option<f64>,
    pub loopStart: Option<f64>,
    pub playbackRate: Option<f32>,
}
"#).unwrap();

    let biquad_filter_node_binding = bindings_dir.join("BiquadFilterNodeBinding.rs");
    fs::write(&biquad_filter_node_binding, r#"
// Extended stub for BiquadFilterNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait BiquadFilterNodeMethods<D: DomTypes> {}

pub mod BiquadFilterNode_Binding {
    use super::*;
    pub use super::BiquadFilterNodeMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BiquadFilterType { Lowpass, Highpass, Bandpass, Lowshelf, Highshelf, Peaking, Notch, Allpass }

#[derive(Clone, Debug, Default)]
pub struct BiquadFilterOptions {
    pub type_: Option<BiquadFilterType>,
    pub Q: Option<f32>,
    pub detune: Option<f32>,
    pub frequency: Option<f32>,
    pub gain: Option<f32>,
}
"#).unwrap();

    // Audio node options bindings
    let channel_merger_node_binding = bindings_dir.join("ChannelMergerNodeBinding.rs");
    fs::write(&channel_merger_node_binding, r#"
// Extended stub for ChannelMergerNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ChannelMergerNodeMethods<D: DomTypes> {}

pub mod ChannelMergerNode_Binding {
    use super::*;
    pub use super::ChannelMergerNodeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct ChannelMergerOptions {
    pub numberOfInputs: Option<u32>,
}
"#).unwrap();

    let channel_splitter_node_binding = bindings_dir.join("ChannelSplitterNodeBinding.rs");
    fs::write(&channel_splitter_node_binding, r#"
// Extended stub for ChannelSplitterNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ChannelSplitterNodeMethods<D: DomTypes> {}

pub mod ChannelSplitterNode_Binding {
    use super::*;
    pub use super::ChannelSplitterNodeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct ChannelSplitterOptions {
    pub numberOfOutputs: Option<u32>,
}
"#).unwrap();

    let constant_source_node_binding = bindings_dir.join("ConstantSourceNodeBinding.rs");
    fs::write(&constant_source_node_binding, r#"
// Extended stub for ConstantSourceNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ConstantSourceNodeMethods<D: DomTypes> {}

pub mod ConstantSourceNode_Binding {
    use super::*;
    pub use super::ConstantSourceNodeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct ConstantSourceOptions {
    pub offset: Option<f32>,
}
"#).unwrap();

    let gain_node_binding = bindings_dir.join("GainNodeBinding.rs");
    fs::write(&gain_node_binding, r#"
// Extended stub for GainNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait GainNodeMethods<D: DomTypes> {}

pub mod GainNode_Binding {
    use super::*;
    pub use super::GainNodeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct GainOptions {
    pub gain: Option<f32>,
}
"#).unwrap();

    let iir_filter_node_binding = bindings_dir.join("IIRFilterNodeBinding.rs");
    fs::write(&iir_filter_node_binding, r#"
// Extended stub for IIRFilterNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait IIRFilterNodeMethods<D: DomTypes> {}

pub mod IIRFilterNode_Binding {
    use super::*;
    pub use super::IIRFilterNodeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct IIRFilterOptions {
    pub feedforward: Vec<f64>,
    pub feedback: Vec<f64>,
}
"#).unwrap();

    let oscillator_node_binding = bindings_dir.join("OscillatorNodeBinding.rs");
    fs::write(&oscillator_node_binding, r#"
// Extended stub for OscillatorNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait OscillatorNodeMethods<D: DomTypes> {}

pub mod OscillatorNode_Binding {
    use super::*;
    pub use super::OscillatorNodeMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OscillatorType { Sine, Square, Sawtooth, Triangle, Custom }

#[derive(Clone, Debug, Default)]
pub struct OscillatorOptions {
    pub type_: Option<OscillatorType>,
    pub frequency: Option<f32>,
    pub detune: Option<f32>,
}
"#).unwrap();

    let panner_node_binding = bindings_dir.join("PannerNodeBinding.rs");
    fs::write(&panner_node_binding, r#"
// Extended stub for PannerNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait PannerNodeMethods<D: DomTypes> {}

pub mod PannerNode_Binding {
    use super::*;
    pub use super::PannerNodeMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanningModelType { Equalpower, HRTF }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistanceModelType { Linear, Inverse, Exponential }

#[derive(Clone, Debug, Default)]
pub struct PannerOptions {
    pub panningModel: Option<PanningModelType>,
    pub distanceModel: Option<DistanceModelType>,
    pub positionX: Option<f32>,
    pub positionY: Option<f32>,
    pub positionZ: Option<f32>,
    pub orientationX: Option<f32>,
    pub orientationY: Option<f32>,
    pub orientationZ: Option<f32>,
    pub refDistance: Option<f64>,
    pub maxDistance: Option<f64>,
    pub rolloffFactor: Option<f64>,
    pub coneInnerAngle: Option<f64>,
    pub coneOuterAngle: Option<f64>,
    pub coneOuterGain: Option<f64>,
}
"#).unwrap();

    let stereo_panner_node_binding = bindings_dir.join("StereoPannerNodeBinding.rs");
    fs::write(&stereo_panner_node_binding, r#"
// Extended stub for StereoPannerNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait StereoPannerNodeMethods<D: DomTypes> {}

pub mod StereoPannerNode_Binding {
    use super::*;
    pub use super::StereoPannerNodeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct StereoPannerOptions {
    pub pan: Option<f32>,
}
"#).unwrap();

    let media_element_audio_source_node_binding = bindings_dir.join("MediaElementAudioSourceNodeBinding.rs");
    fs::write(&media_element_audio_source_node_binding, r#"
// Extended stub for MediaElementAudioSourceNodeBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait MediaElementAudioSourceNodeMethods<D: DomTypes> {}

pub mod MediaElementAudioSourceNode_Binding {
    use super::*;
    pub use super::MediaElementAudioSourceNodeMethods;
}

pub struct MediaElementAudioSourceOptions<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    let media_stream_audio_source_node_binding = bindings_dir.join("MediaStreamAudioSourceNodeBinding.rs");
    fs::write(&media_stream_audio_source_node_binding, r#"
// Extended stub for MediaStreamAudioSourceNodeBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait MediaStreamAudioSourceNodeMethods<D: DomTypes> {}

pub mod MediaStreamAudioSourceNode_Binding {
    use super::*;
    pub use super::MediaStreamAudioSourceNodeMethods;
}

pub struct MediaStreamAudioSourceOptions<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    let media_stream_track_audio_source_node_binding = bindings_dir.join("MediaStreamTrackAudioSourceNodeBinding.rs");
    fs::write(&media_stream_track_audio_source_node_binding, r#"
// Extended stub for MediaStreamTrackAudioSourceNodeBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait MediaStreamTrackAudioSourceNodeMethods<D: DomTypes> {}

pub mod MediaStreamTrackAudioSourceNode_Binding {
    use super::*;
    pub use super::MediaStreamTrackAudioSourceNodeMethods;
}

pub struct MediaStreamTrackAudioSourceOptions<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    let audio_node_binding = bindings_dir.join("AudioNodeBinding.rs");
    fs::write(&audio_node_binding, r#"
// Extended stub for AudioNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait AudioNodeMethods<D: DomTypes> {}

pub mod AudioNode_Binding {
    use super::*;
    pub use super::AudioNodeMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelCountMode { Max, ClampedMax, Explicit }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelInterpretation { Speakers, Discrete }

#[derive(Clone, Debug, Default)]
pub struct AudioNodeOptions {
    pub channelCount: Option<u32>,
    pub channelCountMode: Option<ChannelCountMode>,
    pub channelInterpretation: Option<ChannelInterpretation>,
}
"#).unwrap();

    let audio_param_binding = bindings_dir.join("AudioParamBinding.rs");
    fs::write(&audio_param_binding, r#"
// Extended stub for AudioParamBinding

use crate::codegen::DomTypes::DomTypes;

pub trait AudioParamMethods<D: DomTypes> {}

pub mod AudioParam_Binding {
    use super::*;
    pub use super::AudioParamMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationRate { ARate, KRate }
"#).unwrap();

    let audio_buffer_binding = bindings_dir.join("AudioBufferBinding.rs");
    fs::write(&audio_buffer_binding, r#"
// Extended stub for AudioBufferBinding

use crate::codegen::DomTypes::DomTypes;

pub trait AudioBufferMethods<D: DomTypes> {}

pub mod AudioBuffer_Binding {
    use super::*;
    pub use super::AudioBufferMethods;
}

#[derive(Clone, Debug, Default)]
pub struct AudioBufferOptions {
    pub numberOfChannels: u32,
    pub length: u32,
    pub sampleRate: f32,
}
"#).unwrap();

    let analyser_node_binding = bindings_dir.join("AnalyserNodeBinding.rs");
    fs::write(&analyser_node_binding, r#"
// Extended stub for AnalyserNodeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait AnalyserNodeMethods<D: DomTypes> {}

pub mod AnalyserNode_Binding {
    use super::*;
    pub use super::AnalyserNodeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct AnalyserOptions {
    pub fftSize: Option<u32>,
    pub maxDecibels: Option<f64>,
    pub minDecibels: Option<f64>,
    pub smoothingTimeConstant: Option<f64>,
}
"#).unwrap();

    // Create IDB bindings
    let idb_cursor_binding = bindings_dir.join("IDBCursorBinding.rs");
    fs::write(&idb_cursor_binding, r#"
// Extended stub for IDBCursorBinding

use crate::codegen::DomTypes::DomTypes;

pub trait IDBCursorMethods<D: DomTypes> {}

pub mod IDBCursor_Binding {
    use super::*;
    pub use super::IDBCursorMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IDBCursorDirection { Next, Nextunique, Prev, Prevunique }
"#).unwrap();

    let idb_database_binding = bindings_dir.join("IDBDatabaseBinding.rs");
    fs::write(&idb_database_binding, r#"
// Extended stub for IDBDatabaseBinding

use crate::codegen::DomTypes::DomTypes;

pub trait IDBDatabaseMethods<D: DomTypes> {}

pub mod IDBDatabase_Binding {
    use super::*;
    pub use super::IDBDatabaseMethods;
}

#[derive(Clone, Debug, Default)]
pub struct IDBObjectStoreParameters {
    pub keyPath: Option<String>,
    pub autoIncrement: bool,
}

#[derive(Clone, Debug, Default)]
pub struct IDBIndexParameters {
    pub unique: bool,
    pub multiEntry: bool,
}

#[derive(Clone, Debug, Default)]
pub struct IDBTransactionOptions {
    pub durability: Option<IDBTransactionDurability>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IDBTransactionDurability { Default, Strict, Relaxed }
"#).unwrap();

    let idb_request_binding = bindings_dir.join("IDBRequestBinding.rs");
    fs::write(&idb_request_binding, r#"
// Extended stub for IDBRequestBinding

use crate::codegen::DomTypes::DomTypes;

pub trait IDBRequestMethods<D: DomTypes> {}

pub mod IDBRequest_Binding {
    use super::*;
    pub use super::IDBRequestMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IDBRequestReadyState { Pending, Done }
"#).unwrap();

    let idb_transaction_binding = bindings_dir.join("IDBTransactionBinding.rs");
    fs::write(&idb_transaction_binding, r#"
// Extended stub for IDBTransactionBinding

use crate::codegen::DomTypes::DomTypes;

pub trait IDBTransactionMethods<D: DomTypes> {}

pub mod IDBTransaction_Binding {
    use super::*;
    pub use super::IDBTransactionMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IDBTransactionMode { Readonly, Readwrite, Versionchange }
"#).unwrap();

    let idb_version_change_event_binding = bindings_dir.join("IDBVersionChangeEventBinding.rs");
    fs::write(&idb_version_change_event_binding, r#"
// Extended stub for IDBVersionChangeEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait IDBVersionChangeEventMethods<D: DomTypes> {}

pub mod IDBVersionChangeEvent_Binding {
    use super::*;
    pub use super::IDBVersionChangeEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct IDBVersionChangeEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub oldVersion: u64,
    pub newVersion: Option<u64>,
}
"#).unwrap();

    // Create WebGL bindings
    let webgl_context_binding = bindings_dir.join("WebGLRenderingContextBinding.rs");
    fs::write(&webgl_context_binding, r#"
// Extended stub for WebGLRenderingContextBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait WebGLRenderingContextMethods<D: DomTypes> {}

pub mod WebGLRenderingContext_Binding {
    use super::*;
    pub use super::WebGLRenderingContextMethods;
}

#[derive(Clone, Debug, Default)]
pub struct WebGLContextAttributes {
    pub alpha: bool,
    pub depth: bool,
    pub stencil: bool,
    pub antialias: bool,
    pub premultipliedAlpha: bool,
    pub preserveDrawingBuffer: bool,
    pub powerPreference: Option<String>,
    pub failIfMajorPerformanceCaveat: bool,
    pub desynchronized: bool,
}

pub struct WebGLRenderingContextConstants;

pub enum TexImageSource<D: DomTypes> {
    ImageBitmap(PhantomData<D>),
    ImageData(PhantomData<D>),
    HTMLImageElement(PhantomData<D>),
    HTMLCanvasElement(PhantomData<D>),
    HTMLVideoElement(PhantomData<D>),
    OffscreenCanvas(PhantomData<D>),
}
"#).unwrap();

    let webgl2_context_binding = bindings_dir.join("WebGL2RenderingContextBinding.rs");
    fs::write(&webgl2_context_binding, r#"
// Extended stub for WebGL2RenderingContextBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait WebGL2RenderingContextMethods<D: DomTypes> {}

pub mod WebGL2RenderingContext_Binding {
    use super::*;
    pub use super::WebGL2RenderingContextMethods;
}

/// WebGL2 rendering context constants
pub mod WebGL2RenderingContextConstants {
    // WebGL 2.0 constants
    pub const READ_BUFFER: u32 = 0x0C02;
    pub const UNPACK_ROW_LENGTH: u32 = 0x0CF2;
    pub const UNPACK_SKIP_ROWS: u32 = 0x0CF3;
    pub const UNPACK_SKIP_PIXELS: u32 = 0x0CF4;
    pub const PACK_ROW_LENGTH: u32 = 0x0D02;
    pub const PACK_SKIP_ROWS: u32 = 0x0D03;
    pub const PACK_SKIP_PIXELS: u32 = 0x0D04;
    pub const COLOR: u32 = 0x1800;
    pub const DEPTH: u32 = 0x1801;
    pub const STENCIL: u32 = 0x1802;
    pub const RED: u32 = 0x1903;
    pub const RGB8: u32 = 0x8051;
    pub const RGBA8: u32 = 0x8058;
    pub const RGB10_A2: u32 = 0x8059;
    pub const TEXTURE_BINDING_3D: u32 = 0x806A;
    pub const UNPACK_SKIP_IMAGES: u32 = 0x806D;
    pub const UNPACK_IMAGE_HEIGHT: u32 = 0x806E;
    pub const TEXTURE_3D: u32 = 0x806F;
    pub const TEXTURE_WRAP_R: u32 = 0x8072;
    pub const MAX_3D_TEXTURE_SIZE: u32 = 0x8073;
    pub const UNSIGNED_INT_2_10_10_10_REV: u32 = 0x8368;
    pub const MAX_ELEMENTS_VERTICES: u32 = 0x80E8;
    pub const MAX_ELEMENTS_INDICES: u32 = 0x80E9;
    pub const TEXTURE_MIN_LOD: u32 = 0x813A;
    pub const TEXTURE_MAX_LOD: u32 = 0x813B;
    pub const TEXTURE_BASE_LEVEL: u32 = 0x813C;
    pub const TEXTURE_MAX_LEVEL: u32 = 0x813D;
    pub const MIN: u32 = 0x8007;
    pub const MAX: u32 = 0x8008;
    pub const DEPTH_COMPONENT24: u32 = 0x81A6;
    pub const MAX_TEXTURE_LOD_BIAS: u32 = 0x84FD;
    pub const TEXTURE_COMPARE_MODE: u32 = 0x884C;
    pub const TEXTURE_COMPARE_FUNC: u32 = 0x884D;
    pub const CURRENT_QUERY: u32 = 0x8865;
    pub const QUERY_RESULT: u32 = 0x8866;
    pub const QUERY_RESULT_AVAILABLE: u32 = 0x8867;
    pub const STREAM_READ: u32 = 0x88E1;
    pub const STREAM_COPY: u32 = 0x88E2;
    pub const STATIC_READ: u32 = 0x88E5;
    pub const STATIC_COPY: u32 = 0x88E6;
    pub const DYNAMIC_READ: u32 = 0x88E9;
    pub const DYNAMIC_COPY: u32 = 0x88EA;
    pub const MAX_DRAW_BUFFERS: u32 = 0x8824;
    pub const DRAW_BUFFER0: u32 = 0x8825;
    pub const DRAW_BUFFER1: u32 = 0x8826;
    pub const DRAW_BUFFER2: u32 = 0x8827;
    pub const DRAW_BUFFER3: u32 = 0x8828;
    pub const DRAW_BUFFER4: u32 = 0x8829;
    pub const DRAW_BUFFER5: u32 = 0x882A;
    pub const DRAW_BUFFER6: u32 = 0x882B;
    pub const DRAW_BUFFER7: u32 = 0x882C;
    pub const DRAW_BUFFER8: u32 = 0x882D;
    pub const DRAW_BUFFER9: u32 = 0x882E;
    pub const DRAW_BUFFER10: u32 = 0x882F;
    pub const DRAW_BUFFER11: u32 = 0x8830;
    pub const DRAW_BUFFER12: u32 = 0x8831;
    pub const DRAW_BUFFER13: u32 = 0x8832;
    pub const DRAW_BUFFER14: u32 = 0x8833;
    pub const DRAW_BUFFER15: u32 = 0x8834;
    pub const MAX_FRAGMENT_UNIFORM_COMPONENTS: u32 = 0x8B49;
    pub const MAX_VERTEX_UNIFORM_COMPONENTS: u32 = 0x8B4A;
    pub const SAMPLER_3D: u32 = 0x8B5F;
    pub const SAMPLER_2D_SHADOW: u32 = 0x8B62;
    pub const FRAGMENT_SHADER_DERIVATIVE_HINT: u32 = 0x8B8B;
    pub const PIXEL_PACK_BUFFER: u32 = 0x88EB;
    pub const PIXEL_UNPACK_BUFFER: u32 = 0x88EC;
    pub const PIXEL_PACK_BUFFER_BINDING: u32 = 0x88ED;
    pub const PIXEL_UNPACK_BUFFER_BINDING: u32 = 0x88EF;
    pub const FLOAT_MAT2x3: u32 = 0x8B65;
    pub const FLOAT_MAT2x4: u32 = 0x8B66;
    pub const FLOAT_MAT3x2: u32 = 0x8B67;
    pub const FLOAT_MAT3x4: u32 = 0x8B68;
    pub const FLOAT_MAT4x2: u32 = 0x8B69;
    pub const FLOAT_MAT4x3: u32 = 0x8B6A;
    pub const SRGB: u32 = 0x8C40;
    pub const SRGB8: u32 = 0x8C41;
    pub const SRGB8_ALPHA8: u32 = 0x8C43;
    pub const COMPARE_REF_TO_TEXTURE: u32 = 0x884E;
    pub const RGBA32F: u32 = 0x8814;
    pub const RGB32F: u32 = 0x8815;
    pub const RGBA16F: u32 = 0x881A;
    pub const RGB16F: u32 = 0x881B;
    pub const VERTEX_ATTRIB_ARRAY_INTEGER: u32 = 0x88FD;
    pub const MAX_ARRAY_TEXTURE_LAYERS: u32 = 0x88FF;
    pub const MIN_PROGRAM_TEXEL_OFFSET: u32 = 0x8904;
    pub const MAX_PROGRAM_TEXEL_OFFSET: u32 = 0x8905;
    pub const MAX_VARYING_COMPONENTS: u32 = 0x8B4B;
    pub const TEXTURE_2D_ARRAY: u32 = 0x8C1A;
    pub const TEXTURE_BINDING_2D_ARRAY: u32 = 0x8C1D;
    pub const R11F_G11F_B10F: u32 = 0x8C3A;
    pub const UNSIGNED_INT_10F_11F_11F_REV: u32 = 0x8C3B;
    pub const RGB9_E5: u32 = 0x8C3D;
    pub const UNSIGNED_INT_5_9_9_9_REV: u32 = 0x8C3E;
    pub const TRANSFORM_FEEDBACK_BUFFER_MODE: u32 = 0x8C7F;
    pub const MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS: u32 = 0x8C80;
    pub const TRANSFORM_FEEDBACK_VARYINGS: u32 = 0x8C83;
    pub const TRANSFORM_FEEDBACK_BUFFER_START: u32 = 0x8C84;
    pub const TRANSFORM_FEEDBACK_BUFFER_SIZE: u32 = 0x8C85;
    pub const TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN: u32 = 0x8C88;
    pub const RASTERIZER_DISCARD: u32 = 0x8C89;
    pub const MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS: u32 = 0x8C8A;
    pub const MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS: u32 = 0x8C8B;
    pub const INTERLEAVED_ATTRIBS: u32 = 0x8C8C;
    pub const SEPARATE_ATTRIBS: u32 = 0x8C8D;
    pub const TRANSFORM_FEEDBACK_BUFFER: u32 = 0x8C8E;
    pub const TRANSFORM_FEEDBACK_BUFFER_BINDING: u32 = 0x8C8F;
    pub const RGBA32UI: u32 = 0x8D70;
    pub const RGB32UI: u32 = 0x8D71;
    pub const RGBA16UI: u32 = 0x8D76;
    pub const RGB16UI: u32 = 0x8D77;
    pub const RGBA8UI: u32 = 0x8D7C;
    pub const RGB8UI: u32 = 0x8D7D;
    pub const RGBA32I: u32 = 0x8D82;
    pub const RGB32I: u32 = 0x8D83;
    pub const RGBA16I: u32 = 0x8D88;
    pub const RGB16I: u32 = 0x8D89;
    pub const RGBA8I: u32 = 0x8D8E;
    pub const RGB8I: u32 = 0x8D8F;
    pub const RED_INTEGER: u32 = 0x8D94;
    pub const RGB_INTEGER: u32 = 0x8D98;
    pub const RGBA_INTEGER: u32 = 0x8D99;
    pub const SAMPLER_2D_ARRAY: u32 = 0x8DC1;
    pub const SAMPLER_2D_ARRAY_SHADOW: u32 = 0x8DC4;
    pub const SAMPLER_CUBE_SHADOW: u32 = 0x8DC5;
    pub const UNSIGNED_INT_VEC2: u32 = 0x8DC6;
    pub const UNSIGNED_INT_VEC3: u32 = 0x8DC7;
    pub const UNSIGNED_INT_VEC4: u32 = 0x8DC8;
    pub const INT_SAMPLER_2D: u32 = 0x8DCA;
    pub const INT_SAMPLER_3D: u32 = 0x8DCB;
    pub const INT_SAMPLER_CUBE: u32 = 0x8DCC;
    pub const INT_SAMPLER_2D_ARRAY: u32 = 0x8DCF;
    pub const UNSIGNED_INT_SAMPLER_2D: u32 = 0x8DD2;
    pub const UNSIGNED_INT_SAMPLER_3D: u32 = 0x8DD3;
    pub const UNSIGNED_INT_SAMPLER_CUBE: u32 = 0x8DD4;
    pub const UNSIGNED_INT_SAMPLER_2D_ARRAY: u32 = 0x8DD7;
    pub const DEPTH_COMPONENT32F: u32 = 0x8CAC;
    pub const DEPTH32F_STENCIL8: u32 = 0x8CAD;
    pub const FLOAT_32_UNSIGNED_INT_24_8_REV: u32 = 0x8DAD;
    pub const FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING: u32 = 0x8210;
    pub const FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE: u32 = 0x8211;
    pub const FRAMEBUFFER_ATTACHMENT_RED_SIZE: u32 = 0x8212;
    pub const FRAMEBUFFER_ATTACHMENT_GREEN_SIZE: u32 = 0x8213;
    pub const FRAMEBUFFER_ATTACHMENT_BLUE_SIZE: u32 = 0x8214;
    pub const FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE: u32 = 0x8215;
    pub const FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE: u32 = 0x8216;
    pub const FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE: u32 = 0x8217;
    pub const FRAMEBUFFER_DEFAULT: u32 = 0x8218;
    pub const UNSIGNED_INT_24_8: u32 = 0x84FA;
    pub const DEPTH24_STENCIL8: u32 = 0x88F0;
    pub const UNSIGNED_NORMALIZED: u32 = 0x8C17;
    pub const DRAW_FRAMEBUFFER_BINDING: u32 = 0x8CA6;
    pub const READ_FRAMEBUFFER: u32 = 0x8CA8;
    pub const DRAW_FRAMEBUFFER: u32 = 0x8CA9;
    pub const READ_FRAMEBUFFER_BINDING: u32 = 0x8CAA;
    pub const RENDERBUFFER_SAMPLES: u32 = 0x8CAB;
    pub const FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER: u32 = 0x8CD4;
    pub const MAX_COLOR_ATTACHMENTS: u32 = 0x8CDF;
    pub const COLOR_ATTACHMENT1: u32 = 0x8CE1;
    pub const COLOR_ATTACHMENT2: u32 = 0x8CE2;
    pub const COLOR_ATTACHMENT3: u32 = 0x8CE3;
    pub const COLOR_ATTACHMENT4: u32 = 0x8CE4;
    pub const COLOR_ATTACHMENT5: u32 = 0x8CE5;
    pub const COLOR_ATTACHMENT6: u32 = 0x8CE6;
    pub const COLOR_ATTACHMENT7: u32 = 0x8CE7;
    pub const COLOR_ATTACHMENT8: u32 = 0x8CE8;
    pub const COLOR_ATTACHMENT9: u32 = 0x8CE9;
    pub const COLOR_ATTACHMENT10: u32 = 0x8CEA;
    pub const COLOR_ATTACHMENT11: u32 = 0x8CEB;
    pub const COLOR_ATTACHMENT12: u32 = 0x8CEC;
    pub const COLOR_ATTACHMENT13: u32 = 0x8CED;
    pub const COLOR_ATTACHMENT14: u32 = 0x8CEE;
    pub const COLOR_ATTACHMENT15: u32 = 0x8CEF;
    pub const FRAMEBUFFER_INCOMPLETE_MULTISAMPLE: u32 = 0x8D56;
    pub const MAX_SAMPLES: u32 = 0x8D57;
    pub const HALF_FLOAT: u32 = 0x140B;
    pub const RG: u32 = 0x8227;
    pub const RG_INTEGER: u32 = 0x8228;
    pub const R8: u32 = 0x8229;
    pub const RG8: u32 = 0x822B;
    pub const R16F: u32 = 0x822D;
    pub const R32F: u32 = 0x822E;
    pub const RG16F: u32 = 0x822F;
    pub const RG32F: u32 = 0x8230;
    pub const R8I: u32 = 0x8231;
    pub const R8UI: u32 = 0x8232;
    pub const R16I: u32 = 0x8233;
    pub const R16UI: u32 = 0x8234;
    pub const R32I: u32 = 0x8235;
    pub const R32UI: u32 = 0x8236;
    pub const RG8I: u32 = 0x8237;
    pub const RG8UI: u32 = 0x8238;
    pub const RG16I: u32 = 0x8239;
    pub const RG16UI: u32 = 0x823A;
    pub const RG32I: u32 = 0x823B;
    pub const RG32UI: u32 = 0x823C;
    pub const VERTEX_ARRAY_BINDING: u32 = 0x85B5;
    pub const R8_SNORM: u32 = 0x8F94;
    pub const RG8_SNORM: u32 = 0x8F95;
    pub const RGB8_SNORM: u32 = 0x8F96;
    pub const RGBA8_SNORM: u32 = 0x8F97;
    pub const SIGNED_NORMALIZED: u32 = 0x8F9C;
    pub const COPY_READ_BUFFER: u32 = 0x8F36;
    pub const COPY_WRITE_BUFFER: u32 = 0x8F37;
    pub const COPY_READ_BUFFER_BINDING: u32 = 0x8F36;
    pub const COPY_WRITE_BUFFER_BINDING: u32 = 0x8F37;
    pub const UNIFORM_BUFFER: u32 = 0x8A11;
    pub const UNIFORM_BUFFER_BINDING: u32 = 0x8A28;
    pub const UNIFORM_BUFFER_START: u32 = 0x8A29;
    pub const UNIFORM_BUFFER_SIZE: u32 = 0x8A2A;
    pub const MAX_VERTEX_UNIFORM_BLOCKS: u32 = 0x8A2B;
    pub const MAX_FRAGMENT_UNIFORM_BLOCKS: u32 = 0x8A2D;
    pub const MAX_COMBINED_UNIFORM_BLOCKS: u32 = 0x8A2E;
    pub const MAX_UNIFORM_BUFFER_BINDINGS: u32 = 0x8A2F;
    pub const MAX_UNIFORM_BLOCK_SIZE: u32 = 0x8A30;
    pub const MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS: u32 = 0x8A31;
    pub const MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS: u32 = 0x8A33;
    pub const UNIFORM_BUFFER_OFFSET_ALIGNMENT: u32 = 0x8A34;
    pub const ACTIVE_UNIFORM_BLOCKS: u32 = 0x8A36;
    pub const UNIFORM_TYPE: u32 = 0x8A37;
    pub const UNIFORM_SIZE: u32 = 0x8A38;
    pub const UNIFORM_BLOCK_INDEX: u32 = 0x8A3A;
    pub const UNIFORM_OFFSET: u32 = 0x8A3B;
    pub const UNIFORM_ARRAY_STRIDE: u32 = 0x8A3C;
    pub const UNIFORM_MATRIX_STRIDE: u32 = 0x8A3D;
    pub const UNIFORM_IS_ROW_MAJOR: u32 = 0x8A3E;
    pub const UNIFORM_BLOCK_BINDING: u32 = 0x8A3F;
    pub const UNIFORM_BLOCK_DATA_SIZE: u32 = 0x8A40;
    pub const UNIFORM_BLOCK_ACTIVE_UNIFORMS: u32 = 0x8A42;
    pub const UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES: u32 = 0x8A43;
    pub const UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER: u32 = 0x8A44;
    pub const UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER: u32 = 0x8A46;
    pub const INVALID_INDEX: u32 = 0xFFFFFFFF;
    pub const MAX_VERTEX_OUTPUT_COMPONENTS: u32 = 0x9122;
    pub const MAX_FRAGMENT_INPUT_COMPONENTS: u32 = 0x9125;
    pub const MAX_SERVER_WAIT_TIMEOUT: u32 = 0x9111;
    pub const OBJECT_TYPE: u32 = 0x9112;
    pub const SYNC_CONDITION: u32 = 0x9113;
    pub const SYNC_STATUS: u32 = 0x9114;
    pub const SYNC_FLAGS: u32 = 0x9115;
    pub const SYNC_FENCE: u32 = 0x9116;
    pub const SYNC_GPU_COMMANDS_COMPLETE: u32 = 0x9117;
    pub const UNSIGNALED: u32 = 0x9118;
    pub const SIGNALED: u32 = 0x9119;
    pub const ALREADY_SIGNALED: u32 = 0x911A;
    pub const TIMEOUT_EXPIRED: u32 = 0x911B;
    pub const CONDITION_SATISFIED: u32 = 0x911C;
    pub const WAIT_FAILED: u32 = 0x911D;
    pub const SYNC_FLUSH_COMMANDS_BIT: u32 = 0x00000001;
    pub const VERTEX_ATTRIB_ARRAY_DIVISOR: u32 = 0x88FE;
    pub const ANY_SAMPLES_PASSED: u32 = 0x8C2F;
    pub const ANY_SAMPLES_PASSED_CONSERVATIVE: u32 = 0x8D6A;
    pub const SAMPLER_BINDING: u32 = 0x8919;
    pub const RGB10_A2UI: u32 = 0x906F;
    pub const INT_2_10_10_10_REV: u32 = 0x8D9F;
    pub const TRANSFORM_FEEDBACK: u32 = 0x8E22;
    pub const TRANSFORM_FEEDBACK_PAUSED: u32 = 0x8E23;
    pub const TRANSFORM_FEEDBACK_ACTIVE: u32 = 0x8E24;
    pub const TRANSFORM_FEEDBACK_BINDING: u32 = 0x8E25;
    pub const TEXTURE_IMMUTABLE_FORMAT: u32 = 0x912F;
    pub const MAX_ELEMENT_INDEX: u32 = 0x8D6B;
    pub const TEXTURE_IMMUTABLE_LEVELS: u32 = 0x82DF;
    pub const TIMEOUT_IGNORED: i64 = -1;
    pub const MAX_CLIENT_WAIT_TIMEOUT_WEBGL: u32 = 0x9247;
}
"#).unwrap();

    let webgl_context_event_binding = bindings_dir.join("WebGLContextEventBinding.rs");
    fs::write(&webgl_context_event_binding, r#"
// Extended stub for WebGLContextEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait WebGLContextEventMethods<D: DomTypes> {}

pub mod WebGLContextEvent_Binding {
    use super::*;
    pub use super::WebGLContextEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct WebGLContextEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub statusMessage: String,
}
"#).unwrap();

    // Create RTC bindings
    let rtc_peer_connection_binding = bindings_dir.join("RTCPeerConnectionBinding.rs");
    fs::write(&rtc_peer_connection_binding, r#"
// Extended stub for RTCPeerConnectionBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait RTCPeerConnectionMethods<D: DomTypes> {}

pub mod RTCPeerConnection_Binding {
    use super::*;
    pub use super::RTCPeerConnectionMethods;
}

#[derive(Clone, Debug, Default)]
pub struct RTCConfiguration {
    pub iceServers: Option<Vec<()>>,
    pub iceTransportPolicy: Option<String>,
    pub bundlePolicy: Option<String>,
    pub rtcpMuxPolicy: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct RTCOfferOptions {
    pub iceRestart: bool,
    pub offerToReceiveAudio: Option<bool>,
    pub offerToReceiveVideo: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct RTCAnswerOptions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RTCBundlePolicy { Balanced, MaxCompat, MaxBundle }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RTCIceConnectionState { New, Checking, Connected, Completed, Failed, Disconnected, Closed }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RTCIceGatheringState { New, Gathering, Complete }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RTCSignalingState { Stable, HaveLocalOffer, HaveRemoteOffer, HaveLocalPranswer, HaveRemotePranswer, Closed }

#[derive(Clone, Debug, Default)]
pub struct RTCRtpTransceiverInit {
    pub direction: Option<RTCRtpTransceiverDirection>,
    pub streams: Option<Vec<()>>,
    pub sendEncodings: Option<Vec<()>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RTCRtpTransceiverDirection { Sendrecv, Sendonly, Recvonly, Inactive, Stopped }
"#).unwrap();

    let rtc_session_description_binding = bindings_dir.join("RTCSessionDescriptionBinding.rs");
    fs::write(&rtc_session_description_binding, r#"
// Extended stub for RTCSessionDescriptionBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RTCSessionDescriptionMethods<D: DomTypes> {}

pub mod RTCSessionDescription_Binding {
    use super::*;
    pub use super::RTCSessionDescriptionMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RTCSdpType { Offer, Pranswer, Answer, Rollback }

#[derive(Clone, Debug, Default)]
pub struct RTCSessionDescriptionInit {
    pub type_: Option<RTCSdpType>,
    pub sdp: String,
}
"#).unwrap();

    let rtc_ice_candidate_binding = bindings_dir.join("RTCIceCandidateBinding.rs");
    fs::write(&rtc_ice_candidate_binding, r#"
// Extended stub for RTCIceCandidateBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RTCIceCandidateMethods<D: DomTypes> {}

pub mod RTCIceCandidate_Binding {
    use super::*;
    pub use super::RTCIceCandidateMethods;
}

#[derive(Clone, Debug, Default)]
pub struct RTCIceCandidateInit {
    pub candidate: String,
    pub sdpMid: Option<String>,
    pub sdpMLineIndex: Option<u16>,
    pub usernameFragment: Option<String>,
}
"#).unwrap();

    let rtc_data_channel_binding = bindings_dir.join("RTCDataChannelBinding.rs");
    fs::write(&rtc_data_channel_binding, r#"
// Extended stub for RTCDataChannelBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RTCDataChannelMethods<D: DomTypes> {}

pub mod RTCDataChannel_Binding {
    use super::*;
    pub use super::RTCDataChannelMethods;
}

#[derive(Clone, Debug, Default)]
pub struct RTCDataChannelInit {
    pub ordered: Option<bool>,
    pub maxPacketLifeTime: Option<u16>,
    pub maxRetransmits: Option<u16>,
    pub protocol: Option<String>,
    pub negotiated: Option<bool>,
    pub id: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RTCDataChannelState { Connecting, Open, Closing, Closed }
"#).unwrap();

    let rtc_rtp_transceiver_binding = bindings_dir.join("RTCRtpTransceiverBinding.rs");
    fs::write(&rtc_rtp_transceiver_binding, r#"
// Extended stub for RTCRtpTransceiverBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RTCRtpTransceiverMethods<D: DomTypes> {}

pub mod RTCRtpTransceiver_Binding {
    use super::*;
    pub use super::RTCRtpTransceiverMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RTCRtpTransceiverDirection { Sendrecv, Sendonly, Recvonly, Inactive, Stopped }
"#).unwrap();

    let rtc_rtp_sender_binding = bindings_dir.join("RTCRtpSenderBinding.rs");
    fs::write(&rtc_rtp_sender_binding, r#"
// Extended stub for RTCRtpSenderBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RTCRtpSenderMethods<D: DomTypes> {}

pub mod RTCRtpSender_Binding {
    use super::*;
    pub use super::RTCRtpSenderMethods;
}

#[derive(Clone, Debug, Default)]
pub struct RTCRtcpParameters {
    pub cname: Option<String>,
    pub reducedSize: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct RTCRtpParameters {
    pub rtcp: Option<RTCRtcpParameters>,
    pub codecs: Vec<()>,
    pub headerExtensions: Vec<()>,
}

#[derive(Clone, Debug, Default)]
pub struct RTCRtpSendParameters {
    pub rtcp: Option<RTCRtcpParameters>,
    pub codecs: Vec<()>,
    pub headerExtensions: Vec<()>,
    pub encodings: Vec<()>,
    pub transactionId: Option<String>,
}
"#).unwrap();

    let rtc_error_binding = bindings_dir.join("RTCErrorBinding.rs");
    fs::write(&rtc_error_binding, r#"
// Extended stub for RTCErrorBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RTCErrorMethods<D: DomTypes> {}

pub mod RTCError_Binding {
    use super::*;
    pub use super::RTCErrorMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RTCErrorDetailType {
    DataChannelFailure,
    DtlsFailure,
    FingerprintFailure,
    SctpFailure,
    SdpSyntaxError,
    HardwareEncoderNotAvailable,
    HardwareEncoderError,
}

#[derive(Clone, Debug, Default)]
pub struct RTCErrorInit {
    pub errorDetail: Option<RTCErrorDetailType>,
    pub sdpLineNumber: Option<i32>,
    pub sctpCauseCode: Option<i32>,
    pub receivedAlert: Option<u32>,
    pub sentAlert: Option<u32>,
}
"#).unwrap();

    // Create more event bindings
    let rtc_data_channel_event_binding = bindings_dir.join("RTCDataChannelEventBinding.rs");
    fs::write(&rtc_data_channel_event_binding, r#"
// Extended stub for RTCDataChannelEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RTCDataChannelEventMethods<D: DomTypes> {}

pub mod RTCDataChannelEvent_Binding {
    use super::*;
    pub use super::RTCDataChannelEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct RTCDataChannelEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub channel: (),
}
"#).unwrap();

    let rtc_peer_connection_ice_event_binding = bindings_dir.join("RTCPeerConnectionIceEventBinding.rs");
    fs::write(&rtc_peer_connection_ice_event_binding, r#"
// Extended stub for RTCPeerConnectionIceEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RTCPeerConnectionIceEventMethods<D: DomTypes> {}

pub mod RTCPeerConnectionIceEvent_Binding {
    use super::*;
    pub use super::RTCPeerConnectionIceEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct RTCPeerConnectionIceEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub candidate: Option<()>,
    pub url: Option<String>,
}
"#).unwrap();

    let rtc_error_event_binding = bindings_dir.join("RTCErrorEventBinding.rs");
    fs::write(&rtc_error_event_binding, r#"
// Extended stub for RTCErrorEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait RTCErrorEventMethods<D: DomTypes> {}

pub mod RTCErrorEvent_Binding {
    use super::*;
    pub use super::RTCErrorEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct RTCErrorEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub error: (),
}
"#).unwrap();

    // Create Stream bindings
    let readable_stream_binding = bindings_dir.join("ReadableStreamBinding.rs");
    fs::write(&readable_stream_binding, r#"
// Extended stub for ReadableStreamBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait ReadableStreamMethods<D: DomTypes> {}

pub mod ReadableStream_Binding {
    use super::*;
    pub use super::ReadableStreamMethods;
}

#[derive(Clone, Debug, Default)]
pub struct ReadableStreamGetReaderOptions {
    pub mode: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadableStreamReaderMode { Byob }

#[derive(Clone)]
pub struct ReadableWritablePair<D: DomTypes> {
    _marker: PhantomData<D>,
}

#[derive(Clone, Debug, Default)]
pub struct StreamPipeOptions {
    pub preventClose: Option<bool>,
    pub preventAbort: Option<bool>,
    pub preventCancel: Option<bool>,
}
"#).unwrap();

    let readable_stream_default_reader_binding = bindings_dir.join("ReadableStreamDefaultReaderBinding.rs");
    fs::write(&readable_stream_default_reader_binding, r#"
// Extended stub for ReadableStreamDefaultReaderBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ReadableStreamDefaultReaderMethods<D: DomTypes> {}

pub mod ReadableStreamDefaultReader_Binding {
    use super::*;
    pub use super::ReadableStreamDefaultReaderMethods;
}

#[derive(Clone, Debug)]
pub struct ReadableStreamReadResult {
    pub done: bool,
    pub value: Option<()>,
}

impl Default for ReadableStreamReadResult {
    fn default() -> Self {
        Self { done: false, value: None }
    }
}
"#).unwrap();

    let readable_stream_byob_reader_binding = bindings_dir.join("ReadableStreamBYOBReaderBinding.rs");
    fs::write(&readable_stream_byob_reader_binding, r#"
// Extended stub for ReadableStreamBYOBReaderBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ReadableStreamBYOBReaderMethods<D: DomTypes> {}

pub mod ReadableStreamBYOBReader_Binding {
    use super::*;
    pub use super::ReadableStreamBYOBReaderMethods;
}

#[derive(Clone, Debug, Default)]
pub struct ReadableStreamBYOBReaderReadOptions {
    pub min: Option<u64>,
}
"#).unwrap();

    // Create Worker bindings
    let worker_binding = bindings_dir.join("WorkerBinding.rs");
    fs::write(&worker_binding, r#"
// Extended stub for WorkerBinding

use crate::codegen::DomTypes::DomTypes;

pub trait WorkerMethods<D: DomTypes> {}

pub mod Worker_Binding {
    use super::*;
    pub use super::WorkerMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerType { Classic, Module }

#[derive(Clone, Debug, Default)]
pub struct WorkerOptions {
    pub type_: Option<WorkerType>,
    pub credentials: Option<String>,
    pub name: Option<String>,
}
"#).unwrap();

    let worklet_binding = bindings_dir.join("WorkletBinding.rs");
    fs::write(&worklet_binding, r#"
// Extended stub for WorkletBinding

use crate::codegen::DomTypes::DomTypes;

pub trait WorkletMethods<D: DomTypes> {}

pub mod Worklet_Binding {
    use super::*;
    pub use super::WorkletMethods;
}

#[derive(Clone, Debug, Default)]
pub struct WorkletOptions {
    pub credentials: Option<String>,
}
"#).unwrap();

    // Create Service Worker bindings
    let service_worker_binding = bindings_dir.join("ServiceWorkerBinding.rs");
    fs::write(&service_worker_binding, r#"
// Extended stub for ServiceWorkerBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ServiceWorkerMethods<D: DomTypes> {}

pub mod ServiceWorker_Binding {
    use super::*;
    pub use super::ServiceWorkerMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceWorkerState { Parsed, Installing, Installed, Activating, Activated, Redundant }
"#).unwrap();

    let service_worker_container_binding = bindings_dir.join("ServiceWorkerContainerBinding.rs");
    fs::write(&service_worker_container_binding, r#"
// Extended stub for ServiceWorkerContainerBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ServiceWorkerContainerMethods<D: DomTypes> {}

pub mod ServiceWorkerContainer_Binding {
    use super::*;
    pub use super::ServiceWorkerContainerMethods;
}

#[derive(Clone, Debug, Default)]
pub struct RegistrationOptions {
    pub scope: Option<String>,
    pub type_: Option<String>,
    pub updateViaCache: Option<String>,
}
"#).unwrap();

    let service_worker_registration_binding = bindings_dir.join("ServiceWorkerRegistrationBinding.rs");
    fs::write(&service_worker_registration_binding, r#"
// Extended stub for ServiceWorkerRegistrationBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ServiceWorkerRegistrationMethods<D: DomTypes> {}

pub mod ServiceWorkerRegistration_Binding {
    use super::*;
    pub use super::ServiceWorkerRegistrationMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceWorkerUpdateViaCache { Imports, All, None }
"#).unwrap();

    // Create MediaStream bindings  
    let media_stream_binding = bindings_dir.join("MediaStreamBinding.rs");
    fs::write(&media_stream_binding, r#"
// Extended stub for MediaStreamBinding

use crate::codegen::DomTypes::DomTypes;

pub trait MediaStreamMethods<D: DomTypes> {}

pub mod MediaStream_Binding {
    use super::*;
    pub use super::MediaStreamMethods;
}
"#).unwrap();

    let media_devices_binding = bindings_dir.join("MediaDevicesBinding.rs");
    fs::write(&media_devices_binding, r#"
// Extended stub for MediaDevicesBinding

use crate::codegen::DomTypes::DomTypes;

pub trait MediaDevicesMethods<D: DomTypes> {}

pub mod MediaDevices_Binding {
    use super::*;
    pub use super::MediaDevicesMethods;
}

#[derive(Clone, Debug, Default)]
pub struct MediaStreamConstraints {
    pub audio: Option<bool>,
    pub video: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct DisplayMediaStreamConstraints {
    pub audio: Option<bool>,
    pub video: Option<bool>,
}
"#).unwrap();

    let media_device_info_binding = bindings_dir.join("MediaDeviceInfoBinding.rs");
    fs::write(&media_device_info_binding, r#"
// Extended stub for MediaDeviceInfoBinding

use crate::codegen::DomTypes::DomTypes;

pub trait MediaDeviceInfoMethods<D: DomTypes> {}

pub mod MediaDeviceInfo_Binding {
    use super::*;
    pub use super::MediaDeviceInfoMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaDeviceKind { Audioinput, Audiooutput, Videoinput }
"#).unwrap();

    // Create MediaSource bindings
    let media_source_binding = bindings_dir.join("MediaSourceBinding.rs");
    fs::write(&media_source_binding, r#"
// Extended stub for MediaSourceBinding

use crate::codegen::DomTypes::DomTypes;

pub trait MediaSourceMethods<D: DomTypes> {}

pub mod MediaSource_Binding {
    use super::*;
    pub use super::MediaSourceMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndOfStreamError { Network, Decode }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadyState { Closed, Open, Ended }
"#).unwrap();

    let source_buffer_binding = bindings_dir.join("SourceBufferBinding.rs");
    fs::write(&source_buffer_binding, r#"
// Extended stub for SourceBufferBinding

use crate::codegen::DomTypes::DomTypes;

pub trait SourceBufferMethods<D: DomTypes> {}

pub mod SourceBuffer_Binding {
    use super::*;
    pub use super::SourceBufferMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppendMode { Segments, Sequence }
"#).unwrap();

    // Create Notification binding
    let notification_binding = bindings_dir.join("NotificationBinding.rs");
    fs::write(&notification_binding, r#"
// Extended stub for NotificationBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait NotificationMethods<D: DomTypes> {}

pub mod Notification_Binding {
    use super::*;
    pub use super::NotificationMethods;
}

#[derive(Clone, Debug, Default)]
pub struct NotificationOptions {
    pub dir: Option<String>,
    pub lang: Option<String>,
    pub body: Option<String>,
    pub tag: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub vibrate: Option<Vec<u32>>,
    pub timestamp: Option<f64>,
    pub renotify: Option<bool>,
    pub silent: Option<bool>,
    pub requireInteraction: Option<bool>,
    pub data: Option<()>,
    pub actions: Option<Vec<NotificationAction>>,
}

#[derive(Clone, Debug, Default)]
pub struct NotificationAction {
    pub action: String,
    pub title: String,
    pub icon: Option<String>,
}

pub struct NotificationPermissionCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationDirection { Auto, Ltr, Rtl }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationPermission { Default, Denied, Granted }
"#).unwrap();

    // Create MediaSession binding
    let media_session_binding = bindings_dir.join("MediaSessionBinding.rs");
    fs::write(&media_session_binding, r#"
// Extended stub for MediaSessionBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait MediaSessionMethods<D: DomTypes> {}

pub mod MediaSession_Binding {
    use super::*;
    pub use super::MediaSessionMethods;
}

#[derive(Clone, Debug, Default)]
pub struct MediaPositionState {
    pub duration: Option<f64>,
    pub playbackRate: Option<f64>,
    pub position: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaSessionPlaybackState { None, Paused, Playing }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaSessionAction {
    Play, Pause, Seekbackward, Seekforward, Previoustrack, Nexttrack, Skipad, Stop, Seekto,
    Togglemicrophone, Togglecamera, Hangup, Previousslide, Nextslide,
}

/// MediaSessionActionHandler callback type
pub struct MediaSessionActionHandler<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create MediaMetadataBinding
    let media_metadata_binding = bindings_dir.join("MediaMetadataBinding.rs");
    fs::write(&media_metadata_binding, r#"
// Extended stub for MediaMetadataBinding

use crate::codegen::DomTypes::DomTypes;

pub trait MediaMetadataMethods<D: DomTypes> {}

pub mod MediaMetadata_Binding {
    use super::*;
    pub use super::MediaMetadataMethods;
}

#[derive(Clone, Debug, Default)]
pub struct MediaMetadataInit {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub artwork: Option<Vec<()>>,
}
"#).unwrap();

    // Create MediaError binding
    let media_error_binding = bindings_dir.join("MediaErrorBinding.rs");
    fs::write(&media_error_binding, r#"
// Extended stub for MediaErrorBinding

use crate::codegen::DomTypes::DomTypes;

pub trait MediaErrorMethods<D: DomTypes> {}

pub mod MediaError_Binding {
    use super::*;
    pub use super::MediaErrorMethods;
}

pub mod MediaErrorConstants {
    pub const MEDIA_ERR_ABORTED: u16 = 1;
    pub const MEDIA_ERR_NETWORK: u16 = 2;
    pub const MEDIA_ERR_DECODE: u16 = 3;
    pub const MEDIA_ERR_SRC_NOT_SUPPORTED: u16 = 4;
}
"#).unwrap();

    // Create MediaQueryListEventBinding
    let media_query_list_event_binding = bindings_dir.join("MediaQueryListEventBinding.rs");
    fs::write(&media_query_list_event_binding, r#"
// Extended stub for MediaQueryListEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait MediaQueryListEventMethods<D: DomTypes> {}

pub mod MediaQueryListEvent_Binding {
    use super::*;
    pub use super::MediaQueryListEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct MediaQueryListEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub media: String,
    pub matches: bool,
}
"#).unwrap();

    // Create OfflineAudioContext bindings
    let offline_audio_context_binding = bindings_dir.join("OfflineAudioContextBinding.rs");
    fs::write(&offline_audio_context_binding, r#"
// Extended stub for OfflineAudioContextBinding

use crate::codegen::DomTypes::DomTypes;

pub trait OfflineAudioContextMethods<D: DomTypes> {}

pub mod OfflineAudioContext_Binding {
    use super::*;
    pub use super::OfflineAudioContextMethods;
}

#[derive(Clone, Debug, Default)]
pub struct OfflineAudioContextOptions {
    pub numberOfChannels: u32,
    pub length: u32,
    pub sampleRate: f32,
}
"#).unwrap();

    let offline_audio_completion_event_binding = bindings_dir.join("OfflineAudioCompletionEventBinding.rs");
    fs::write(&offline_audio_completion_event_binding, r#"
// Extended stub for OfflineAudioCompletionEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait OfflineAudioCompletionEventMethods<D: DomTypes> {}

pub mod OfflineAudioCompletionEvent_Binding {
    use super::*;
    pub use super::OfflineAudioCompletionEventMethods;
}

#[derive(Clone, Debug, Default)]
pub struct OfflineAudioCompletionEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub renderedBuffer: (),
}
"#).unwrap();

    // Create MessagePortBinding with options
    let message_port_binding = bindings_dir.join("MessagePortBinding.rs");
    fs::write(&message_port_binding, r#"
// Extended stub for MessagePortBinding

use crate::codegen::DomTypes::DomTypes;

pub trait MessagePortMethods<D: DomTypes> {}

pub mod MessagePort_Binding {
    use super::*;
    pub use super::MessagePortMethods;
}

#[derive(Clone, Debug, Default)]
pub struct StructuredSerializeOptions {
    pub transfer: Vec<()>,
}
"#).unwrap();

    // Create NavigationPreloadManagerBinding
    let navigation_preload_binding = bindings_dir.join("NavigationPreloadManagerBinding.rs");
    fs::write(&navigation_preload_binding, r#"
// Extended stub for NavigationPreloadManagerBinding

use crate::codegen::DomTypes::DomTypes;

pub trait NavigationPreloadManagerMethods<D: DomTypes> {}

pub mod NavigationPreloadManager_Binding {
    use super::*;
    pub use super::NavigationPreloadManagerMethods;
}

#[derive(Clone, Debug, Default)]
pub struct NavigationPreloadState {
    pub enabled: bool,
    pub headerValue: Option<String>,
}
"#).unwrap();

    // Create TextDecoderBinding
    let text_decoder_binding = bindings_dir.join("TextDecoderBinding.rs");
    fs::write(&text_decoder_binding, r#"
// Extended stub for TextDecoderBinding

use crate::codegen::DomTypes::DomTypes;

pub trait TextDecoderMethods<D: DomTypes> {}

pub mod TextDecoder_Binding {
    use super::*;
    pub use super::TextDecoderMethods;
}

#[derive(Clone, Debug, Default)]
pub struct TextDecodeOptions {
    pub stream: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TextDecoderOptions {
    pub fatal: bool,
    pub ignoreBOM: bool,
}
"#).unwrap();

    // Create TextEncoderBinding
    let text_encoder_binding = bindings_dir.join("TextEncoderBinding.rs");
    fs::write(&text_encoder_binding, r#"
// Extended stub for TextEncoderBinding

use crate::codegen::DomTypes::DomTypes;

pub trait TextEncoderMethods<D: DomTypes> {}

pub mod TextEncoder_Binding {
    use super::*;
    pub use super::TextEncoderMethods;
}

#[derive(Clone, Debug, Default)]
pub struct TextEncoderEncodeIntoResult {
    pub read: u64,
    pub written: u64,
}
"#).unwrap();

    // Create TextTrackBinding
    let text_track_binding = bindings_dir.join("TextTrackBinding.rs");
    fs::write(&text_track_binding, r#"
// Extended stub for TextTrackBinding

use crate::codegen::DomTypes::DomTypes;

pub trait TextTrackMethods<D: DomTypes> {}

pub mod TextTrack_Binding {
    use super::*;
    pub use super::TextTrackMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextTrackKind { Subtitles, Captions, Descriptions, Chapters, Metadata }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextTrackMode { Disabled, Hidden, Showing }
"#).unwrap();

    // Create VTTCueBinding
    let vtt_cue_binding = bindings_dir.join("VTTCueBinding.rs");
    fs::write(&vtt_cue_binding, r#"
// Extended stub for VTTCueBinding

use crate::codegen::DomTypes::DomTypes;

pub trait VTTCueMethods<D: DomTypes> {}

pub mod VTTCue_Binding {
    use super::*;
    pub use super::VTTCueMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignSetting { Start, Center, End, Left, Right }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineAlignSetting { Start, Center, End }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionAlignSetting { LineLeft, Center, LineRight, Auto }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollSetting { Empty, Up }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectionSetting { Empty, Rl, Lr }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoKeyword { Auto }
"#).unwrap();

    // Create VTTRegionBinding
    let vtt_region_binding = bindings_dir.join("VTTRegionBinding.rs");
    fs::write(&vtt_region_binding, r#"
// Extended stub for VTTRegionBinding

use crate::codegen::DomTypes::DomTypes;

pub trait VTTRegionMethods<D: DomTypes> {}

pub mod VTTRegion_Binding {
    use super::*;
    pub use super::VTTRegionMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollSetting { Empty, Up }
"#).unwrap();

    // Create StaticRangeBinding
    let static_range_binding = bindings_dir.join("StaticRangeBinding.rs");
    fs::write(&static_range_binding, r#"
// Extended stub for StaticRangeBinding

use crate::codegen::DomTypes::DomTypes;

pub trait StaticRangeMethods<D: DomTypes> {}

pub mod StaticRange_Binding {
    use super::*;
    pub use super::StaticRangeMethods;
}

#[derive(Clone, Debug, Default)]
pub struct StaticRangeInit {
    pub startContainer: (),
    pub startOffset: u32,
    pub endContainer: (),
    pub endOffset: u32,
}
"#).unwrap();

    // Create XPathResultBinding with constants
    let xpath_result_binding = bindings_dir.join("XPathResultBinding.rs");
    fs::write(&xpath_result_binding, r#"
// Extended stub for XPathResultBinding

use crate::codegen::DomTypes::DomTypes;

pub trait XPathResultMethods<D: DomTypes> {}

pub mod XPathResult_Binding {
    use super::*;
    pub use super::XPathResultMethods;
}

pub struct XPathResultConstants;
impl XPathResultConstants {
    pub const ANY_TYPE: u16 = 0;
    pub const NUMBER_TYPE: u16 = 1;
    pub const STRING_TYPE: u16 = 2;
    pub const BOOLEAN_TYPE: u16 = 3;
    pub const UNORDERED_NODE_ITERATOR_TYPE: u16 = 4;
    pub const ORDERED_NODE_ITERATOR_TYPE: u16 = 5;
    pub const UNORDERED_NODE_SNAPSHOT_TYPE: u16 = 6;
    pub const ORDERED_NODE_SNAPSHOT_TYPE: u16 = 7;
    pub const ANY_UNORDERED_NODE_TYPE: u16 = 8;
    pub const FIRST_ORDERED_NODE_TYPE: u16 = 9;
}
"#).unwrap();

    // Create WebSocketBinding
    let websocket_binding = bindings_dir.join("WebSocketBinding.rs");
    fs::write(&websocket_binding, r#"
// Extended stub for WebSocketBinding

use crate::codegen::DomTypes::DomTypes;

pub trait WebSocketMethods<D: DomTypes> {}

pub mod WebSocket_Binding {
    use super::*;
    pub use super::WebSocketMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryType { Blob, Arraybuffer }
"#).unwrap();

    // Create TrustedTypePolicyFactoryBinding
    let trusted_type_policy_factory_binding = bindings_dir.join("TrustedTypePolicyFactoryBinding.rs");
    fs::write(&trusted_type_policy_factory_binding, r#"
// Extended stub for TrustedTypePolicyFactoryBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait TrustedTypePolicyFactoryMethods<D: DomTypes> {}

pub mod TrustedTypePolicyFactory_Binding {
    use super::*;
    pub use super::TrustedTypePolicyFactoryMethods;
}

#[derive(Clone, Debug, Default)]
pub struct TrustedTypePolicyOptions {
    pub createHTML: Option<()>,
    pub createScript: Option<()>,
    pub createScriptURL: Option<()>,
}

pub struct CreateHTMLCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct CreateScriptCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct CreateScriptURLCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create URLPatternBinding
    let url_pattern_binding = bindings_dir.join("URLPatternBinding.rs");
    fs::write(&url_pattern_binding, r#"
// Extended stub for URLPatternBinding

use crate::codegen::DomTypes::DomTypes;

pub trait URLPatternMethods<D: DomTypes> {}

pub mod URLPattern_Binding {
    use super::*;
    pub use super::URLPatternMethods;
}

#[derive(Clone, Debug, Default)]
pub struct URLPatternInit {
    pub protocol: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<String>,
    pub pathname: Option<String>,
    pub search: Option<String>,
    pub hash: Option<String>,
    pub baseURL: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct URLPatternResult {
    pub inputs: Vec<()>,
    pub protocol: Option<URLPatternComponentResult>,
    pub username: Option<URLPatternComponentResult>,
    pub password: Option<URLPatternComponentResult>,
    pub hostname: Option<URLPatternComponentResult>,
    pub port: Option<URLPatternComponentResult>,
    pub pathname: Option<URLPatternComponentResult>,
    pub search: Option<URLPatternComponentResult>,
    pub hash: Option<URLPatternComponentResult>,
}

#[derive(Clone, Debug, Default)]
pub struct URLPatternComponentResult {
    pub input: String,
    pub groups: Vec<(String, String)>,
}
"#).unwrap();

    // Create ReportingObserverBinding
    let reporting_observer_binding = bindings_dir.join("ReportingObserverBinding.rs");
    fs::write(&reporting_observer_binding, r#"
// Extended stub for ReportingObserverBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait ReportingObserverMethods<D: DomTypes> {}

pub mod ReportingObserver_Binding {
    use super::*;
    pub use super::ReportingObserverMethods;
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub type_: String,
    pub url: String,
    pub body: Option<ReportBody>,
}

#[derive(Clone, Debug, Default)]
pub struct ReportBody;

#[derive(Clone, Debug, Default)]
pub struct ReportingObserverOptions {
    pub types: Option<Vec<String>>,
    pub buffered: Option<bool>,
}

/// ReportList type alias
pub type ReportList = Vec<Report>;

/// ReportingObserverCallback type
pub struct ReportingObserverCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create QuotaExceededErrorBinding
    let quota_exceeded_error_binding = bindings_dir.join("QuotaExceededErrorBinding.rs");
    fs::write(&quota_exceeded_error_binding, r#"
// Extended stub for QuotaExceededErrorBinding

use crate::codegen::DomTypes::DomTypes;

pub trait QuotaExceededErrorMethods<D: DomTypes> {}

pub mod QuotaExceededError_Binding {
    use super::*;
    pub use super::QuotaExceededErrorMethods;
}

#[derive(Clone, Debug, Default)]
pub struct QuotaExceededErrorOptions {
    pub quota: Option<u64>,
    pub usage: Option<u64>,
}
"#).unwrap();

    // Create SecurityPolicyViolationEventBinding
    let security_policy_violation_event_binding = bindings_dir.join("SecurityPolicyViolationEventBinding.rs");
    fs::write(&security_policy_violation_event_binding, r#"
// Extended stub for SecurityPolicyViolationEventBinding

use crate::codegen::DomTypes::DomTypes;

pub trait SecurityPolicyViolationEventMethods<D: DomTypes> {}

pub mod SecurityPolicyViolationEvent_Binding {
    use super::*;
    pub use super::SecurityPolicyViolationEventMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityPolicyViolationEventDisposition { Enforce, Report }

#[derive(Clone, Debug, Default)]
pub struct SecurityPolicyViolationEventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub documentURI: String,
    pub referrer: String,
    pub blockedURI: String,
    pub violatedDirective: String,
    pub effectiveDirective: String,
    pub originalPolicy: String,
    pub sourceFile: String,
    pub sample: String,
    pub disposition: Option<SecurityPolicyViolationEventDisposition>,
    pub statusCode: u16,
    pub lineNumber: u32,
    pub columnNumber: u32,
}
"#).unwrap();

    // Create CSPViolationReportBodyBinding
    let csp_violation_report_body_binding = bindings_dir.join("CSPViolationReportBodyBinding.rs");
    fs::write(&csp_violation_report_body_binding, r#"
// Extended stub for CSPViolationReportBodyBinding

use crate::codegen::DomTypes::DomTypes;

pub trait CSPViolationReportBodyMethods<D: DomTypes> {}

pub mod CSPViolationReportBody_Binding {
    use super::*;
    pub use super::CSPViolationReportBodyMethods;
}

#[derive(Clone, Debug, Default)]
pub struct CSPViolationReportBody {
    pub documentURL: String,
    pub referrer: Option<String>,
    pub blockedURL: Option<String>,
    pub effectiveDirective: String,
    pub originalPolicy: String,
    pub sourceFile: Option<String>,
    pub sample: Option<String>,
    pub disposition: String,
    pub statusCode: u16,
    pub lineNumber: Option<u32>,
    pub columnNumber: Option<u32>,
}
"#).unwrap();

    // Create PromiseBinding
    let promise_binding = bindings_dir.join("PromiseBinding.rs");
    fs::write(&promise_binding, r#"
// Extended stub for PromiseBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait PromiseMethods<D: DomTypes> {}

pub mod Promise_Binding {
    use super::*;
    pub use super::PromiseMethods;
}

pub struct PromiseJobCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create VoidFunctionBinding
    let void_function_binding = bindings_dir.join("VoidFunctionBinding.rs");
    fs::write(&void_function_binding, r#"
// Extended stub for VoidFunctionBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait VoidFunctionMethods<D: DomTypes> {}

pub mod VoidFunction_Binding {
    use super::*;
    pub use super::VoidFunctionMethods;
}

pub struct VoidFunction<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create UnderlyingSourceBinding
    let underlying_source_binding = bindings_dir.join("UnderlyingSourceBinding.rs");
    fs::write(&underlying_source_binding, r#"
// Extended stub for UnderlyingSourceBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait UnderlyingSourceMethods<D: DomTypes> {}

pub mod UnderlyingSource_Binding {
    use super::*;
    pub use super::UnderlyingSourceMethods;
}

#[derive(Clone, Debug, Default)]
pub struct UnderlyingSource;

pub struct UnderlyingSourceStartCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct UnderlyingSourcePullCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct UnderlyingSourceCancelCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create UnderlyingSinkBinding
    let underlying_sink_binding = bindings_dir.join("UnderlyingSinkBinding.rs");
    fs::write(&underlying_sink_binding, r#"
// Extended stub for UnderlyingSinkBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait UnderlyingSinkMethods<D: DomTypes> {}

pub mod UnderlyingSink_Binding {
    use super::*;
    pub use super::UnderlyingSinkMethods;
}

#[derive(Clone, Debug, Default)]
pub struct UnderlyingSink;

pub struct UnderlyingSinkStartCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct UnderlyingSinkWriteCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct UnderlyingSinkCloseCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct UnderlyingSinkAbortCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create TransformerBinding
    let transformer_binding = bindings_dir.join("TransformerBinding.rs");
    fs::write(&transformer_binding, r#"
// Extended stub for TransformerBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait TransformerMethods<D: DomTypes> {}

pub mod Transformer_Binding {
    use super::*;
    pub use super::TransformerMethods;
}

#[derive(Clone, Debug, Default)]
pub struct Transformer;

pub struct TransformerStartCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct TransformerFlushCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct TransformerTransformCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}

pub struct TransformerCancelCallback<D: DomTypes> {
    _marker: PhantomData<D>,
}
"#).unwrap();

    // Create WebGL extension bindings
    let angle_instanced_arrays_binding = bindings_dir.join("ANGLEInstancedArraysBinding.rs");
    fs::write(&angle_instanced_arrays_binding, r#"
// Extended stub for ANGLEInstancedArraysBinding

use crate::codegen::DomTypes::DomTypes;

pub trait ANGLEInstancedArraysMethods<D: DomTypes> {}

pub mod ANGLEInstancedArrays_Binding {
    use super::*;
    pub use super::ANGLEInstancedArraysMethods;
}

pub struct ANGLEInstancedArraysConstants;
impl ANGLEInstancedArraysConstants {
    pub const VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE: u32 = 0x88FE;
}
"#).unwrap();

    let ext_blend_minmax_binding = bindings_dir.join("EXTBlendMinmaxBinding.rs");
    fs::write(&ext_blend_minmax_binding, r#"
// Extended stub for EXTBlendMinmaxBinding

use crate::codegen::DomTypes::DomTypes;

pub trait EXTBlendMinmaxMethods<D: DomTypes> {}

pub mod EXTBlendMinmax_Binding {
    use super::*;
    pub use super::EXTBlendMinmaxMethods;
}

pub struct EXTBlendMinmaxConstants;
impl EXTBlendMinmaxConstants {
    pub const MIN_EXT: u32 = 0x8007;
    pub const MAX_EXT: u32 = 0x8008;
}
"#).unwrap();

    let ext_color_buffer_half_float_binding = bindings_dir.join("EXTColorBufferHalfFloatBinding.rs");
    fs::write(&ext_color_buffer_half_float_binding, r#"
// Extended stub for EXTColorBufferHalfFloatBinding

use crate::codegen::DomTypes::DomTypes;

pub trait EXTColorBufferHalfFloatMethods<D: DomTypes> {}

pub mod EXTColorBufferHalfFloat_Binding {
    use super::*;
    pub use super::EXTColorBufferHalfFloatMethods;
}

pub struct EXTColorBufferHalfFloatConstants;
impl EXTColorBufferHalfFloatConstants {
    pub const RGBA16F_EXT: u32 = 0x881A;
    pub const RGB16F_EXT: u32 = 0x881B;
    pub const FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT: u32 = 0x8211;
    pub const UNSIGNED_NORMALIZED_EXT: u32 = 0x8C17;
}
"#).unwrap();

    let ext_texture_filter_anisotropic_binding = bindings_dir.join("EXTTextureFilterAnisotropicBinding.rs");
    fs::write(&ext_texture_filter_anisotropic_binding, r#"
// Extended stub for EXTTextureFilterAnisotropicBinding

use crate::codegen::DomTypes::DomTypes;

pub trait EXTTextureFilterAnisotropicMethods<D: DomTypes> {}

pub mod EXTTextureFilterAnisotropic_Binding {
    use super::*;
    pub use super::EXTTextureFilterAnisotropicMethods;
}

pub struct EXTTextureFilterAnisotropicConstants;
impl EXTTextureFilterAnisotropicConstants {
    pub const TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
    pub const MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;
}
"#).unwrap();

    let oes_standard_derivatives_binding = bindings_dir.join("OESStandardDerivativesBinding.rs");
    fs::write(&oes_standard_derivatives_binding, r#"
// Extended stub for OESStandardDerivativesBinding

use crate::codegen::DomTypes::DomTypes;

pub trait OESStandardDerivativesMethods<D: DomTypes> {}

pub mod OESStandardDerivatives_Binding {
    use super::*;
    pub use super::OESStandardDerivativesMethods;
}

pub struct OESStandardDerivativesConstants;
impl OESStandardDerivativesConstants {
    pub const FRAGMENT_SHADER_DERIVATIVE_HINT_OES: u32 = 0x8B8B;
}
"#).unwrap();

    let oes_texture_half_float_binding = bindings_dir.join("OESTextureHalfFloatBinding.rs");
    fs::write(&oes_texture_half_float_binding, r#"
// Extended stub for OESTextureHalfFloatBinding

use crate::codegen::DomTypes::DomTypes;

pub trait OESTextureHalfFloatMethods<D: DomTypes> {}

pub mod OESTextureHalfFloat_Binding {
    use super::*;
    pub use super::OESTextureHalfFloatMethods;
}

pub struct OESTextureHalfFloatConstants;
impl OESTextureHalfFloatConstants {
    pub const HALF_FLOAT_OES: u32 = 0x8D61;
}
"#).unwrap();

    let oes_vertex_array_object_binding = bindings_dir.join("OESVertexArrayObjectBinding.rs");
    fs::write(&oes_vertex_array_object_binding, r#"
// Extended stub for OESVertexArrayObjectBinding

use crate::codegen::DomTypes::DomTypes;

pub trait OESVertexArrayObjectMethods<D: DomTypes> {}

pub mod OESVertexArrayObject_Binding {
    use super::*;
    pub use super::OESVertexArrayObjectMethods;
}

pub struct OESVertexArrayObjectConstants;
impl OESVertexArrayObjectConstants {
    pub const VERTEX_ARRAY_BINDING_OES: u32 = 0x85B5;
}
"#).unwrap();

    let webgl_color_buffer_float_binding = bindings_dir.join("WEBGLColorBufferFloatBinding.rs");
    fs::write(&webgl_color_buffer_float_binding, r#"
// Extended stub for WEBGLColorBufferFloatBinding

use crate::codegen::DomTypes::DomTypes;

pub trait WEBGLColorBufferFloatMethods<D: DomTypes> {}

pub mod WEBGLColorBufferFloat_Binding {
    use super::*;
    pub use super::WEBGLColorBufferFloatMethods;
}

pub struct WEBGLColorBufferFloatConstants;
impl WEBGLColorBufferFloatConstants {
    pub const RGBA32F_EXT: u32 = 0x8814;
    pub const FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT: u32 = 0x8211;
    pub const UNSIGNED_NORMALIZED_EXT: u32 = 0x8C17;
}
"#).unwrap();

    // Create SubtleCryptoBinding
    let subtle_crypto_binding = bindings_dir.join("SubtleCryptoBinding.rs");
    fs::write(&subtle_crypto_binding, r#"
// Extended stub for SubtleCryptoBinding

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

pub trait SubtleCryptoMethods<D: DomTypes> {}

pub mod SubtleCrypto_Binding {
    use super::*;
    pub use super::SubtleCryptoMethods;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyFormat { Raw, Spki, Pkcs8, Jwk }

#[derive(Clone, Debug, Default)]
pub struct Algorithm {
    pub name: String,
}

pub type AlgorithmIdentifier = Algorithm;

#[derive(Clone, Debug, Default)]
pub struct AesCbcParams {
    pub name: String,
    pub iv: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct AesCtrParams {
    pub name: String,
    pub counter: Vec<u8>,
    pub length: u8,
}

#[derive(Clone, Debug, Default)]
pub struct AesDerivedKeyParams {
    pub name: String,
    pub length: u16,
}

#[derive(Clone, Debug, Default)]
pub struct AesGcmParams {
    pub name: String,
    pub iv: Vec<u8>,
    pub additionalData: Option<Vec<u8>>,
    pub tagLength: Option<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct AesKeyAlgorithm {
    pub name: String,
    pub length: u16,
}

#[derive(Clone, Debug, Default)]
pub struct AesKeyGenParams {
    pub name: String,
    pub length: u16,
}

#[derive(Clone, Debug, Default)]
pub struct Argon2Params {
    pub name: String,
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub parallelism: u32,
    pub memory: u32,
}

#[derive(Clone, Debug, Default)]
pub struct CShakeParams {
    pub name: String,
    pub outputLength: Option<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct EcKeyAlgorithm {
    pub name: String,
    pub namedCurve: String,
}

#[derive(Clone, Debug, Default)]
pub struct EcKeyGenParams {
    pub name: String,
    pub namedCurve: String,
}

#[derive(Clone, Debug, Default)]
pub struct EcKeyImportParams {
    pub name: String,
    pub namedCurve: String,
}

#[derive(Clone, Debug, Default)]
pub struct EcdhKeyDeriveParams {
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct EcdsaParams {
    pub name: String,
    pub hash: String,
}

#[derive(Clone, Debug, Default)]
pub struct HkdfParams {
    pub name: String,
    pub hash: String,
    pub salt: Vec<u8>,
    pub info: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct HmacImportParams {
    pub name: String,
    pub hash: String,
    pub length: Option<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct HmacKeyAlgorithm {
    pub name: String,
    pub hash: String,
    pub length: u32,
}

#[derive(Clone, Debug, Default)]
pub struct HmacKeyGenParams {
    pub name: String,
    pub hash: String,
    pub length: Option<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct KeyAlgorithm {
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct Pbkdf2Params {
    pub name: String,
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub hash: String,
}

#[derive(Clone, Debug, Default)]
pub struct RsaOaepParams {
    pub name: String,
    pub label: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Default)]
pub struct RsaOtherPrimesInfo {
    pub r: String,
    pub d: String,
    pub t: String,
}

#[derive(Clone, Debug, Default)]
pub struct JsonWebKey {
    pub kty: String,
    pub use_: Option<String>,
    pub key_ops: Option<Vec<String>>,
    pub alg: Option<String>,
    pub ext: Option<bool>,
    pub crv: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub d: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    pub p: Option<String>,
    pub q: Option<String>,
    pub dp: Option<String>,
    pub dq: Option<String>,
    pub qi: Option<String>,
    pub oth: Option<Vec<RsaOtherPrimesInfo>>,
    pub k: Option<String>,
}
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

/// AudioNodeTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioNodeTypeId {
    AudioNode,
    AudioScheduledSourceNode(AudioScheduledSourceNodeTypeId),
    AnalyserNode,
    GainNode,
    BiquadFilterNode,
    ChannelMergerNode,
    ChannelSplitterNode,
    ConvolverNode,
    DelayNode,
    DynamicsCompressorNode,
    PannerNode,
    StereoPannerNode,
    WaveShaperNode,
    AudioDestinationNode,
    MediaElementAudioSourceNode,
    MediaStreamAudioSourceNode,
    MediaStreamAudioDestinationNode,
}

/// AudioScheduledSourceNodeTypeId
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioScheduledSourceNodeTypeId {
    AudioScheduledSourceNode,
    OscillatorNode,
    AudioBufferSourceNode,
    ConstantSourceNode,
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

/// Constructor IDs (mirrors ID but for constructors)
pub type Constructor = ID;

/// Convert prototype ID to string name
pub fn proto_id_to_name(id: u16) -> &'static str {
    // Just return a placeholder - real implementation would match on ID
    "Unknown"
}
"#).unwrap();

    // Create DomTypes.rs stub with associated types but minimal bounds
    let dom_types_rs = out_dir.join("DomTypes.rs");
    fs::write(&dom_types_rs, r#"
// Auto-generated stub for Boa engine - DomTypes trait with associated types

/// Placeholder trait for DOM objects 
pub trait DomObjectPlaceholder: crate::reflector::DomObject {}

/// DomTypes marker trait with all required associated types for DOM interfaces
/// This version has minimal bounds for Boa compatibility
pub trait DomTypes: Sized + 'static {
    // Core DOM types
    type GlobalScope: DomObjectPlaceholder;
    type Window: DomObjectPlaceholder;
    type WindowProxy: DomObjectPlaceholder;
    type Document: DomObjectPlaceholder;
    type DocumentFragment: DomObjectPlaceholder;
    type DocumentType: DomObjectPlaceholder;
    type Element: DomObjectPlaceholder;
    type Node: DomObjectPlaceholder;
    type Comment: DomObjectPlaceholder;
    type Text: DomObjectPlaceholder;
    type CharacterData: DomObjectPlaceholder;
    type Attr: DomObjectPlaceholder;
    type Event: DomObjectPlaceholder;
    type EventTarget: DomObjectPlaceholder;
    type ShadowRoot: DomObjectPlaceholder;

    // Collections and lists
    type NodeList: DomObjectPlaceholder;
    type HTMLCollection: DomObjectPlaceholder;
    type HTMLOptionsCollection: DomObjectPlaceholder;
    type HTMLFormControlsCollection: DomObjectPlaceholder;
    type DOMTokenList: DomObjectPlaceholder;
    type NamedNodeMap: DomObjectPlaceholder;
    type FileList: DomObjectPlaceholder;
    type File: DomObjectPlaceholder;
    type Blob: DomObjectPlaceholder;

    // Canvas and graphics
    type CanvasRenderingContext2D: DomObjectPlaceholder;
    type WebGLRenderingContext: DomObjectPlaceholder;
    type WebGL2RenderingContext: DomObjectPlaceholder;
    type ImageData: DomObjectPlaceholder;

    // URL and Location
    type URL: DomObjectPlaceholder;
    type Location: DomObjectPlaceholder;
    type History: DomObjectPlaceholder;
    type Navigator: DomObjectPlaceholder;

    // Storage
    type Storage: DomObjectPlaceholder;

    // XMLHttpRequest and Fetch
    type XMLHttpRequest: DomObjectPlaceholder;
    type Request: DomObjectPlaceholder;
    type Response: DomObjectPlaceholder;
    type Headers: DomObjectPlaceholder;

    // Events
    type KeyboardEvent: DomObjectPlaceholder;
    type MouseEvent: DomObjectPlaceholder;
    type UIEvent: DomObjectPlaceholder;
    type FocusEvent: DomObjectPlaceholder;
    type WheelEvent: DomObjectPlaceholder;
    type InputEvent: DomObjectPlaceholder;
    type TouchEvent: DomObjectPlaceholder;
    type PointerEvent: DomObjectPlaceholder;
    type CompositionEvent: DomObjectPlaceholder;
    type ClipboardEvent: DomObjectPlaceholder;
    type DragEvent: DomObjectPlaceholder;
    type AnimationEvent: DomObjectPlaceholder;
    type TransitionEvent: DomObjectPlaceholder;
    type MessageEvent: DomObjectPlaceholder;
    type ErrorEvent: DomObjectPlaceholder;
    type ProgressEvent: DomObjectPlaceholder;
    type CustomEvent: DomObjectPlaceholder;

    // HTML Elements
    type HTMLElement: DomObjectPlaceholder;
    type HTMLFormElement: DomObjectPlaceholder;
    type HTMLInputElement: DomObjectPlaceholder;
    type HTMLButtonElement: DomObjectPlaceholder;
    type HTMLSelectElement: DomObjectPlaceholder;
    type HTMLTextAreaElement: DomObjectPlaceholder;
    type HTMLAnchorElement: DomObjectPlaceholder;
    type HTMLImageElement: DomObjectPlaceholder;
    type HTMLScriptElement: DomObjectPlaceholder;
    type HTMLStyleElement: DomObjectPlaceholder;
    type HTMLLinkElement: DomObjectPlaceholder;
    type HTMLDivElement: DomObjectPlaceholder;
    type HTMLSpanElement: DomObjectPlaceholder;
    type HTMLParagraphElement: DomObjectPlaceholder;
    type HTMLHeadingElement: DomObjectPlaceholder;
    type HTMLBodyElement: DomObjectPlaceholder;
    type HTMLHtmlElement: DomObjectPlaceholder;
    type HTMLHeadElement: DomObjectPlaceholder;
    type HTMLTableElement: DomObjectPlaceholder;
    type HTMLTableRowElement: DomObjectPlaceholder;
    type HTMLTableCellElement: DomObjectPlaceholder;
    type HTMLCanvasElement: DomObjectPlaceholder;
    type HTMLVideoElement: DomObjectPlaceholder;
    type HTMLAudioElement: DomObjectPlaceholder;
    type HTMLMediaElement: DomObjectPlaceholder;
    type HTMLIFrameElement: DomObjectPlaceholder;
    type HTMLTemplateElement: DomObjectPlaceholder;
    type HTMLSlotElement: DomObjectPlaceholder;

    // CSS
    type CSSStyleDeclaration: DomObjectPlaceholder;
    type StyleSheet: DomObjectPlaceholder;
    type CSSStyleSheet: DomObjectPlaceholder;
    type CSSRule: DomObjectPlaceholder;
    type CSSStyleRule: DomObjectPlaceholder;

    // Ranges and selections
    type Range: DomObjectPlaceholder;
    type Selection: DomObjectPlaceholder;

    // Workers
    type Worker: DomObjectPlaceholder;
    type WorkerGlobalScope: DomObjectPlaceholder;
    type DedicatedWorkerGlobalScope: DomObjectPlaceholder;
    type ServiceWorkerGlobalScope: DomObjectPlaceholder;

    // Other
    type DOMParser: DomObjectPlaceholder;
    type XMLSerializer: DomObjectPlaceholder;
    type TreeWalker: DomObjectPlaceholder;
    type NodeIterator: DomObjectPlaceholder;
    type MutationObserver: DomObjectPlaceholder;
    type IntersectionObserver: DomObjectPlaceholder;
    type ResizeObserver: DomObjectPlaceholder;
    type Performance: DomObjectPlaceholder;
    type PerformanceEntry: DomObjectPlaceholder;
    type CustomElementRegistry: DomObjectPlaceholder;
    type Animation: DomObjectPlaceholder;
    type DOMException: DomObjectPlaceholder;
    type MessageChannel: DomObjectPlaceholder;
    type MessagePort: DomObjectPlaceholder;
    type AbortController: DomObjectPlaceholder;
    type AbortSignal: DomObjectPlaceholder;
    type Promise: DomObjectPlaceholder;
}
"#).unwrap();

    // Create GenericUnionTypes.rs stub
    let generic_union_types_rs = out_dir.join("GenericUnionTypes.rs");
    fs::write(&generic_union_types_rs, r#"
// Auto-generated stub for Boa engine
// Union types for DOM APIs

use std::marker::PhantomData;
use crate::codegen::DomTypes::DomTypes;

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
pub enum BlobOrString<D: DomTypes> {
    Blob(PhantomData<D>),
    String(String),
}

/// Node or string
pub enum NodeOrString<D: DomTypes> {
    Node(PhantomData<D>),
    String(String),
}

/// Element or string
pub enum ElementOrString<D: DomTypes> {
    Element(PhantomData<D>),
    String(String),
}

// Canvas-related unions
pub enum StringOrCanvasGradientOrCanvasPattern<D: DomTypes> {
    String(String),
    CanvasGradient(PhantomData<D>),
    CanvasPattern(PhantomData<D>),
}

pub enum HTMLCanvasElementOrOffscreenCanvas<D: DomTypes> {
    HTMLCanvasElement(PhantomData<D>),
    OffscreenCanvas(PhantomData<D>),
}

// ArrayBuffer unions
pub enum ArrayBufferOrArrayBufferViewOrBlobOrString<D: DomTypes> {
    ArrayBuffer(Vec<u8>),
    ArrayBufferView(Vec<u8>),
    Blob(PhantomData<D>),
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

// Media unions
pub enum MediaListOrString {
    MediaList(()),
    String(String),
}

// Trusted Types unions
pub enum TrustedHTMLOrTrustedScriptOrTrustedScriptURLOrString {
    TrustedHTML(String),
    TrustedScript(String),
    TrustedScriptURL(String),
    String(String),
}

// Additional missing union types
pub enum AddEventListenerOptionsOrBoolean<D: DomTypes> {
    AddEventListenerOptions(PhantomData<D>),
    Boolean(bool),
}

pub enum EventListenerOptionsOrBoolean<D: DomTypes> {
    EventListenerOptions(PhantomData<D>),
    Boolean(bool),
}

pub enum BooleanOrScrollIntoViewOptions {
    Boolean(bool),
    ScrollIntoViewOptions(()),
}

pub enum EventOrString<D: DomTypes> {
    Event(PhantomData<D>),
    String(String),
}

pub enum StringOrElementCreationOptions {
    String(String),
    ElementCreationOptions(()),
}

pub enum TrustedHTMLOrString {
    TrustedHTML(String),
    String(String),
}

pub enum TrustedHTMLOrNullIsEmptyString {
    TrustedHTML(String),
    NullIsEmptyString(Option<String>),
}

pub enum TrustedScriptURLOrUSVString {
    TrustedScriptURL(String),
    USVString(String),
}

pub enum TrustedScriptOrString {
    TrustedScript(String),
    String(String),
}

pub enum FileOrUSVString<D: DomTypes> {
    File(PhantomData<D>),
    USVString(String),
}

pub enum FileOrUSVStringOrFormData<D: DomTypes> {
    File(PhantomData<D>),
    USVString(String),
    FormData(PhantomData<D>),
}

pub enum StringOrObject {
    String(String),
    Object(()),
}

pub enum StringOrDouble {
    String(String),
    Double(f64),
}

pub enum DoubleOrDoubleSequence {
    Double(f64),
    DoubleSequence(Vec<f64>),
}

pub enum StringOrUnrestrictedDoubleSequence {
    String(String),
    UnrestrictedDoubleSequence(Vec<f64>),
}

pub enum Float32ArrayOrUnrestrictedFloatSequence {
    Float32Array(Vec<f32>),
    UnrestrictedFloatSequence(Vec<f32>),
}

pub enum UnsignedLongOrUnsignedLongSequence {
    UnsignedLong(u32),
    UnsignedLongSequence(Vec<u32>),
}

pub enum ArrayBufferViewOrArrayBufferOrJsonWebKey {
    ArrayBufferView(Vec<u8>),
    ArrayBuffer(Vec<u8>),
    JsonWebKey(()),
}

pub enum AudioContextLatencyCategoryOrDouble {
    AudioContextLatencyCategory(()),
    Double(f64),
}

pub enum BooleanOrMediaTrackConstraints {
    Boolean(bool),
    MediaTrackConstraints(()),
}

pub enum DocumentOrBlobOrArrayBufferViewOrArrayBufferOrFormDataOrStringOrURLSearchParams<D: DomTypes> {
    Document(PhantomData<D>),
    Blob(PhantomData<D>),
    ArrayBufferView(Vec<u8>),
    ArrayBuffer(Vec<u8>),
    FormData(PhantomData<D>),
    String(String),
    URLSearchParams(PhantomData<D>),
}

pub enum ElementOrText<D: DomTypes> {
    Element(PhantomData<D>),
    Text(PhantomData<D>),
}

pub enum HTMLElementOrLong<D: DomTypes> {
    HTMLElement(PhantomData<D>),
    Long(i32),
}

pub enum IDBObjectStoreOrIDBIndex<D: DomTypes> {
    IDBObjectStore(PhantomData<D>),
    IDBIndex(PhantomData<D>),
}

pub enum MediaStreamOrBlob<D: DomTypes> {
    MediaStream(PhantomData<D>),
    Blob(PhantomData<D>),
}

pub enum MediaStreamTrackOrString<D: DomTypes> {
    MediaStreamTrack(PhantomData<D>),
    String(String),
}

pub enum RadioNodeListOrElement<D: DomTypes> {
    RadioNodeList(PhantomData<D>),
    Element(PhantomData<D>),
}

pub enum ReadableStreamDefaultControllerOrReadableByteStreamController<D: DomTypes> {
    ReadableStreamDefaultController(PhantomData<D>),
    ReadableByteStreamController(PhantomData<D>),
}

pub enum ReadableStreamDefaultReaderOrReadableStreamBYOBReader<D: DomTypes> {
    ReadableStreamDefaultReader(PhantomData<D>),
    ReadableStreamBYOBReader(PhantomData<D>),
}

pub enum RequestOrUSVString<D: DomTypes> {
    Request(PhantomData<D>),
    USVString(String),
}

pub enum StringOrPerformanceMeasureOptions {
    String(String),
    PerformanceMeasureOptions(()),
}

pub enum USVStringOrUndefined {
    USVString(String),
    Undefined,
}

pub enum USVStringOrURLPatternInit {
    USVString(String),
    URLPatternInit(()),
}

pub enum USVStringSequenceSequenceOrUSVStringUSVStringRecordOrUSVString {
    USVStringSequenceSequence(Vec<Vec<String>>),
    USVStringUSVStringRecord(Vec<(String, String)>),
    USVString(String),
}

pub enum VideoTrackOrAudioTrackOrTextTrack<D: DomTypes> {
    VideoTrack(PhantomData<D>),
    AudioTrack(PhantomData<D>),
    TextTrack(PhantomData<D>),
}

// Additional missing union types from script crate
pub enum HTMLOptionElementOrHTMLOptGroupElement<D: DomTypes> {
    HTMLOptionElement(PhantomData<D>),
    HTMLOptGroupElement(PhantomData<D>),
}

pub enum ElementOrDocument<D: DomTypes> {
    Element(PhantomData<D>),
    Document(PhantomData<D>),
}

#[derive(Clone, Debug, Default)]
pub struct ConstrainULongRange {
    pub exact: Option<u32>,
    pub ideal: Option<u32>,
    pub min: Option<u32>,
    pub max: Option<u32>,
}

pub enum ClampedUnsignedLongOrConstrainULongRange {
    ClampedUnsignedLong(u32),
    ConstrainULongRange(ConstrainULongRange),
}

#[derive(Clone, Debug, Default)]
pub struct ConstrainDoubleRange {
    pub exact: Option<f64>,
    pub ideal: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

pub enum DoubleOrConstrainDoubleRange {
    Double(f64),
    ConstrainDoubleRange(ConstrainDoubleRange),
}

pub enum WindowProxyOrMessagePortOrServiceWorker<D: DomTypes> {
    WindowProxy(PhantomData<D>),
    MessagePort(PhantomData<D>),
    ServiceWorker(PhantomData<D>),
}

pub enum ObjectOrString {
    Object(()),
    String(String),
}

pub enum Int32ArrayOrLongSequence {
    Int32Array(Vec<i32>),
    LongSequence(Vec<i32>),
}

pub enum Uint32ArrayOrUnsignedLongSequence {
    Uint32Array(Vec<u32>),
    UnsignedLongSequence(Vec<u32>),
}

pub enum TrustedScriptOrStringOrFunction<D: DomTypes> {
    TrustedScript(PhantomData<D>),
    String(String),
    Function(PhantomData<D>),
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

    // Create InterfaceTypes.rs stub (required by script/build.rs)
    let interface_types_rs = out_dir.join("InterfaceTypes.rs");
    fs::write(&interface_types_rs, r#"
// Auto-generated stub for Boa engine
// Interface type definitions

/// Marker trait for interface types
pub trait InterfaceType {}
"#).unwrap();

    // Create DomTypeHolder.rs stub (required by script/build.rs)
    let dom_type_holder_rs = out_dir.join("DomTypeHolder.rs");
    fs::write(&dom_type_holder_rs, r#"
// Auto-generated stub for Boa engine
// DOM type holder for concrete types

/// Marker struct for DOM type holder
pub struct DomTypeHolder;

impl DomTypeHolder {
    pub fn new() -> Self { Self }
}

impl Default for DomTypeHolder {
    fn default() -> Self { Self::new() }
}
"#).unwrap();

    // Create InterfaceObjectMap.rs stub (required by script/build.rs)
    let interface_object_map_rs = out_dir.join("InterfaceObjectMap.rs");
    fs::write(&interface_object_map_rs, r#"
// Auto-generated stub for Boa engine
// Interface object map for DOM interfaces

use phf::Map;

/// Interface definition stub
#[derive(Clone, Copy, Debug)]
pub struct Interface;

/// Empty interface object map
pub static MAP: Map<&'static [u8], Interface> = phf::phf_map! {};

/// Get interface by name
pub fn get(_name: &[u8]) -> Option<Interface> {
    None
}
"#).unwrap();

    // Create ConcreteInheritTypes.rs stub (required by script/build.rs)
    let concrete_inherit_types_rs = out_dir.join("ConcreteInheritTypes.rs");
    fs::write(&concrete_inherit_types_rs, r#"
// Auto-generated stub for Boa engine
// Concrete inheritance types for DOM elements

/// Trait for types with concrete inheritance info
pub trait ConcreteInheritTypes {}
"#).unwrap();

    // Create UnionTypes.rs stub (required by script/build.rs)
    let union_types_rs = out_dir.join("UnionTypes.rs");
    fs::write(&union_types_rs, r#"
// Auto-generated stub for Boa engine
// Union types for WebIDL union definitions

use crate::codegen::DomTypes::DomTypes;
use std::marker::PhantomData;

/// HTMLOptionElementOrHTMLOptGroupElement union type
#[derive(Clone)]
pub enum HTMLOptionElementOrHTMLOptGroupElement<D: DomTypes> {
    HTMLOptionElement(PhantomData<D>),
    HTMLOptGroupElement(PhantomData<D>),
}

/// ElementOrDocument union type
#[derive(Clone)]
pub enum ElementOrDocument<D: DomTypes> {
    Element(PhantomData<D>),
    Document(PhantomData<D>),
}

/// ClampedUnsignedLongOrConstrainULongRange union type
#[derive(Clone)]
pub enum ClampedUnsignedLongOrConstrainULongRange {
    ClampedUnsignedLong(u32),
    ConstrainULongRange(ConstrainULongRange),
}

#[derive(Clone, Debug, Default)]
pub struct ConstrainULongRange {
    pub exact: Option<u32>,
    pub ideal: Option<u32>,
    pub min: Option<u32>,
    pub max: Option<u32>,
}

/// DoubleOrConstrainDoubleRange union type
#[derive(Clone)]
pub enum DoubleOrConstrainDoubleRange {
    Double(f64),
    ConstrainDoubleRange(ConstrainDoubleRange),
}

#[derive(Clone, Debug, Default)]
pub struct ConstrainDoubleRange {
    pub exact: Option<f64>,
    pub ideal: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

/// WindowProxyOrMessagePortOrServiceWorker union type
#[derive(Clone)]
pub enum WindowProxyOrMessagePortOrServiceWorker<D: DomTypes> {
    WindowProxy(PhantomData<D>),
    MessagePort(PhantomData<D>),
    ServiceWorker(PhantomData<D>),
}

/// DoubleOrDoubleSequence union type
#[derive(Clone)]
pub enum DoubleOrDoubleSequence {
    Double(f64),
    DoubleSequence(Vec<f64>),
}

/// ObjectOrString union type
#[derive(Clone)]
pub enum ObjectOrString {
    Object(crate::js::jsapi::JSObject),
    String(String),
}

/// Int32ArrayOrLongSequence union type
#[derive(Clone)]
pub enum Int32ArrayOrLongSequence {
    Int32Array(Vec<i32>),
    LongSequence(Vec<i32>),
}

/// Uint32ArrayOrUnsignedLongSequence union type
#[derive(Clone)]
pub enum Uint32ArrayOrUnsignedLongSequence {
    Uint32Array(Vec<u32>),
    UnsignedLongSequence(Vec<u32>),
}

/// TrustedScriptOrStringOrFunction union type
#[derive(Clone)]
pub enum TrustedScriptOrStringOrFunction<D: DomTypes> {
    TrustedScript(PhantomData<D>),
    String(String),
    Function(PhantomData<D>),
}

/// Placeholder for union type stubs
pub mod unions {
    pub use super::*;
}
"#).unwrap();

    // Create ConcreteBindings folder (required by script/build.rs)
    let concrete_bindings_dir = out_dir.join("ConcreteBindings");
    fs::create_dir_all(&concrete_bindings_dir).unwrap();
    
    // Create a placeholder file in ConcreteBindings
    let placeholder = concrete_bindings_dir.join("mod.rs");
    fs::write(&placeholder, r#"
// Auto-generated stub for Boa engine
// Concrete bindings module
"#).unwrap();
}
