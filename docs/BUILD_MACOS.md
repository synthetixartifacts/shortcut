# macOS Production Build Guide

Complete step-by-step guide to build ShortCut for macOS.

> **Important**: macOS builds cannot be cross-compiled. You must build on a Mac.

---

## Prerequisites

### 1. Install Xcode Command Line Tools

```bash
xcode-select --install
```

A popup will appear — click "Install" and wait for completion.

### 2. Install Homebrew (if not installed)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the instructions at the end to add Homebrew to your PATH.

### 3. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Verify:
```bash
rustc --version
cargo --version
```

### 4. Install Node.js

```bash
brew install node
node --version   # Should be v20.19+
```

---

## Get the Code

```bash
git clone https://github.com/synthetixartifacts/shortcut.git
cd shortcut
```

---

## Install Dependencies

```bash
npm install
```

---

## Build

No environment variables required. All AI provider configuration is runtime-only
via the Settings UI (users configure their API keys on first launch).

### Rosetta 2 Timeline (Why Architecture Matters)

Apple announced at WWDC 2025 that Rosetta 2 — the Intel-to-Apple-Silicon
translator — is being phased out:

| macOS | Released | Rosetta 2 status |
|-------|----------|------------------|
| 26 Tahoe | Sep 2025 | Full support. Last macOS supporting Intel Macs. |
| 26.4 | Mar 2026 | Shows "Intel-based apps" warning when launching Intel-only binaries. |
| 27 | Sep 2026 (expected) | Full Rosetta 2 support. Apple Silicon only. |
| 28 | Sep 2027 (expected) | Rosetta 2 restricted to legacy gaming; regular Intel apps stop launching. |

Intel Macs running Tahoe continue to receive security updates for three years
after macOS 27 ships.

### Architecture Options

Pick the option that matches your release goal. **All three work from an Intel
Mac** or an Apple Silicon Mac — cross-compilation is supported in both directions
via `rustup`, and `lipo` ships with Xcode Command Line Tools. Requires Xcode
12.2+ (macOS 10.15.4 Catalina or newer) for arm64 target support.

#### Option A — Host-native (legacy / quick local testing)

Builds for the current Mac's architecture only. On an Intel Mac this produces an
Intel-only binary, which triggers the Rosetta warning on Apple Silicon users.
Fine for local testing; not recommended for new distributions.

```bash
npm run tauri build
```

Output: `src-tauri/target/release/bundle/macos/ShortCut.app`

#### Option B — Apple Silicon only (recommended for new releases)

Smallest binary, one ONNX slice, no warnings on modern Macs. Drops Intel Mac
support — reasonable given Tahoe is the last Intel-supporting macOS.

```bash
rustup target add aarch64-apple-darwin
npm run tauri build -- --target aarch64-apple-darwin
```

Output: `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ShortCut.app`

Optional: set `bundle.macOS.minimumSystemVersion` to `"11.0"` in
`tauri.conf.json` to document the Apple Silicon floor.

#### Option C — Universal Binary (Intel + Apple Silicon)

Runs natively on both architectures (~2× bundle size). Only needed if Intel Mac
support matters to your user base during the macOS 26 → 27 window.

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
npm run tauri build -- --target universal-apple-darwin
```

Output: `src-tauri/target/universal-apple-darwin/release/bundle/macos/ShortCut.app`

---

## Building with `local-stt` (ONNX Runtime on macOS)

The production Windows build bundles `onnxruntime.dll`. For macOS with
`--features local-stt` you need a matching `libonnxruntime.dylib`. Microsoft's
macOS release assets have shifted:

| ONNX Runtime version | `osx-arm64` | `osx-x86_64` | `osx-universal2` |
|---------------------|:-----------:|:------------:|:----------------:|
| v1.24.x (latest) | yes | no | no |
| v1.23.x and earlier | yes | yes | yes |

**Option B (arm64-only)** — trivial:

```bash
curl -L -o /tmp/ort.tgz \
  https://github.com/microsoft/onnxruntime/releases/download/v1.24.4/onnxruntime-osx-arm64-1.24.4.tgz
