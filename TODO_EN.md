# Ferro Browser — TODO

**A Servo fork for creating a lightweight, minimalist browser**

## Project Goal
Create a lightweight, minimalist browser based on Servo (Rust) that:
- Opens modern Tailwind websites (like lorachi.xyz) without major layout issues
- Supports basic search (DDG full mode, not lite)
- Has acceptable interactivity (caret, selection, smooth scroll)
- Blocks ads "by default" (due to incomplete compatibility, until fixed)
- Works stably on Linux/macOS/Windows (via embedding in custom shell)

Current status: Servo 0.0.3 — proof-of-concept engine, servoshell — minimal demo. Many missing web platform features.

## Priority 1: Critical Usability (so it doesn't annoy daily)
- [x] Visible caret + blinking in <input>/<textarea> (textinput.rs / layout)
  - ✅ Implemented caret blinking with 530ms interval
  - ✅ Caret resets on text input and position change
  - ✅ Page visibility: blinking stops when tab in background (saves CPU)
  - ✅ Timers are unique per-element (Option<i32> in each element)
  - 🔧 TODO: prefers-reduced-motion (static caret for accessibility)
  - 🔧 TODO: Repaint optimization (only caret area, not entire input)
  - 🔧 TODO: A11y integration (caret position in accessibility tree)
  - Changes: htmlinputelement.rs, htmltextareaelement.rs, document.rs
- [x] Mouse text selection + visual highlight (components/script/dom/selection)
  - ✅ Selection by mouse: mousedown starts, mousemove extends, mouseup completes
  - ✅ Shift+click: extend selection from current caret position
  - ✅ Double-click: select word (Unicode word boundaries)
  - ✅ Triple-click: select entire line
  - ✅ Caret hidden when selection active (non-collapsed selection)
  - ✅ Ctrl+C/X/V: copy, cut, paste (already implemented in textinput.rs)
  - ✅ Added methods select_word_at_index(), select_line_at_index(), selection_is_collapsed()
  - ✅ Layout supports selection_range rendering (visual highlight)
  - 🔧 TODO: Auto-scroll when dragging beyond visible area
  - 🔧 TODO: Highlight color in preferences
  - Changes: textinput.rs, htmlinputelement.rs, htmltextareaelement.rs
- [x] Smooth scrolling (momentum/inertia/easing) for mouse wheel/trackpad
  - ✅ Created smooth_scroll.rs module with animation and easing (ease-out cubic)
  - ✅ SmoothScrollAnimation: position interpolation with 300ms duration
  - ✅ SmoothScrollState: delta accumulation, frame_delta calculation each frame
  - ✅ Integration with WebViewRenderer: on_wheel_event → smooth_scroll_state
  - ✅ Automatic repaint while animation active
  - ✅ Smooth merging of sequential wheel events (momentum)
  - 🔧 TODO: Configurable duration and easing via preferences
  - 🔧 TODO: Disable smooth scroll for accessibility (reduced-motion)
  - 🔧 TODO: Trackpad-specific momentum (velocity-based deceleration)
  - Changes: smooth_scroll.rs (new), webview_renderer.rs, painter.rs, lib.rs
- [x] Clipboard copy-paste (keyboard shortcuts)
  - ✅ Ctrl+C/X/V work in text inputs (textinput.rs handle_keydown)
  - ✅ OS clipboard integration via EmbedderClipboardProvider
  - 🔧 TODO: navigator.clipboard JS API
  - 🔧 TODO: Context menu copy/paste
- [ ] Basic form interaction (submit, checkbox/radio visuals, select dropdown)
  - Estimate: 3–6 weeks

## Priority 2: Compatibility with Real Websites (lorachi.xyz, DDG, etc.)
- [x] Full/improved CSS Grid + Flexbox edge-case support
  - ✅ minmax(auto-fit), repeat(auto-fit, ...), subgrid, nested grids
  - ✅ Implemented subgrid and masonry support in Taffy wrapper (wrapper.rs)
  - Issue: cards overlapping, masonry effects breaking on Tailwind sites
  - Estimate: 3–6 weeks (stylo crate)
- [ ] Dark mode (prefers-color-scheme + .dark class handling)
  - Often ignored → light theme on dark sites
  - Estimate: 1–3 weeks
- [x] Fetch API + modern JS (async/await, modules, BigInt if needed)
  - ✅ IndexedDB enabled by default (dom_indexeddb_enabled = true)
  - ✅ Performance API User Timing Level 3 (mark/measure return objects)
  - ✅ PerformanceMark.detail and PerformanceMeasure.detail attributes
  - ✅ Constructable StyleSheets (adoptedStyleSheets) enabled for GitHub
  - For DDG to switch to full mode (filters, suggestions, bangs)
  - 🔧 TODO: Check async/await edge-cases
  - 🔧 TODO: ES Modules imports (import/export)
