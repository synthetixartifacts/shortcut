# ShortCut Architecture

## System Overview

ShortCut is a system-tray desktop application built with Tauri 2. It runs as a background service with global hotkeys for voice dictation and AI text transformation. All AI calls go directly to the configured provider — no proxy, no token gate.

### Why Tauri 2?

| Criteria | Tauri 2 | Electron |
|----------|---------|----------|
| Bundle size | 2-5 MB | 80-120 MB |
| RAM usage | 20-50 MB | 200-400 MB |
| Startup time | <500ms | 1-2s |

For a tray utility that runs constantly, Tauri's smaller footprint is ideal.

## High-Level Architecture

```
┌──────────────┐     ┌─────────────────┐     ┌──────────────┐
│ System Tray  │◄───►│  Tauri Backend  │◄───►│  Webview UI  │
│ (always on)  │     │     (Rust)      │     │  (Svelte 5)  │
└──────────────┘     └────────┬────────┘     └──────────────┘
                              │                     │
                    ┌─────────┴──────────┐          │ Tauri Events
                    │                    │
          ┌─────────▼──────────┐   ┌────▼────────────┐
          │  providers/        │   │  transcription/ │
          │  LLM Providers     │   │  STT Providers  │
          │  ─────────────     │   │  ─────────────  │
          │  OpenAI            │   │  Soniox (cloud) │
          │  Anthropic         │   │  Parakeet ONNX  │
          │  Gemini            │   │  (Windows local)│
          │  Grok              │   └─────────────────┘
          │  Local ─► ollama  │
          │        └► openai- │
          │           compat  │
          └────────────────────┘
                    │
         ┌──────────┼──────────────────────────────┐
         ▼          ▼                              ▼
     OpenAI    Anthropic        Gemini / Grok / Local
     API        API                    APIs
```

Overlay windows (indicator, action menu, screen question) communicate via Tauri events. LLM and STT providers are called directly from Rust — no intermediary proxy.

**Local LLM** is a single provider slot with a runtime-resolved protocol: `ollama` (native `/api/chat`) or `openai_compatible` (LM Studio, LocalAI, vLLM, llama.cpp server, any `/v1/chat/completions` endpoint). The factory dispatch (`providers/local.rs::build`) reads `creds.local.protocol` — if set to `Auto`, it falls back to the cached `detected_protocol` resolved by `discovery::local::fetch_local_models`, which races `/api/tags` and `/v1/models` in parallel and accepts each only if the 2xx body has the expected JSON shape (`{"models": [...]}` or `{"data": [...]}` — rejects permissive catchalls). Ollama wins ties; if both fail, a typed `AppError::Provider { kind: Network }` surfaces with both URLs and the cache stays empty. The user-entered base URL is normalized via `normalize_local_base_url` (strips known endpoint suffixes longest-first) before every request. Readiness is gated on "URL non-empty", not on a key — unlike cloud providers.

## Provider Abstraction

