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

### Build a Universal Binary (Intel + Apple Silicon)

ShortCut ships one DMG that runs on every Mac from 2012 Intel through the
latest Apple Silicon (M-series). macOS picks the correct slice at launch, so
end users never have to choose an architecture.

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
npm run tauri build -- --target universal-apple-darwin
```

Output:
- `src-tauri/target/universal-apple-darwin/release/bundle/macos/ShortCut.app`
- `src-tauri/target/universal-apple-darwin/release/bundle/dmg/ShortCut_<version>_universal.dmg`

> Do NOT also build `aarch64-apple-darwin` or `x86_64-apple-darwin` separately
> for a release — the universal DMG already contains both slices. Shipping
> additional per-architecture DMGs only confuses users.

---

## Build Output

Copy the DMG to `dist/production/`:

```bash
mkdir -p dist/production
cp src-tauri/target/universal-apple-darwin/release/bundle/dmg/ShortCut_*.dmg dist/production/
```

---

## Optional: Local STT (Parakeet ONNX) on macOS

The default release does NOT include local STT on macOS — cloud Soniox works
without ONNX, so skip this section for a v0.1.0 release. If you want local
STT on macOS later, pin ONNX Runtime to v1.23.0 for the pre-built `universal2`
archive:

```bash
curl -L -o /tmp/ort.tgz \
  https://github.com/microsoft/onnxruntime/releases/download/v1.23.0/onnxruntime-osx-universal2-1.23.0.tgz
tar -xzf /tmp/ort.tgz -C /tmp
cp /tmp/onnxruntime-osx-universal2-1.23.0/lib/libonnxruntime.dylib src-tauri/resources/
npm run tauri build -- --features local-stt --target universal-apple-darwin
```

Then add the dylib to `bundle.resources` in `tauri.conf.json` (mirroring the
Windows `onnxruntime.dll` reference).

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
rustup target add x86_64-apple-darwin aarch64-apple-darwin
npm install

# Build universal DMG (Intel + Apple Silicon, single file)
npm run tauri build -- --target universal-apple-darwin

# Copy to dist
mkdir -p dist/production && \
  cp src-tauri/target/universal-apple-darwin/release/bundle/dmg/ShortCut_*.dmg dist/production/

# Fix "damaged" error on end-user Mac
xattr -cr /Applications/ShortCut.app
```