tar -xzf /tmp/ort.tgz -C /tmp
cp /tmp/onnxruntime-osx-arm64-1.24.4/lib/libonnxruntime.dylib src-tauri/resources/
npm run tauri build -- --features local-stt --target aarch64-apple-darwin
```

**Option C (universal)** — pin to ONNX ≤ 1.23 for the pre-built universal2
archive, or build x86_64 from source. The simplest path:

```bash
curl -L -o /tmp/ort.tgz \
  https://github.com/microsoft/onnxruntime/releases/download/v1.23.0/onnxruntime-osx-universal2-1.23.0.tgz
tar -xzf /tmp/ort.tgz -C /tmp
cp /tmp/onnxruntime-osx-universal2-1.23.0/lib/libonnxruntime.dylib src-tauri/resources/
npm run tauri build -- --features local-stt --target universal-apple-darwin
```

Add the dylib to `bundle.resources` in `tauri.conf.json` (mirroring how the
Windows build references `onnxruntime.dll`).

**Option A or skipping local-stt** — omit `--features local-stt`. Cloud STT
(Soniox) continues to work without ONNX.

---

## Build Output

Copy the resulting DMG to `dist/production/` (replace the target path to match
the option you built):

```bash
mkdir -p dist/production
# Option A (host-native)
cp src-tauri/target/release/bundle/dmg/ShortCut_*.dmg dist/production/
# Option B (arm64)
cp src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/ShortCut_*.dmg dist/production/
# Option C (universal)
cp src-tauri/target/universal-apple-darwin/release/bundle/dmg/ShortCut_*.dmg dist/production/
```

---

## First Launch Setup

When a user launches the app for the first time:

1. **Right-click → Open** (required for unsigned apps)
2. Click "Open" in the security dialog
3. Grant permissions when prompted:
   - **Accessibility** (for global shortcuts)
   - **Microphone** (for voice recording)
4. Open **Settings → AI Providers** and configure at least one provider

---

## Granting Permissions

### Accessibility (Required for Shortcuts)

System Settings → Privacy & Security → Accessibility → Add ShortCut

### Microphone (Required for Dictation)

System Settings → Privacy & Security → Microphone → Enable ShortCut

---

## Code Signing (Optional)

### For Personal Use / Testing

Unsigned apps work fine. Users right-click → Open on first launch.

### For Distribution

```bash
# Sign the app
codesign --force --deep --sign "Developer ID Application: Your Name (TEAM_ID)" \
  src-tauri/target/release/bundle/macos/ShortCut.app

# Notarize
xcrun notarytool submit src-tauri/target/release/bundle/dmg/ShortCut_*.dmg \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# Staple
xcrun stapler staple src-tauri/target/release/bundle/dmg/ShortCut_*.dmg
```

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `command not found: npm` | Run `source ~/.zshrc` or open new terminal |
| `command not found: cargo` | Run `source ~/.cargo/env` |
| `xcrun: error: invalid active developer path` | Run `xcode-select --install` |
| Missing dependencies | Delete `node_modules` and run `npm install` again |
| Rust compilation errors | Run `rustup update` |
| "ShortCut.app is damaged" | Run: `xattr -cr /Applications/ShortCut.app` |
| "unidentified developer" | Right-click → Open instead of double-click |
| Shortcuts don't work | Enable Accessibility permission (see above) |
| Microphone not working | Enable Microphone permission (see above) |

---

## Quick Reference

```bash
# One-time setup
xcode-select --install
brew install node
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
npm install

# Build — pick one
npm run tauri build                                             # A: host-native (legacy)
npm run tauri build -- --target aarch64-apple-darwin            # B: arm64-only (recommended)
npm run tauri build -- --target universal-apple-darwin          # C: universal (Intel + arm64)

# Copy to dist (adjust target path per option)
mkdir -p dist/production && cp src-tauri/target/release/bundle/dmg/ShortCut_*.dmg dist/production/

# Fix "damaged" error
xattr -cr /Applications/ShortCut.app
```
