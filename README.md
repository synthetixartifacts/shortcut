# ShortCut

Voice-to-text with AI text transformation — source-available, multi-provider.

No proprietary proxy. Your API keys, your providers, direct calls.

> **Dev flow is Docker-only.** Do not run `npm install` or `cargo build` on the host — the supported workflow is containerized (see below).

## Features

- **Hold-to-record dictation** — Soniox cloud (direct API) or local Parakeet ONNX (Windows)
- **Fix grammar, translate, improve** — any selected text, with any configured LLM
- **Screen Question** — ask AI about your screen using vision-capable providers
- **Radial action wheel** — quick access to all features from any app
- **Global keyboard shortcuts** — work system-wide without switching windows
- **Activity indicator overlay** — visual feedback during operations

## Supported Providers

| Category | Providers |
|----------|-----------|
| LLM | OpenAI, Anthropic, Gemini, Grok, Ollama |
| STT | Soniox (cloud, direct API), Parakeet ONNX (local, Windows) |

## Quick Start

1. Download from [Releases](../../releases)
2. Install and authorize — **[full step-by-step install guide](docs/INSTALL.md)** (macOS Gatekeeper, Windows Defender walkthrough)
3. Go to **Settings → AI Providers** and configure at least one provider
4. Use `Alt+D` (hold, or `Cmd+Shift+D` on macOS) to start dictating

No token gate, no proxy account. Configure only the providers you want to use.

## Default Shortcuts

| Action | Windows / Linux | macOS |
|--------|----------------|-------|
| Dictation (hold to record) | `Alt+D` | `Cmd+Shift+D` |
| Fix grammar | `Alt+G` | `Cmd+Shift+G` |
| Translate | `Alt+T` | `Cmd+Shift+T` |
| Improve | `Alt+I` | `Cmd+Shift+I` |
| Action wheel | `Alt+J` | `Cmd+Shift+J` |
| Screen Question | `Alt+S` | `Cmd+Shift+S` |

All shortcuts are customizable in Settings.

## Platform Support

| Platform | Status |
|----------|--------|
| Windows 10/11 | Primary target (includes local STT) |
| macOS | Supported (requires Accessibility + Microphone permissions) |
| Linux | Supported (X11/Wayland, no local STT) |

## Tech Stack

| Component | Technology |
|-----------|------------|
| Desktop framework | [Tauri 2](https://v2.tauri.app) |
| Frontend | [Svelte 5](https://svelte.dev) + SvelteKit |
| Backend | Rust |
| LLM providers | OpenAI, Anthropic, Gemini, Grok, Ollama — direct HTTP via `reqwest` |
| STT (cloud) | [Soniox](https://soniox.com) direct API |
| STT (local) | NVIDIA Parakeet TDT 0.6B v3 via `transcribe-rs` + ONNX Runtime |

## Development

See [docs/SETUP.md](docs/SETUP.md) for full setup instructions.

Quick start:
```bash
git clone https://github.com/synthetixartifacts/shortcut.git
cd shortcut
docker compose build check
docker compose run --rm check
docker compose up frontend-dev
```

For desktop smoke tests, build an artifact in Docker and run the output from `dist/` on the target OS. `frontend-dev` is UI preview only and does not expose Tauri features such as tray, hotkeys, clipboard, or overlays.

Build for Windows (from Linux/WSL2):
```bash
docker compose build --no-cache build-windows && docker compose up build-windows
```

See [docs/BUILD.md](docs/BUILD.md) for all build targets and options.

## Documentation

| Document | Description |
|----------|-------------|
| [docs/INSTALL.md](docs/INSTALL.md) | Step-by-step install guide (macOS + Windows) |
| [docs/SETUP.md](docs/SETUP.md) | Development environment setup |
| [docs/BUILD.md](docs/BUILD.md) | Build commands, Docker |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design, data flows |
| [docs/BACKEND.md](docs/BACKEND.md) | Rust modules, provider layer, commands |
| [docs/FRONTEND.md](docs/FRONTEND.md) | Frontend structure, Svelte 5 patterns |
| [docs/PROVIDERS.md](docs/PROVIDERS.md) | Provider configuration guide (5 LLM + 2 STT providers) |
| [docs/EXTENDING.md](docs/EXTENDING.md) | Adding features, providers, shortcuts |
| [docs/PRODUCTION.md](docs/PRODUCTION.md) | Distribution, signing, installers |

## License

**Free for personal and non-commercial use.** See [LICENSE](LICENSE) (PolyForm Noncommercial 1.0.0).

Commercial use — including resale, paid support, hosted/SaaS offerings, or bundling into a commercial product — requires a separate agreement. Open an issue to discuss.

This is a **source-available** project, not OSI-approved open source. Contributions are welcome under the project's inbound-license rule; see [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

To report a vulnerability, see [SECURITY.md](SECURITY.md). Do not open a public issue for security bugs.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.