- [ ] Media / images improvements (lazy loading, object-fit/cover, aspect-ratio)
  - Images stretch/break/load slowly
  - Estimate: 2–5 weeks
- [ ] Shadow DOM + custom elements basics
  - Many modern UI components (web components) break
  - Estimate: 4–8 weeks (roadmap priority)

## Priority 3: Performance & Stability
- [ ] Incremental layout / reflow optimization
  - Currently often full relayout → lags on input/scroll
  - Estimate: 4–8 weeks (layout engine)
- [ ] Reduce crashes on complex websites (memory leaks, panic in webrender)
  - Estimate: ongoing (mach test-wpt + fuzzing)
- [x] Video playback streaming (Media Source Extensions)
  - ✅ MediaSource API implemented (WebIDL + Rust)
  - ✅ SourceBuffer and SourceBufferList created
  - ✅ isTypeSupported() for video/mp4, video/webm, audio/mp4, audio/webm
  - ✅ Enabled by default (dom_mediasource_enabled = true)
  - ✅ Created MSE SegmentParser in ferro_media/src/mse.rs
  - ✅ Support for parsing ISOBMFF (MP4) and WebM containers
  - ✅ MseSourceBuffer with buffered ranges management
  - ✅ MsePlayer in servo_media_ferro for integration
  - ✅ Standalone video/audio documents with controls
  - ✅ **FerroPlayer fully implemented with FFmpeg:**
    - ✅ play/pause/stop/seek working (components/servo_media_ferro/lib.rs)
    - ✅ Automatic seek(0) on play from Ended state (restart video)
    - ✅ EndOfStream handling → Ended state
    - ✅ push_data() buffers stream and sends NeedData/EnoughData events
    - ✅ Support for both URL and Media Source Extensions
    - ✅ Video/audio decoding via FFmpeg
    - ✅ System audio output via cpal/rodio
  - ✅ Improved standalone media document design (components/script/dom/servoparser/mod.rs):
    - ✅ Black background for video/audio player
    - ✅ Content centering (flexbox)
    - ✅ Proper scaling with aspect ratio preservation
    - ✅ CSS styles also for img elements (images)
    - ✅ Player fills screen without white bars
  - ✅ Added debug logs for diagnostics:
    - ✅ Logs in handle_animated_image (components/layout/context.rs - frame count)
    - ✅ Logs in decode_animated_image (components/pixels/lib.rs - decoding info)
    - ✅ Logs in update_active_frames (components/script/image_animation.rs - frame updates)
  - 🔧 TODO: Test GIF animation in browser (logs will show problem stage)
  - 🔧 TODO: Optimize frame updates for smooth animation
  - 🔧 TODO: Add AVIF, WebP format support on Windows (requires libdav1d)

## Priority 4: Browser Shell / UI Features (on top of Servo)
- [x] Modern navigation bar design
  - ✅ Adaptive tabs: scale from 60px to 200px depending on count
  - ✅ Horizontal scroll for many tabs (13+ working)
  - ✅ Modern colors for dark/light mode
  - ✅ Rounded tabs in Chrome style
  - ✅ Accent bar on active tab
  - ✅ Rounded address bar with lock/warning icons
  - ✅ Improved navigation buttons (back, forward, reload)
  - 🔧 TODO: Drag-and-drop tab reordering
  - Changes: ports/servoshell/desktop/gui.rs
- [ ] Tabbed browsing + session restore
- [ ] Bookmarks / history basics
- [ ] Address bar with autocompletion (duckduckgo suggestions once JS fixed)
- [ ] Context menu (copy, open in new tab, inspect if devtools)
- [ ] Ad/tracker "blocking" as feature (currently via incompleteness — later optional filter lists)
- [ ] Dark/light theme switcher in UI (independent of site)

## Priority 5: Long-term / Nice-to-have
- [ ] WebGL/WebGPU full usability (already partially there)
- [ ] Accessibility (ARIA, screen reader support)
- [ ] Extensions subset (WebExtensions basics)
- [ ] Android embedding (already experimental)
- [ ] Upstream patches to servo/servo (to stay in sync with main)

## Testing & Metrics
- Test on:
  - lorachi.xyz (Tailwind grid + dark + cards)
  - duckduckgo.com (full mode with filters)
  - wikipedia.org (text selection + smooth scroll)
  - youtube.com (video playback)
- Target scores: HTML5Test > 300–350, Acid3 ~80–90%
- Tools: mach test-wpt, wpt.fyi comparison with Gecko

## Media Migration: GStreamer → FFmpeg
**Status: FFmpeg BUILDING ✅ | GStreamer REMOVED ✅**

### Migration Reasons:
- GStreamer poor cross-platform support (especially Windows)
- Complex dependency setup (GStreamer DLLs, plugins)
- servo-media adds much overhead

