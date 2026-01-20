# Contributing to Ferro Browser

Thank you for your interest in contributing to Ferro Browser! We welcome contributions from everyone.

## Getting Started

1. **Fork and clone** the repository:
   ```bash
   git clone https://github.com/YOUR_USERNAME/ferro-browser.git
   cd ferro-browser
   git remote add upstream https://github.com/Baconana-chan/ferro-browser.git
   ```

2. **Set up your environment** following the [README.md](README.md) instructions for your platform

3. **Create a branch** for your changes:
   ```bash
   git checkout -b fix/your-feature-name
   ```

## Development

### Building

```bash
./mach build                    # Debug build
./mach build --release          # Optimized build
./mach test                     # Run tests
```

### Code Style

- Follow [Rust naming conventions](https://rust-lang.github.io/api-guidelines/naming.html)
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common mistakes
- Use meaningful commit messages

### Testing

Please ensure your changes:
- Don't break existing tests: `./mach test`
- Include tests for new functionality
- Are tested on at least one platform (Linux/macOS/Windows)

### Documentation

- Update [TODO.md](TODO.md) if your changes affect the roadmap
- Add doc comments to public APIs
- Update README if user-facing behavior changes

## Project Areas

Ferro Browser is organized into several key areas:

### Engine Improvements
- **boa_bindings**: Pure Rust JavaScript engine integration (Boa 0.21)
- **Layout & Rendering**: CSS, layout engine, painting
- **DOM/Script**: JavaScript execution and DOM implementation
- **Networking**: HTTP/HTTPS, resource loading

### User Experience
- **UI/Shell**: GUI, browser controls, viewport rendering
- **Input Handling**: Keyboard, mouse, text input
- **Media**: Audio/video playback via FFmpeg

### Quality
- **Performance**: Profiling, optimization
- **Compatibility**: Web compatibility, standards compliance
- **Accessibility**: Screen readers, keyboard navigation

## Licensing

Ferro Browser uses dual licensing:
- **MPL-2.0** for files derived from Servo
- **MIT** for new Ferro-specific code (boa_bindings, ferro_media)

When contributing, new files should use MIT license unless they're modifications of existing Servo files.

## Pull Request Process

1. **Push to your fork** and create a Pull Request
2. **Write a clear description** of what your change does
3. **Link related issues** (e.g., "Fixes #123")
4. **Wait for review** - maintainers will provide feedback
5. **Address feedback** and update your PR
6. **Merge** once approved

## Questions?

- Check [TODO.md](TODO.md) for project roadmap and status
- Read the [Servo Book](https://book.servo.org) for engine documentation
- Open an issue to ask questions or discuss ideas

## Upstream Servo

Ferro Browser is built on Servo. For more information about contributing to Servo, see:
- [Contributing to Servo](https://book.servo.org/contributing.html)
- [Servo GitHub](https://github.com/servo/servo)
