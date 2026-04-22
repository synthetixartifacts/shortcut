# Build Guide

## How Builds Work

ShortCut uses a single build configuration. There is no production vs dev split at
build time — all AI provider configuration is runtime-only via the Settings UI.
Production builds differ from dev only in optimization flags and code signing.

Packaged app icons are generated from `static/icon.png` during `tauri build`.
If branding changes, update that file and rebuild; the build regenerates the
platform bundle icons in `src-tauri/icons/` before packaging.

---

## Docker-Only Workflow

All validation and build commands run inside Docker. Do **NOT** run `npm`, `npm run check`, `cargo`, `cargo check`, or `cargo clippy` on the host — the repo supports WSL/Linux hosts that may not have the Rust toolchain or WebKit/GTK libraries at all. Desktop runtime verification still happens by running the Docker-built artifact on the target OS.

### Validation

```bash
# Full validation (default profile)
docker compose build check && docker compose run --rm check

# Faster iteration / default PR profile
docker compose run --rm -e CHECK_PROFILE=fast check
```

The `check` service is defined in `docker-compose.yml` and builds from `Dockerfile.audit` (which carries Rust, Node 20, and the GTK/WebKit2GTK stack needed for Tauri's Linux target to compile). First build is ~5 min; subsequent runs are fast thanks to cached layers and named volumes (`check-node-modules`, `check-cargo-target`, `check-cargo-registry`).

Supported `CHECK_PROFILE` values are `full`, `fast`, `frontend`, `rust`, `clippy`, and `skip`. `full` runs `svelte-check + cargo check + cargo clippy --all-targets -- -D warnings`; `fast` skips clippy for a quicker preflight.

GitHub Actions uses the same profiles. By default, `pull_request` runs use `fast` and `push` to `main` uses `full`. You can override those defaults with repository variables `CI_PR_CHECK_PROFILE` and `CI_PUSH_CHECK_PROFILE`, or choose a profile manually from the `validation` workflow's `workflow_dispatch` input.

---

## Quick Commands

### Development Build

```bash
# Frontend preview only (no Tauri APIs)
docker compose up frontend-dev

# Windows dev build (from Linux/WSL2) — outputs to dist/dev/
docker compose build --no-cache build-windows && docker compose up build-windows

# Linux dev build — outputs to dist/dev/
docker compose build --no-cache build-linux && docker compose up build-linux
```

`frontend-dev` is for Svelte UI work only. Use artifacts in `dist/` for native smoke tests covering tray, hotkeys, clipboard, screen capture, microphone, and overlays.

### Production Build

```bash
# Windows production build (optimized + minified) — outputs to dist/production/
docker compose build --no-cache build-windows-prod && docker compose up build-windows-prod

# Linux production build — outputs to dist/production/
docker compose build --no-cache build-linux-prod && docker compose up build-linux-prod
```

Production builds use release optimization flags. Users configure their API keys
in Settings → AI Providers on first launch.

---

## macOS Packaging Note

macOS application packaging remains outside the standard containerized workflow because Tauri macOS bundles must be produced on macOS. If you need that path, see [BUILD_MACOS.md](./BUILD_MACOS.md). Day-to-day contributor validation, preview, and non-macOS builds stay inside Docker.

---

## Other Commands

```bash
# Frontend preview only (no Tauri)
docker compose up frontend-dev
# Access at http://localhost:1420

# Open a shell in the build container
docker compose run --rm shell
```

---

## Critical Build Rules

### 1. Always Use `--no-cache`

Docker caches source files. Without `--no-cache`, source changes won't apply.

```bash
# CORRECT
docker compose build --no-cache build-windows

# WRONG — may use stale cached source
docker compose build build-windows
```

### 2. Use MSVC Target, NOT GNU

```bash
# CORRECT — stable Windows binaries
--target x86_64-pc-windows-msvc --runner cargo-xwin

# WRONG — crashes with STATUS_STACK_BUFFER_OVERRUN (0xc0000409)
--target x86_64-pc-windows-gnu
```

### 3. Use Docker build services, NOT `cargo build`

```bash
# CORRECT — build-* services invoke `tauri build` inside Docker
docker compose up build-windows

# WRONG — no frontend, shows "localhost refused to connect"
cargo xwin build --release
```

### 4. Empty Plugins Config

In `tauri.conf.json`:
```json
"plugins": {}
```

Do NOT use `"plugins": { "plugin-name": {} }` — causes crash.

---

## Environment Variables

No build-time URL embedding. Runtime provider configuration is handled in the app UI.

---

## Build Output

| Platform | Build Type | Output Location |
|----------|------------|-----------------|
| Windows | Dev | `dist/dev/shortcut.exe` |
| Windows | Production | `dist/production/shortcut.exe` |
| macOS | Dev | `src-tauri/target/debug/shortcut` |
| macOS | Production | `src-tauri/target/release/bundle/macos/ShortCut.app` |

---

## Troubleshooting

| Issue | Cause | Fix |
|-------|-------|-----|
| App crashes immediately | Used GNU target | Rebuild with MSVC target |
| "localhost refused to connect" | Frontend not bundled | Use the Docker build service, which runs `tauri build`, not `cargo build` |
| Plugin initialization error | Bad plugins config | Set `"plugins": {}` in tauri.conf.json |
| Changes not applied | Docker cache | Add `--no-cache` to build command |

### Appindicator Warning (Ignorable)

```
Can't detect any appindicator library
```

Happens because Linux build host lacks appindicator. Windows exe works fine.

---

## Docker Files

| File | Purpose |
|------|---------|
| `Dockerfile.windows-msvc` | MSVC cross-compilation (recommended) |
| `Dockerfile` | Linux native build |
| `Dockerfile.audit` | Dependency audit |