### New ferro_media Stack:
- **ffmpeg-next v8** — Rust bindings for FFmpeg (video/audio decoding)
- **cpal/rodio** — cross-platform audio output
- **image crate** — static images and GIF

### Progress:
- [x] Created components/ferro_media with basic structure
- [x] Implemented MediaPlayer with FFmpeg decoder
- [x] Implemented AudioOutput with cpal/rodio
- [x] Added to workspace dependencies
- [x] **FFmpeg 7.1 + ffmpeg-next 8.0 successfully builds on Windows!**
- [x] Created servo-media-ferro backend (components/servo_media_ferro)
- [x] Integration with libservo via `media-ferro` feature
- [x] Building servoshell with `--features media-ferro` works!
- [x] **GStreamer completely removed from project!** 🎉
- [ ] Testing actual media playback
- [ ] Testing on Linux/macOS

### FFmpeg Setup for Development (Windows):
```powershell
# 1. Download FFmpeg 7.1 shared builds:
Invoke-WebRequest -Uri "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.1-latest-win64-gpl-shared-7.1.zip" -OutFile "$env:USERPROFILE\ffmpeg71.zip"
Expand-Archive -Path "$env:USERPROFILE\ffmpeg71.zip" -DestinationPath "C:\ffmpeg71" -Force

# 2. Before building run setup script:
. .\setup_ffmpeg.ps1

# 3. Build with FFmpeg:
cargo build -p ferro_media --features ffmpeg
```

### Requirements:
- LLVM/Clang (for bindgen): `C:\Program Files\LLVM\bin`
- FFmpeg 7.1 shared libs: `C:\ffmpeg71\ffmpeg-n7.1-latest-win64-gpl-shared-7.1`

**Linux:**
```bash
sudo apt install libavcodec-dev libavformat-dev libavutil-dev libswscale-dev
```

**macOS:**
```bash
brew install ffmpeg
```

## Notes
- Many issues — not "bugs", but missing features (Servo — research/embedding engine, not full browser).
- AI (Claude/Grok/o3) helps well with individual modules (stylo, script), but debug/reflow — manual.
- Upstream Servo updates — rebase/merge every 1–2 months (if servo/ changes — conflicts will happen).

Start with caret + selection + smooth scroll — this gives biggest "doesn't annoy" improvement.

## Web Platform APIs Progress

### Implemented APIs:
- [x] **IndexedDB** — enabled by default (dom_indexeddb_enabled = true)
- [x] **MediaSource Extensions (MSE)** — full implementation
  - MediaSource, SourceBuffer, SourceBufferList (DOM)
  - isTypeSupported() for video/mp4, video/webm, audio/mp4, audio/webm, audio/mpeg
  - Events: sourceopen, sourceended, sourceclose, updatestart, update, updateend
  - ferro_media/src/mse.rs — SegmentParser for MP4/WebM
  - servo_media_ferro — MsePlayer with buffered ranges
- [x] **Performance User Timing Level 3**
  - performance.mark() returns PerformanceMark object
  - performance.measure() returns PerformanceMeasure object
  - detail attribute on PerformanceMark and PerformanceMeasure
  - PerformanceMarkOptions and PerformanceMeasureOptions support

### Require Implementation for YouTube/TikTok:
- [ ] Encrypted Media Extensions (EME) — DRM content
- [ ] WebCodecs API — low-level decoding
- [ ] ResizeObserver — track size changes
- [ ] IntersectionObserver improvements
- [ ] Web Workers / Service Workers full support

---

## 🦀 JavaScript Migration: SpiderMonkey → Boa

**Status: IN DEVELOPMENT 🚧 | Priority: HIGH**

### Licensing:
- Files from Servo: MPL-2.0 (preserve original license)
- New Ferro files (boa_bindings, ferro_media): MIT
- Dual licensing allows maximum flexibility for developers

### Migration Reasons:
- SpiderMonkey — C++ legacy code from Mozilla, complex FFI via mozjs crate
- Compiling mozjs takes 10-15 minutes, requires Clang/LLVM
- Debugging JS errors nearly impossible (C++ <-> Rust boundary)
- High overhead even on simple pages due to SM weight
- Boa — pure Rust, unified ecosystem with Servo
- Boa has Rust GC (boa_gc), compatible with our architecture

### Boa 0.21 Features (October 2025):
- **94.12% ECMAScript conformance** (Test262)
- NaN-boxing — less memory for JsValue
- Register-based VM — faster execution
- boa_runtime: fetch, setTimeout, setInterval, queueMicrotask
- Temporal proposal ~97% conformance
- Error.isError, new Set methods, Float16 support

