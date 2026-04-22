# Production Distribution Guide

Complete guide for distributing ShortCut to end users.

## Overview

Production builds are identical to dev builds in functionality. The difference is:
- Release optimization flags (`opt-level = 3`, LTO, strip symbols)
- Code signing (platform-specific)

No API keys or service URLs are embedded at compile time. End users configure their
own provider API keys in Settings → AI Providers on first launch.

## Quick Reference

| Platform | Build From | Output Files | Code Signing |
|----------|------------|--------------|--------------|
| Windows | Linux/WSL2 (Docker) | `.exe` | Optional ($200-400/yr) |
| macOS | macOS only | `.app` bundle | Required ($99/yr) |
| Linux | Linux | AppImage, .deb, .rpm | Optional |

## Windows Distribution

### Build

```bash
docker compose build --no-cache build-windows-prod && docker compose up build-windows-prod
```

Output: `dist/production/shortcut.exe`

### Issue 1: Console Window Appearing

In `src-tauri/src/main.rs`, ensure this line is uncommented for production:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

### Issue 2: WebView2Loader.dll Requirement

When cross-compiling from Linux, the WebView2 loader cannot be statically linked.

| Approach | Trade-off |
|----------|-----------|
| Keep DLL alongside exe | Simple, 2 files |
| Build on Windows with MSVC | 1 file, DLL statically linked |
| NSIS/MSI installer | Single installer, professional distribution |

### Windows Code Signing

Optional but recommended to avoid "Unknown Publisher" warnings.

```bash
# Tauri environment variables for signtool
TAURI_SIGNING_PRIVATE_KEY=<base64 encoded .pfx>
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=<password>
```

### Windows Installer (NSIS)

```bash
npm run tauri build -- --bundles nsis
```

Output: `src-tauri/target/release/bundle/nsis/shortcut_x.x.x_x64-setup.exe`

---

## macOS Distribution

### Requirements

- Build machine: macOS required (cannot cross-compile)
- Apple Developer account: $99/year for distribution

### Build

```bash
# Standard build
npm run tauri build

# Universal binary (Intel + Apple Silicon)
rustup target add x86_64-apple-darwin aarch64-apple-darwin
npm run tauri build -- --target universal-apple-darwin
```

### Code Signing (Required for Distribution)

```bash
# Environment variables
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"

# Or for CI (base64-encoded .p12)
export APPLE_CERTIFICATE="<base64 string>"
export APPLE_CERTIFICATE_PASSWORD="<password>"

# Notarization (App Store Connect API)
export APPLE_API_ISSUER="<issuer-id>"
export APPLE_API_KEY="<key-id>"
export APPLE_API_KEY_PATH="/path/to/AuthKey_XXXX.p8"
```

See [BUILD_MACOS.md](./BUILD_MACOS.md) for step-by-step macOS build instructions.

---

## Linux Distribution

```bash
# AppImage (portable)
npm run tauri build -- --bundles appimage

# Debian package
npm run tauri build -- --bundles deb

# RPM package
npm run tauri build -- --bundles rpm
```

---

## GitHub Actions CI/CD

```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    permissions:
      contents: write
    strategy:
      matrix:
        include:
          - platform: 'macos-latest'
            args: '--target universal-apple-darwin'
          - platform: 'ubuntu-22.04'
            args: ''
          - platform: 'windows-latest'
            args: ''

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - name: Install frontend dependencies
        run: npm ci

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          # No API key secrets needed — users configure keys at runtime
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_API_ISSUER: ${{ secrets.APPLE_API_ISSUER }}
          APPLE_API_KEY: ${{ secrets.APPLE_API_KEY }}
          APPLE_API_KEY_PATH: ${{ secrets.APPLE_API_KEY_PATH }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: v__VERSION__
          releaseName: 'ShortCut v__VERSION__'
          releaseDraft: true
          args: ${{ matrix.args }}
```

### Required Secrets

| Secret | Platform | Description |
|--------|----------|-------------|
| `APPLE_CERTIFICATE` | macOS | Base64-encoded .p12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | macOS | Certificate password |
| `APPLE_SIGNING_IDENTITY` | macOS | Signing identity name |
| `APPLE_API_ISSUER` | macOS | App Store Connect API issuer |
| `APPLE_API_KEY` | macOS | App Store Connect API key ID |
| `APPLE_API_KEY_PATH` | macOS | Path to .p8 private key |
| `TAURI_SIGNING_PRIVATE_KEY` | Windows | Base64-encoded .pfx certificate |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Windows | Certificate password |

No API key secrets needed — users configure keys at runtime via the Settings UI.

---

## Pre-Release Checklist

### Before Release

- [ ] Uncomment `windows_subsystem = "windows"` in `main.rs`
- [ ] Remove debug `println!` statements
- [ ] Update version in `tauri.conf.json` and `Cargo.toml`
- [ ] Test on clean machine (no dev tools installed)
- [ ] Verify all shortcuts work
- [ ] Verify provider setup flow (fresh config, no API keys)
- [ ] Verify settings persist after restart

### Windows Release

- [ ] Build with `--no-cache` flag
- [ ] Include `WebView2Loader.dll` with exe (or use NSIS installer)
- [ ] Test on Windows 10 and 11

### macOS Release

- [ ] Build on macOS machine
- [ ] Sign with Developer ID certificate
- [ ] Notarize the application
- [ ] Test on Intel and Apple Silicon

### Linux Release

- [ ] Build AppImage for portable distribution
- [ ] Test on Ubuntu LTS and Fedora

---

## File Size Optimization

Add to `Cargo.toml`:

```toml
[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
```

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Console window appears on Windows | Uncomment `windows_subsystem` in `main.rs` |
| "WebView2Loader.dll not found" | Copy DLL alongside exe or use NSIS installer |
| "App is damaged" on macOS | `xattr -cr /path/to/ShortCut.app` |
| "unidentified developer" on macOS | Sign the app or right-click → Open |
| "Can't detect any appindicator" | Ignorable warning during Linux cross-compile |
