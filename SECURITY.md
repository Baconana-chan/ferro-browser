# Security Policy

## Reporting Security Issues

Ferro Browser takes security issues seriously. If you discover a security vulnerability, please **do not** open a public issue on GitHub.

Instead, please report it privately via:
- **GitHub Security Advisory**: [Ferro Browser Security](https://github.com/Baconana-chan/ferro-browser/security/advisories/new)
- Include details about the vulnerability and steps to reproduce
- Allow reasonable time for a fix before public disclosure

## Security Considerations

As a browser engine, Ferro Browser handles untrusted web content. Key security areas include:

### JavaScript Engine (Boa)
- Pure Rust implementation reduces C++ memory safety risks
- Regular updates to Boa 0.21+ with security patches
- No external process execution from JavaScript

### Network Security
- HTTPS/TLS support via Rust-based libraries
- Certificate validation (webpki-based)
- No support for insecure content by default

### Sandboxing
- DOM/JavaScript execution in isolated context
- No access to filesystem unless explicitly required
- Resource limits on script execution

### Dependencies
- Regular audits of Rust crates via `cargo audit`
- Minimal C/C++ dependencies (vs. SpiderMonkey's complexity)
- Security patches applied promptly

## Known Limitations

Ferro Browser is a **lightweight browser focused on modern web compatibility**, not a production security solution. Use at your own discretion.

### What Ferro is NOT:
- A security-hardened browser for sensitive operations
- A replacement for production browsers (Chrome, Firefox)
- Suitable for accessing sensitive accounts without additional security measures

### Best Practices

1. Keep Ferro Browser updated
2. Use HTTPS for all web browsing
3. Be cautious with untrusted websites
4. Use a dedicated profile for sensitive operations
5. Report security issues responsibly

## Upstream Security

Ferro Browser inherits security from [Servo Project](https://servo.org/). For Servo security issues:
- See [Servo Security](https://github.com/servo/servo/security)
- [Servo Bug Bounty](https://www.mozilla.org/en-US/security/bug-bounty-program/)

## Security Updates

Security patches are tracked in [TODO.md](TODO.md) and released as soon as practical. Subscribe to [GitHub Releases](https://github.com/Baconana-chan/ferro-browser/releases) for updates.