### Boa Crates (https://github.com/boa-dev/boa):
- `boa_engine` — main engine, builtin objects, execution
- `boa_parser` — lexer and parser for ECMAScript
- `boa_ast` — Abstract Syntax Tree
- `boa_gc` — Rust garbage collector
- `boa_interner` — string interner for optimization
- `boa_runtime` — WebAPI features (console, fetch, setTimeout, etc.)
- `boa_icu_provider` — ICU4X for internationalization

### Current SpiderMonkey State in Servo:
```
Cargo.toml:
  js = { package = "mozjs", git = "https://github.com/servo/mozjs" }

Key modules:
  components/script/           — DOM implementation, uses js:: directly
  components/script_bindings/  — WebIDL bindings, code generation for SM
  components/script_bindings/codegen/ — generator for Bindings from WebIDL
  
SpiderMonkey dependencies:
  - js::jsapi::* — low-level SM API
  - js::rust::* — Rust wrappers for SM
  - js::gc::* — SM garbage collector integration
  - js::typedarray::* — TypedArray bindings
```

### Migration Plan (Phases):

#### Phase 0: Preparation and Research [✅ COMPLETED]
- [x] Create TODO migration plan
- [x] Add Boa 0.21 crates to workspace dependencies
- [x] Create components/boa_bindings module (MIT license)
- [x] Implement JsRuntime wrapper for Context
- [x] Implement basic type conversions
- [x] Implement error handling
- [x] Implement GC integration (DomRef, DomCell)
- [x] Pass basic tests (eval, functions, strings)

#### Phase 1: Basic Boa Integration [CURRENT]
- [x] Add boa_engine, boa_parser, boa_gc to workspace dependencies
- [ ] Create components/boa_bindings/ — new bindings module
- [ ] Implement basic JsRuntime on Boa (analog to script_runtime.rs)
- [ ] Implement Reflector/DomObject for Boa GC
- [ ] Simple tests: eval("1+1"), console.log(), setTimeout()

#### Phase 2: WebIDL Code Generation
- [ ] Modify components/script_bindings/codegen/ for Boa
  - Or create separate codegen for Boa
- [ ] Generate Rust bindings from .webidl files for Boa
- [ ] Implement type conversions (DOMString, Uint8Array, etc.)
- [ ] Interfaces: Window, Document, Element, Node (basic)

#### Phase 3: DOM Bindings Core
- [ ] Port htmlelement.rs, document.rs, window.rs
- [ ] Event system (addEventListener, dispatchEvent)
- [ ] DOM manipulation (createElement, appendChild, etc.)
- [ ] CSS Object Model (getComputedStyle, classList)

#### Phase 4: Web APIs
- [ ] Console API (console.log/warn/error)
- [ ] Fetch API (using existing net stack)
- [ ] Timers (setTimeout, setInterval, requestAnimationFrame)
- [ ] Storage (localStorage, sessionStorage)
- [ ] IndexedDB (existing implementation)

#### Phase 5: Advanced Features
- [ ] ES Modules (import/export)
- [ ] async/await, Promises
- [ ] Web Workers
- [ ] WebGL/WebGPU bindings
- [ ] MediaSource Extensions

#### Phase 6: Complete SpiderMonkey Removal
- [ ] Remove mozjs from dependencies
- [ ] Remove components/script_bindings (old)
- [ ] Rename boa_bindings → script_bindings
- [ ] Update all imports in components/script

### Key Files for Migration:
```
components/script_bindings/
├── script_runtime.rs     → JsRuntime on Boa
├── reflector.rs          → Boa GC integration
├── root.rs               → Rooted pointers for Boa
├── trace.rs              → Boa::Trace instead of JSTraceable
├── conversions.rs        → Type conversions for Boa
├── error.rs              → JS Error handling
├── codegen/              → WebIDL → Boa bindings generator
└── webidls/              → WebIDL definitions (unchanged)
```

### Boa Compliance (Test262):
- ~80% ECMAScript compliance (vs SM ~95%)
- Main gaps: some edge-cases in Proxy, WeakRef, FinalizationRegistry
- For most sites this is sufficient
- Active development, compliance growing

### Feature Flags (for gradual migration):
```toml
[features]
default = ["js-spidermonkey"]  # Current default
js-spidermonkey = ["mozjs"]    # Legacy SpiderMonkey
js-boa = ["boa_engine", "boa_parser", "boa_gc"]  # New Boa
```

### Time Estimate:
- Phase 0-1: 2-4 weeks
- Phase 2-3: 6-10 weeks
- Phase 4-5: 4-8 weeks
- Phase 6: 1-2 weeks
- **Total: ~3-6 months**

### Resources:
- Boa docs: https://docs.rs/boa_engine/
- Boa GitHub: https://github.com/boa-dev/boa
- Boa playground: https://boajs.dev/playground/
- Test262 status: https://test262.fyi/

---