The `providers/` module implements the `LlmProvider` trait:

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, req: &ChatRequest) -> Result<String, AppError>;
    async fn stream(&self, req: &ChatRequest, sink: &EventSinkFn) -> Result<(), AppError>;
    fn capabilities(&self) -> ProviderCapabilities;
    fn provider_id(&self) -> &'static str;
}
```

`get_llm_provider(app, provider_id)` instantiates the correct provider from config.
Task dispatch reads `config.providers.task_assignments` to determine which provider
and model handles each task (grammar, translate, improve, screen_question).

See [BACKEND.md](./BACKEND.md) and [PROVIDERS.md](./PROVIDERS.md) for details.

## Per-Action System Prompts

Every text-transform action carries two editable prompts:

- **System prompt** — sets the model's role/behavior. Rendered as a `role: "system"` `ChatMessage` prepended before the user message when non-empty. Empty string → no system message emitted (backward compat).
- **User prompt template** — rendered with `{text}`. Always present for grammar/translate/improve; screen-question has no user-prompt template. Translation target language is expressed in the translate **system prompt**.

**Source of truth for defaults** — defaults live in Rust in `src-tauri/src/config/types/prompts.rs`:

| Action | Default-fn | Config struct / field |
|--------|-----------|-----------------------|
| Grammar | `default_grammar_system_prompt()` / `default_grammar_prompt()` | `GrammarConfig { prompt, system_prompt }` |
| Translate | `default_translate_system_prompt()` / `default_translate_prompt()` | `TranslateConfig { prompt, system_prompt }` |
| Improve | `default_improve_system_prompt()` / `default_improve_prompt()` | `ImproveConfig { prompt, system_prompt }` |
| Screen Question | `default_screen_question_system_prompt()` | `ScreenQuestionConfig { system_prompt }` — user's typed question is the user message |

**Persistence paths in `config.json`:** `grammar.system_prompt`, `translate.system_prompt`, `improve.system_prompt`, `screen_question.system_prompt`.

**Frontend editing surface** — each action has a dedicated settings page at `/actions/{grammar,translate,improve,screen-question}` that edits the pair (or system prompt only) plus the task's provider/model assignment via the shared `providersSettingsState`. The `/settings` page remains the credentials + full task-assignment matrix hub. Both surfaces write the same `TaskAssignment` rows — single source of truth.

All settings surfaces use per-field auto-save with inline `<SaveIndicator>` feedback (see [FRONTEND.md → Save Feedback Pattern](./FRONTEND.md#save-feedback-pattern)). No Save buttons on `/settings/providers`, `/actions/*`, `/app-settings`, or `/shortcuts` (the shortcut-editor modal keeps its internal saving indicator since it's transient). Credentials debounce at 500 ms; task assignments, toggles, and selects persist immediately on change.

**Reset** — the Reset button calls `get_default_{action}_config` and persists the returned default via `update_{action}_config`.

### Dispatch flow with system prompt

```
transform_text(task, text)                          send_screen_question(img, user_msgs)
  │                                                  │
  │ read config.{task}.{prompt, system_prompt}       │ read config.screen_question.system_prompt
  │ render prompt with {text}                        │
  ▼                                                  ▼
Build messages:                                      Build full_messages:
  if !system_prompt.is_empty():                        if !system_prompt.is_empty():
    push { role: "system", content: system_prompt }     push { role: "system", content }
  push { role: "user", content: rendered_prompt }      extend(user_msgs)
  │                                                  │
  ▼                                                  ▼
get_llm_provider(pid).complete(req)                  stream_screen_question(app, img, full_messages)
                                                       └─► provider.stream(req, sink)
```

Each provider translates `role: "system"` into its native shape — see [PROVIDERS.md](./PROVIDERS.md#system-role-handling-per-provider).

## Auth Model

There is no authentication gate. The app launches without requiring any credentials.
Users configure API keys in Settings → AI Providers. Provider readiness is tracked by
`state/providers.svelte.ts`, which reads local config only (no network calls).

```
App Launch
    │
    ▼
Load config from config.json
    │
    ▼
Check provider readiness (local config only)
    │
    ├── At least one LLM + STT configured → Dashboard (ready)
    │
    └── No providers configured → Dashboard (setup banner)
                                         │
                                         ▼
                               Settings → AI Providers
                               (add API keys)
```

API keys are stored in plain JSON in `{appData}/shortcut/config.json`. Use
OS-level disk encryption for sensitive deployments.

## Settings Persistence

Every settings surface (Providers, per-action Actions pages, App-Settings, Shortcuts, Dashboard microphone) persists via per-field auto-save:

```
user event
   │
   ▼ (optional debounce for typed input — SAVE_DEBOUNCE_MS = 500 ms)
state module's save<Field>(value)
   │
   ▼
withAsyncState(state, fn, { onSaving, onSaved, onError })
   │ ├── onSaving  → saveStatus[key].markSaving()
   │ ├── await update<Feature>Config(...)  (Tauri command)
   │ ├── onSaved   → saveStatus[key].markSaved()   (auto-reverts after 2 s)
   │ └── onError   → saveStatus[key].markError(msg) (sticks until next save)
   │
   ▼
<FormField saveStatus=…> or <SaveIndicator status=…>
```

The Providers page is fully auto-save. `saveProviderCredential(providerId, field, value)` writes the single-field change, and on success re-runs `refreshProviderModels(providerId)` for LLM providers whose credentials just changed — the passive readiness re-check that replaces the old explicit Save + manual refresh flow. `saveTaskAssignment(taskKey)` persists task-assignment rows immediately (no debounce — one user gesture = one Tauri write).

## App Settings

Stored in `AppConfig.app_settings`:

| Setting | Values | Default | Managed By |
|---------|--------|---------|------------|
| Theme | `"light"`, `"dark"` | `"light"` | CSS `[data-theme]` on `<html>` |
| Language | `"en"`, `"fr"`, `"es"` | `"en"` | Custom i18n system (`t()` function) |
| Debug enabled | `true`, `false` | `false` | Sidebar conditional nav item |

## Data Flows

### Dictation Flow (Hold-to-Talk)

```
User holds Alt+D
        │
        ▼
Rust: Global Shortcut Handler
        │ emits "shortcut-triggered: dictation_start"
        ▼
Frontend: ShortcutDispatcher
        │
        ▼
DictationController.startRecording()
        │ reads dictation-config.svelte.ts
        ▼
AudioRecorder captures audio (MediaRecorder API)
        │
User releases Alt+D
        │ emits "shortcut-triggered: dictation_stop"
        ▼
DictationController.stopRecording()
        │
        ▼
Audio written to temp file (avoids IPC size limit)
        │
        ▼
Rust: transcription::transcribe_audio()
        │ reads active_engine from config:
        │   "soniox"        → soniox_provider.rs → Soniox API (5-step)
        │   "local-windows" → local_provider.rs  → Parakeet ONNX (on-device)
        ▼
TranscriptionData { text, duration_ms, language, engine }
        │
        ▼
pasteText() → Rust clipboard + Ctrl+V simulation
```

### Transform Flow (Grammar / Translate / Improve)

```
User selects text in any app
        │
User presses Alt+G (or T, I)         (or selects from Action Wheel)
        │
        ▼
Rust: Global Shortcut Handler emits "shortcut-triggered: grammar"
        │
        ▼
Frontend: ShortcutDispatcher → handleGrammarFix()
        │ (the same handler runs for Action-Wheel selection
        │  via the menu-action-selected event — single dispatch path)
        ▼
Rust: save clipboard → Ctrl+C → read selected text (with format detection)
        │
        ▼
Rust: transform_text("grammar", text)
        │ reads task_assignments.grammar.{provider_id, model}
        │ instantiates provider via get_llm_provider()
        │ calls provider.complete(req) with grammar prompt + text
        ▼
Transformed text returned
        │
        ▼
Rust: write clipboard → Ctrl+V (format-aware paste) → restore original clipboard
        │
        ▼
Frontend: base-controller records the result to text-transform history
        │ (single integration hook in `features/text-transform/base-controller.ts`)
        │ — invokes addTextTransformHistoryEntry(action, result) AFTER paste
        │ — wrapped in its own try/catch: history failure NEVER blocks paste
        │ — gated on non-empty result text
        │ — refreshTextTransformHistory() then surfaces newest entries on the
        │   /text-transform-history page if it's currently open
        ▼
Persisted to text_transform_history.json (separate domain from dictation history;
  see docs/features/TEXT_TRANSFORM_HISTORY.md)
```

**Why this hook lives in the frontend, not in `transform_text`**: the Rust
`transform_text` command remains a pure transform — it returns the result string
and nothing else. The history record only happens after `pasteFormatted`
succeeds, so we never persist a result the user didn't actually receive. Both
direct shortcuts and the Action-Wheel selection flow run through the same
`createTextTransformHandler` factory, so a single hook covers all six entry
points (3 actions × 2 entry types). `screen_question` is a separate streaming
pipeline (`stream_screen_question`) and is **not** recorded — by scope.

### Screen Question Flow

```
User presses Alt+S (or selects from Action Wheel)
        │
        ▼
Rust: screen_capture::screen_question()
        │ 1. Toggle: if overlay already visible, hide and return
        │ 2. Capture screenshot (xcap → resize → JPEG → base64)
        │ 3. Show overlay window (centered, with focus)
        │ 4. Emit "screen-captured" event
        ▼
Frontend: OverlayChat renders with screenshot thumbnail
        │
User types question, presses Enter
        │
        ▼
invoke("send_screen_question", { image_base64, image_mime_type, messages })
        │
        ▼
Rust: providers::stream_screen_question()
        │ reads task_assignments.screen_question.{provider_id, model}
        │ validates provider supports vision
        │ calls provider.stream() with image attachment
        ▼
Events: "screen-answer-chunk" → "screen-answer-complete"
        │
        ▼
Frontend: appends chunks to streaming message
```

### Activity Indicator Flow

```
Feature triggers activity (grammar, translate, improve, dictation)
        │
        ▼
startActivity() in activity.svelte.ts
        │
        ├── invoke('show_indicator') → Rust positions + shows indicator window
        │
        └── emit('indicator-update', { type, state })
                  │
                  ▼
             Indicator window updates UI (animated dots, colors)
        │
        ▼
Operation completes → updateActivity('success') or ('error')
        │
        └── Auto-hide after delay (1s success, 2s error)
```

## Key Technical Decisions

### Hand-rolled HTTP (not JS SDKs)

The CSP is `connect-src 'self'`. All AI provider calls happen from Rust using
`reqwest` with a shared HTTP client (`providers/http.rs`). No CSP relaxation needed,
no JS AI SDK dependencies.

### Audio Pipeline

Audio recording uses the browser's MediaRecorder API in the frontend. The audio
blob is written to a temp file and the path is sent to Rust (avoids IPC size limits).
Rust dispatches to the active transcription engine. The frontend path is identical
regardless of engine.

### Clipboard Save/Restore Pattern

```rust
let saved = clipboard.read_text()?;
clipboard.write_text(transformed_text)?;
simulate_paste();
sleep(Duration::from_millis(50));
clipboard.write_text(saved)?;   // Restore original
```

### Local STT (On-Device)

When `active_engine = "local-windows"`:
- Audio decoded from WebM/Opus to 16kHz mono PCM in Rust
- Transcription runs via `transcribe-rs` with Parakeet TDT 0.6B v3 ONNX model
- No network requests during transcription
- Requires `local-stt` Cargo feature at build time
- Windows only; macOS support planned

Model files (~670 MB) downloaded from HuggingFace, stored in app data directory.
See [features/LOCAL_STT.md](./features/LOCAL_STT.md).

## Security

### API Key Storage

API keys are stored in plain JSON at:
- Windows: `%APPDATA%\com.g-prompter.shortcut\config.json`
- macOS: `~/Library/Application Support/com.g-prompter.shortcut/config.json`
- Linux: `~/.local/share/com.g-prompter.shortcut/config.json`

For sensitive deployments, use OS-level disk encryption. `tauri-plugin-stronghold`
can be added in a follow-up without breaking the config interface.

### Tauri Permissions

Declared in `capabilities/default.json`:
- `global-shortcut:allow-register/unregister`
- `clipboard-manager:allow-read-text/write-text`
- `positioner:default` — for indicator window positioning
- `core:window:allow-show/hide/set-position/set-focus`
- `core:event:default`
