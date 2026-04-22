# Development Setup

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Docker | Latest | Standard workflow for setup, validation, preview, and builds |

The supported contributor workflow is containerized. Do not install Node, Rust, or Linux GTK/WebKit dependencies on the host unless you are intentionally working outside the supported path.

## Installation

```bash
# Clone
git clone https://github.com/synthetixartifacts/shortcut.git
cd shortcut
```

## Configuration

ShortCut stores all configuration at runtime in your user data directory:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\com.g-prompter.shortcut\config.json` |
| macOS | `~/Library/Application Support/com.g-prompter.shortcut/config.json` |
| Linux | `~/.local/share/com.g-prompter.shortcut/config.json` |

### Provider Setup (First Launch)

On first launch, ShortCut starts without any providers configured. The Dashboard
shows a setup banner. To configure providers:

1. Open **Settings → AI Providers**
2. Add at least one **LLM provider** (grammar, translate, improve, screen question)
3. Add **Soniox API key** if you want cloud dictation (or enable Local engine for Windows)
4. In **Settings → Task Assignments**, assign each task to a provider + model

You only need to configure the providers you plan to use. See [docs/PROVIDERS.md](./PROVIDERS.md) for provider-specific instructions.

## Development Commands

```bash
# Full validation
docker compose build check && docker compose run --rm check

# Faster iteration / default PR profile
docker compose run --rm -e CHECK_PROFILE=fast check

# Dead-code audit
docker compose build audit && docker compose run --rm audit

# Web UI preview only
docker compose up frontend-dev

# Windows desktop build → dist/dev/
docker compose build --no-cache build-windows && docker compose up build-windows

# Linux desktop build → dist/dev/
docker compose build --no-cache build-linux && docker compose up build-linux

# Shell inside the project container
docker compose run --rm shell
```

`frontend-dev` serves the Svelte UI only. For tray, hotkeys, clipboard, screen capture, microphone, and overlay behavior, run a Docker-built artifact on the target OS.

## WSL2 Development

Use the same Docker commands as Linux. No host Node or Rust setup is required.

## Default Shortcuts

| Action | Windows/Linux | macOS |
|--------|--------------|-------|
| Dictation (hold) | `Alt+D` | `Cmd+Shift+D` |
| Grammar Fix | `Alt+G` | `Cmd+Shift+G` |
| Translate | `Alt+T` | `Cmd+Shift+T` |
| Improve | `Alt+I` | `Cmd+Shift+I` |
| Action Wheel | `Alt+J` | `Cmd+Shift+J` |
| Screen Question | `Alt+S` | `Cmd+Shift+S` |

Shortcuts are customizable in Settings → Shortcuts.

## Common Issues

### Shortcut already registered

Another app uses the same hotkey. Change the shortcut in Settings or close the other app.

### Clipboard paste doesn't work

Some elevated/admin apps block simulated input. Run ShortCut as admin (Windows)
or grant Accessibility permission (macOS).

### Microphone not detected

- Check browser permissions in webview
- Grant Microphone permission (macOS: System Settings → Privacy → Microphone)
- Ensure no other app has exclusive mic access

### Global shortcuts don't work (macOS)

System Settings → Privacy → Accessibility → Add ShortCut

### `frontend-dev` has no tray or hotkeys

That is expected. `frontend-dev` runs the Svelte UI only; test tray, global shortcuts, clipboard, microphone, and overlays by running a Docker-built desktop artifact on the target OS.

### Provider not working

1. Verify your API key in Settings → AI Providers
2. Check that the task assignment points to the correct provider (Settings → Task Assignments)
3. For Ollama: ensure `ollama serve` is running and the model is pulled (`ollama pull <model>`)
