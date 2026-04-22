# Changelog

All notable changes to ShortCut are documented here.

## [0.1.0] - Initial Release

### Features

- **Voice-to-text dictation** — hold-to-record with cloud (Soniox) or local
  (Parakeet ONNX) engines; onboarding lets the user pick on first run
- **Text transformations** — grammar fix, translate, improve, all dispatched
  via `transform_text(task, text)` to the provider configured for each task
- **Screen Question** — capture a region and ask a vision-capable provider
- **Action Wheel** — radial menu for action selection
- **Activity indicator** — floating window showing recording/processing state
- **History** — local log of transformations with copy-back support
- **Multi-provider, direct calls, no proxy**:
  - OpenAI — grammar, translate, improve, vision (gpt-4o)
  - Anthropic — grammar, translate, improve, vision (claude-3-5-sonnet)
  - Gemini — grammar, translate, improve, vision (gemini-2.0-flash)
  - Grok (xAI) — grammar, translate, improve
  - Ollama — grammar, translate, improve, local vision models
  - Soniox — cloud dictation via direct 5-step REST API
- **Per-task provider assignment** — each task independently assigns a
  provider + model in Settings
- **App-owned prompt templates** — grammar, translate, and improve prompts
  live in `config.json` and are user-editable
- **Provider readiness dashboard** — per-provider configuration status
- **Per-provider diagnostics** on the Debug page

### Security

- **Gemini auth** uses the `x-goog-api-key` header, not the URL query string
- **Atomic config writes** — `persist_config` writes to a `.tmp` file and
  renames, avoiding crash-mid-write corruption
- **Sanitized upstream errors** — providers surface `HTTP <status>` to the
  frontend; full response bodies stay at debug log level
- **Prompt placeholder validation** — `update_*_config` commands reject
  prompts missing `{text}`

### Configuration

- No credentials required to launch; users configure providers in Settings
- Config file location:
  - Windows: `%APPDATA%\com.g-prompter.shortcut\config.json`
  - macOS: `~/Library/Application Support/com.g-prompter.shortcut/config.json`
