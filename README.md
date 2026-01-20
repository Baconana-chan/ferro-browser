# Ferro Browser

🦀 **Lightweight, minimalist web browser** built on [Servo](https://servo.org/) in pure Rust

**Status**: Active development • **Platforms**: macOS, Linux, Windows, OpenHarmony, Android

## About

Ferro Browser is a research project exploring a more streamlined browser experience. Rather than bloat, we focus on:

✅ **Modern web compatibility** (Tailwind, ES2024)  
✅ **Pure Rust** (Boa 0.21 JS engine, no C++ SpiderMonkey)  
✅ **Fast builds** (2-4 minutes vs 10-15 for SpiderMonkey)  
✅ **Essential UX** (text selection, caret, smooth scrolling)  
✅ **FFmpeg media** (no GStreamer overhead)

## Project Status

### ✅ Completed (Priority 1: Critical Usability)

| Feature | Status | Notes |
|---------|--------|-------|
| Text input caret | ✅ | Blinking with 530ms interval, stops in background |
| Mouse selection | ✅ | Click, drag, double-click word, triple-click line |
| Smooth scrolling | ✅ | Momentum with ease-out cubic, 300ms animation |
| Clipboard (Ctrl+C/V) | ✅ | Full copy/paste support |
| Text input handling | ✅ | Input, textarea with full text manipulation |

### 🚧 In Progress (Priority 2)

| Component | Progress | Details |
|-----------|----------|---------|
| Boa bindings | 80% | JsRuntime working, type conversions, error handling complete |
| Media playback | 90% | FFmpeg integration, YouTube thumbnails fixed |
| Web compat | 60% | Modern CSS, Layout engine improvements |

### 📋 Future (Priority 3+)

- Full Boa integration with DOM
- WebIDL code generation
- Performance profiling
- Extended CSS support
- Service Workers (future)

## Architecture Highlights

### JavaScript Engine: Boa 0.21 (MIT License)

```rust
// Pure Rust, no C++ dependencies
let mut runtime = JsRuntime::new();
let result = runtime.eval("1 + 1");
assert_eq!(result.as_number(), Some(2.0));
```

**vs SpiderMonkey:**
- ✅ 94% ECMAScript conformance
- ✅ NaN-boxing for memory efficiency
- ✅ Register-based VM
- ✅ Built-in Web APIs (console, setTimeout, fetch)
- ✅ Fast builds (no LLVM/Clang required)

### Media: FFmpeg (via ferro_media, MIT License)

- Replaced GStreamer complexity
- Direct FFmpeg integration
- YouTube video playback ✅
- Audio/video format support

### Layout & Rendering

- CSS Layout (Taffy)
- WebRender for GPU acceleration
- Modern CSS features

## Licensing

Ferro Browser uses **dual licensing**:

- **MPL-2.0**: Files derived from Servo (upstream compatibility)
- **MIT**: New Ferro-specific code (boa_bindings, ferro_media, UI)

## Building

For detailed instructions, see [Getting started](#getting-started) or the [Servo Book](https://book.servo.org/hacking/building-servo.html).

Quick start:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh      # Install uv
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # Install Rust
./mach bootstrap                                     # Install dependencies
./mach build --release                               # Build (~4 min for Boa)
```

## Getting started

For more detailed build instructions, see the Servo book under [Setting up your environment], [Building Servo], [Building for Android] and [Building for OpenHarmony].

[Setting up your environment]: https://book.servo.org/hacking/setting-up-your-environment.html
[Building Servo]: https://book.servo.org/hacking/building-servo.html
[Building for Android]: https://book.servo.org/hacking/building-for-android.html
[Building for OpenHarmony]: https://book.servo.org/hacking/building-for-openharmony.html

### macOS

- Download and install [Xcode](https://developer.apple.com/xcode/) and [`brew`](https://brew.sh/).
- Install `uv`: `curl -LsSf https://astral.sh/uv/install.sh | sh` 
- Install `rustup`: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Restart your shell to make sure `cargo` is available
- Install the other dependencies: `./mach bootstrap`
- Build servoshell: `./mach build`

### Linux

- Install `curl`:
  - Arch: `sudo pacman -S --needed curl`
  - Debian, Ubuntu: `sudo apt install curl`
  - Fedora: `sudo dnf install curl`
  - Gentoo: `sudo emerge net-misc/curl`
- Install `uv`: `curl -LsSf https://astral.sh/uv/install.sh | sh` 
- Install `rustup`: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Restart your shell to make sure `cargo` is available
- Install the other dependencies: `./mach bootstrap`
- Build servoshell: `./mach build`

### Windows

- Download [`uv`](https://docs.astral.sh/uv/getting-started/installation/#standalone-installer), [`choco`](https://chocolatey.org/install#individual), and [`rustup`](https://win.rustup.rs/)
  - Be sure to select *Quick install via the Visual Studio Community installer*
- In the Visual Studio Installer, ensure the following components are installed:
  - **Windows 10/11 SDK (anything >= 10.0.19041.0)** (`Microsoft.VisualStudio.Component.Windows{10, 11}SDK.{>=19041}`)
  - **MSVC v143 - VS 2022 C++ x64/x86 build tools (Latest)** (`Microsoft.VisualStudio.Component.VC.Tools.x86.x64`)
  - **C++ ATL for latest v143 build tools (x86 & x64)** (`Microsoft.VisualStudio.Component.VC.ATL`)
- Restart your shell to make sure `cargo` is available
- Install the other dependencies: `.\mach bootstrap`
- Build servoshell: `.\mach build`

### Android

- Ensure that the following environment variables are set:
  - `ANDROID_SDK_ROOT`
  - `ANDROID_NDK_ROOT`: `$ANDROID_SDK_ROOT/ndk/28.2.13676358/`
 `ANDROID_SDK_ROOT` can be any directory (such as `~/android-sdk`).
  All of the Android build dependencies will be installed there.
- Install the latest version of the [Android command-line
  tools](https://developer.android.com/studio#command-tools) to
  `$ANDROID_SDK_ROOT/cmdline-tools/latest`.
- Run the following command to install the necessary components:
  ```shell
  sudo $ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager --install \
   "build-tools;34.0.0" \
   "emulator" \
   "ndk;28.2.13676358" \
   "platform-tools" \
   "platforms;android-33" \
   "system-images;android-33;google_apis;x86_64"
  ```
- Follow the instructions above for the platform you are building on

### OpenHarmony

- Follow the instructions above for the platform you are building on to prepare the environment.
- Depending on the target distribution (e.g. `HarmonyOS NEXT` vs pure `OpenHarmony`) the build configuration will differ slightly.
- Ensure that the following environment variables are set
  - `DEVECO_SDK_HOME` (Required when targeting `HarmonyOS NEXT`)
  - `OHOS_BASE_SDK_HOME` (Required when targeting `OpenHarmony`)
  - `OHOS_SDK_NATIVE` (e.g. `${DEVECO_SDK_HOME}/default/openharmony/native` or `${OHOS_BASE_SDK_HOME}/${API_VERSION}/native`)
  - `SERVO_OHOS_SIGNING_CONFIG`: Path to json file containing a valid signing configuration for the demo app.
- Review the detailed instructions at [Building for OpenHarmony].
- The target distribution can be modified by passing `--flavor=<default|harmonyos>` to `mach <build|package|install>`.

## Community

Ferro Browser welcomes contributions! Check out:

- **[TODO.md](TODO.md)** — Project roadmap and tasks
- **[CONTRIBUTING.md](CONTRIBUTING.md)** — How to contribute
- **[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)** — Community guidelines
- **GitHub Issues** — Report bugs, request features

## Resources

- [Servo Book](https://book.servo.org) — Engine documentation
- [Servo GitHub](https://github.com/servo/servo) — Upstream project
- [Boa GitHub](https://github.com/boa-dev/boa) — JavaScript engine

Coordination of Ferro Browser development happens:
- Here on [GitHub Issues](https://github.com/Baconana-chan/ferro-browser/issues)
