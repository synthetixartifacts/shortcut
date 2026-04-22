# Backend Architecture (Rust/Tauri)

## Directory Structure

```
src-tauri/
├── Cargo.toml              # Rust dependencies
├── tauri.conf.json         # Tauri configuration
├── capabilities/           # Permission declarations
│   └── default.json
└── src/
    ├── main.rs             # Entry point (calls lib::run)
    ├── lib.rs              # App setup, tray, plugins, command registration
    ├── errors.rs           # Unified AppError + ProviderErrorKind (thiserror + Serialize)
    ├── build_config.rs     # Minimal stub — no URL embedding
    ├── text_transform.rs   # transform_text(task, text) Tauri command
    ├── providers/          # LLM provider abstraction layer
    │   ├── mod.rs          # LlmProvider trait, factory, vision dispatch (per-model gate)
    │   ├── http.rs         # Shared HTTP client + ensure_ok/read_sse/read_ndjson helpers
    │   ├── openai.rs       # OpenAI (GPT-4o, GPT-4o-mini, vision); reused by Grok + Local
    │   ├── anthropic.rs    # Anthropic (Claude 3.5, vision)
    │   ├── gemini.rs       # Gemini (2.0 Flash, vision)
    │   ├── grok.rs         # Grok (xAI, per-model vision)
    │   ├── ollama.rs       # Ollama-native chat (one branch of Local)
    │   ├── local.rs        # Local LLM protocol resolution + adapter build
    │   └── discovery/      # Live model listing (split)
    │       ├── mod.rs      #   get_provider_models dispatch
    │       ├── openai.rs, anthropic.rs, gemini.rs, xai.rs, ollama.rs
    │       ├── openai_compat.rs  #   GET {base}/v1/models for LM Studio / vLLM / llama.cpp
    │       ├── local.rs         #   Local dispatcher: ollama OR openai_compat; auto-detect race
    │       └── filters.rs  #   Text/vision capability filtering
    ├── config/             # Settings module
    │   ├── mod.rs          # Persistence (atomic tmp→rename)
    │   ├── commands.rs     # Tauri command handlers
    │   └── types/          # Split config types (was a single types.rs)
    │       ├── mod.rs      #   Re-exports
    │       ├── providers.rs    #   ProvidersConfig, ProviderCredentials, TaskAssignment (supports_vision)
    │       ├── hotkeys.rs      #   HotkeyConfig
    │       ├── dictation.rs    #   DictationConfig
    │       ├── prompts.rs      #   GrammarConfig, TranslateConfig, ImproveConfig, ScreenQuestionConfig (each with system_prompt)
    │       ├── user.rs         #   UserConfig
    │       └── app.rs          #   AppConfig, AppSettingsConfig, BehaviorConfig, TranscriptionConfig
    ├── hotkeys/            # Global shortcut handling
    │   ├── mod.rs          # Constants, Tauri commands, display
    │   ├── parser.rs       # Parse shortcut strings + validation
    │   └── registration.rs # Register/unregister with OS + collision detection
    ├── clipboard.rs        # Clipboard + paste simulation
    ├── transcription/      # STT engine dispatch
    │   ├── mod.rs                # Dispatch command, engine routing
    │   ├── soniox_provider.rs    # Soniox direct API (5-step flow)
    │   ├── soniox_api/           # Soniox HTTP operations (split)
    │   │   ├── mod.rs            #   upload/create/poll/fetch/delete
    │   │   └── types.rs          #   request/response types
    │   ├── utils.rs              # Audio decode helpers
    │   ├── local_provider/       # Parakeet ONNX (local-stt feature, split)
    │   │   ├── mod.rs            #   public transcribe entry point
    │   │   ├── audio.rs          #   WebM/Opus decode + 16kHz resample
    │   │   ├── ort_init.rs       #   ORT_DYLIB_PATH one-shot init (unsafe-wrapped)
    │   │   └── transcribe.rs     #   Parakeet ONNX inference
    │   └── model_manager.rs      # Model download (streaming SHA256) + Content-Length + cancellation
    ├── history.rs          # History CRUD (atomic tmp→rename, 10k entry cap)
    ├── action_menu.rs      # Action wheel window management
    ├── screen_capture.rs   # Screen capture + question overlay (+ macOS TCC probe)
    ├── indicator/          # Activity indicator window (split)
    │   ├── mod.rs          #   Public API + Tauri commands
    │   ├── topology.rs     #   Display-topology tracking + poison-recovery mutex
    │   ├── positioning.rs  #   Bottom-center clamp + monitor math
    │   └── lifecycle.rs    #   show/hide/recreate
    └── window_style.rs     # Platform overlay styles + is_window_healthy + OverlayConfig + build_overlay_window
```

## Module Responsibilities

### `providers/` — LLM Provider Abstraction

The core of the multi-provider architecture. All LLM calls go through this module.

#### `LlmProvider` Trait

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Non-streaming completion — returns full response text
    async fn complete(&self, req: &ChatRequest) -> Result<String, AppError>;

    /// Streaming completion — emits chunks via the provided sink function
    async fn stream(&self, req: &ChatRequest, sink: &EventSinkFn) -> Result<(), AppError>;

    fn capabilities(&self) -> ProviderCapabilities;
    fn provider_id(&self) -> &'static str;
}
```

#### Shared Types

```rust
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub image: Option<ImageAttachment>,  // For vision requests
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

pub struct ProviderCapabilities {
    pub supports_streaming: bool,
    pub supports_vision: bool,
}
```

#### Provider Factory

```rust
/// Instantiate an LLM provider from config.
/// Returns Err if the provider is unknown or its credentials are not configured.
pub fn get_llm_provider(app: &AppHandle, provider_id: &str) -> Result<Box<dyn LlmProvider>, AppError>
```

Supported `provider_id` values: `"openai"`, `"anthropic"`, `"gemini"`, `"grok"`, `"local"`.

Cloud provider routing is fixed in code. The `"local"` arm delegates to `providers::local::build(client, &creds.local)` which resolves the protocol (`auto` → `detected_protocol` fallback; else the explicit `ollama` / `openai_compatible` choice) and constructs either an `OllamaProvider` or an `OpenAiProvider` with the configured base URL. Readiness for Local is gated on `creds.local.base_url` being non-empty — unlike cloud providers, which key off API key presence.

#### Vision / Screen Question Dispatch

```rust
/// Stream a screen question response via the configured vision provider.
/// Emits: "screen-answer-chunk" { content } | "screen-answer-complete" | "screen-answer-error" { error }
pub async fn stream_screen_question(app, image_base64, image_mime_type, messages) -> Result<(), AppError>
```

Reads `task_assignments.screen_question` from config, consults
`TaskAssignment.supports_vision` (per-model override) with a fallback to the
provider-level `ProviderCapabilities::supports_vision`, then streams via
`provider.stream()`. Cancellable via an `Arc<AtomicBool>` stored in app state —
closing the overlay chat window flips the flag and `read_sse` / `read_ndjson`
abort the in-flight stream.

#### Shared HTTP / SSE helpers (`providers/http.rs`)

| Helper | Purpose |
|--------|---------|
| `create_http_client()` | HTTP/1.1-only `reqwest::Client` singleton stored as Tauri state. |
| `ensure_ok(resp) -> Result<Response, AppError>` | Normalizes non-2xx into `AppError::Provider { kind, message }` — `Auth` (401/403), `RateLimit { retry_after_secs }` (429), `InvalidRequest` (400), `Server { status }` (5xx), `Other` otherwise. Error `message` format: `"<provider> <URL> failed: HTTP <status> — body: <preview>"` where `<preview>` is the first ~200 chars of the response body (or `"(empty)"`). |
| `truncate_preview(s, max_chars) -> String` | Truncates a string at `max_chars` user-visible chars and appends `…` on overflow. Used by `ensure_ok` and by `ollama.rs` / `openai.rs` / `discovery/mod.rs::parse_json_response` when formatting parse-error messages so we never dump a full 50 KB body. |
| `read_sse(resp, cancel, on_event)` | Buffers raw bytes across TCP chunks, emits complete `\n\n`-framed events, decodes UTF-8 only once an event is fully buffered (fixes the multi-byte split corruption that plagued the pre-PHASE 3A `String::from_utf8_lossy` loops). |
| `read_ndjson(resp, cancel, on_line)` | Same buffering contract but framed by `\n`, with typed `serde::DeserializeOwned`. Per-line parse errors logged at debug and skipped (Ollama's streaming protocol tolerates this). |

**Error-message convention for Local debugging**: `ollama.rs` and `openai.rs` include the request URL + body preview in transport/parse error messages, and `discovery/mod.rs::parse_json_response` follows the same pattern. Transport errors from `reqwest::Error` surface `.url()` when available. The goal is that any debug-log copy/paste points directly at the failing endpoint without needing a packet capture.

Both streaming helpers accept an optional `Arc<AtomicBool>` cancellation token
so callers (e.g. `stream_screen_question`) can abort cleanly when the target UI
window closes. Provider files (`openai.rs`, `anthropic.rs`, `gemini.rs`,
`grok.rs`, `ollama.rs`, and the `local.rs` dispatcher) only contain body shape
+ chunk parsing logic — the networking plumbing is centralized.

### `transcription/` — STT Engine Dispatch

Routes `transcribe_audio` to the active engine based on `config.transcription.active_engine`.

| Engine ID | Provider | File |
|-----------|----------|------|
| `"soniox"` | Soniox direct API | `soniox_provider.rs` |
| `"local-windows"` | Parakeet ONNX (Windows) | `local_provider.rs` (`local-stt` feature) |
| `"local-macos"` | Not yet implemented | stub returning error |

**Soniox 5-step flow** (`soniox_provider.rs`):
1. `POST /v1/files` — upload audio file
2. `POST /v1/transcriptions` — create transcription job
3. Poll `GET /v1/transcriptions/{id}` — wait for "completed"
4. `GET /v1/transcriptions/{id}/transcript` — fetch text
5. `DELETE /v1/files/{file_id}` — fire-and-forget cleanup

**Local STT** (`local_provider.rs`, `local-stt` feature):
WebM/Opus → matroska-demuxer → opus-rs → resample to 16kHz mono → transcribe-rs Parakeet ONNX

Model management commands (`get_model_status`, `download_model`, `delete_model`, `cancel_model_download`) are always compiled as stubs so `generate_handler!` works — returns `{ "state": "unavailable" }` when `local-stt` feature is disabled.

### `config/` — Settings Persistence

**Config location:**
- Windows: `%APPDATA%\com.g-prompter.shortcut\config.json`
- macOS: `~/Library/Application Support/com.g-prompter.shortcut\config.json`
- Linux: `~/.local/share/com.g-prompter.shortcut/config.json`

**Config structure:**
```rust
AppConfig {
    hotkeys: HotkeyConfig,
    behavior: BehaviorConfig,
    user: UserConfig,
    dictation: DictationConfig,
    app_settings: AppSettingsConfig,    // theme, language, debug_enabled
    improve: ImproveConfig,             // prompt + system_prompt
    transcription: TranscriptionConfig, // active_engine, first_run_completed
    providers: ProvidersConfig,         // credentials + task_assignments
    grammar: GrammarConfig,             // prompt + system_prompt
    translate: TranslateConfig,         // prompt + system_prompt
    screen_question: ScreenQuestionConfig,  // system_prompt only
    local_detection_schema_version: u32,    // one-shot migration marker (Local detect cache)
}

ProvidersConfig {
    credentials: ProviderCredentials {
        openai_api_key: String,
        anthropic_api_key: String,
        gemini_api_key: String,
        grok_api_key: String,
        soniox_api_key: String,
        local: LocalCredentials {
            base_url: String,                  // e.g. "http://localhost:11434"
            protocol: String,                  // "auto" | "ollama" | "openai_compatible"
            detected_protocol: Option<String>, // cached auto-detect winner
            api_key: Option<String>,           // optional, openai_compatible only
        },
        // Legacy fields retained for read-time migration only:
        ollama_base_url: String,               // migrated → local.base_url
        openai_base_url: String,               // legacy hidden field, ignored
        soniox_base_url: String,               // legacy hidden field, ignored
    },
    task_assignments: TaskAssignments {
        grammar:         TaskAssignment { provider_id, model, supports_vision },
        translate:       TaskAssignment { provider_id, model, supports_vision },
        improve:         TaskAssignment { provider_id, model, supports_vision },
        screen_question: TaskAssignment { provider_id, model, supports_vision },
    }
}
```

**Config migration** (`src-tauri/src/config/mod.rs::migrate_providers_config`) — an idempotent read-time migration applies four passes:

1. Copy legacy `ollama_base_url` into `local.base_url` (when `local` is still default) and clear the legacy field.
2. Rewrite any `task_assignment.provider_id == "ollama"` to `"local"`.
3. Backfill `local.protocol = "auto"` if empty.
4. **One-shot schema-version bump** — if `local_detection_schema_version < 1`, clear `local.detected_protocol` and bump the marker to `1`. Purpose: unstick users whose pre-shape-check auto-detect had cached `detected_protocol = "ollama"` against what is actually an LM Studio endpoint (the old probe accepted any 2xx). The next discovery re-runs the shape-aware race. Subsequent loads see `marker >= 1` and skip the clear, preserving legitimate detection.

Running the migration twice on the same config yields the exact same bytes (`migration_is_idempotent` test). Constant: `LOCAL_DETECTION_SCHEMA_VERSION` in `config/mod.rs`.

Default task assignments: all tasks → OpenAI (`gpt-4o-mini` for grammar/translate, `gpt-4o` for improve/screen).

**Atomic persistence**: `persist_config` writes to `config.json.tmp` and then atomically renames it to `config.json`, so a crash mid-write cannot leave a truncated or corrupt config file on disk.

**Prompt template validation**: `update_grammar_config`, `update_translate_config`, and `update_improve_config` validate that the submitted prompt contains the `{text}` placeholder and return an error without persisting if the placeholder is missing. **System prompts are never placeholder-validated** — they are sent verbatim. `update_screen_question_config` has no placeholder validation (system prompt only, no user template). Translation target language is expressed in the translate system prompt; there is no dedicated `target_language` field.

**Per-action system prompts**: `ImproveConfig`, `GrammarConfig`, and `TranslateConfig` each carry a `system_prompt: String` field alongside their user-prompt template. `ScreenQuestionConfig` has only `system_prompt`. Defaults are authored in Rust via `default_{grammar,translate,improve,screen_question}_system_prompt()` in `config/types/prompts.rs` — the frontend's Reset button fetches these through the `get_default_*_config` commands. Empty string semantics: an empty `system_prompt` emits **no** system message at dispatch time (backward compat).

**Dispatch**: `text_transform::transform_text` reads the task's `system_prompt` alongside the rendered user prompt; when non-empty, a `ChatMessage { role: "system", content }` is prepended before the `user` message. `screen_capture::send_screen_question` does the same with `config.screen_question.system_prompt` before calling `providers::stream_screen_question`. Providers translate `role: "system"` into their native shape — see [PROVIDERS.md](./PROVIDERS.md#system-role-handling-per-provider).

### `hotkeys/` — Global Shortcuts

| File | Purpose |
|------|---------|
| `mod.rs` | Constants, Tauri commands, display formatting |
| `parser.rs` | Parse + validate shortcut strings ("Alt+D") |
| `registration.rs` | Register/unregister with OS + collision detection |

**String validation** (`parser.rs`): every shortcut string is parsed to a
normalized `(modifiers, code)` tuple before the config is accepted. Unknown
tokens or modifier-less bindings are rejected with `AppError::Config`.

**Collision detection** (`registration.rs::register_shortcuts_from_config`):
before calling the OS registration API, the config is scanned for two bindings
that resolve to the same physical chord. On collision the duplicate is skipped
and the conflict is surfaced via `log::warn!` so the UI banner can alert the
user.

Default shortcuts (Windows/Linux):
```
Alt+D  — dictation
Alt+G  — grammar
Alt+T  — translate
Alt+I  — improve
Alt+J  — action wheel
Alt+S  — screen question
```
macOS: `Cmd+Shift+<key>` for each action.

Emits `"shortcut-triggered"` Tauri events with the action name.

### `window_style.rs` — Overlay Window Helpers

Shared scaffolding for the three overlay windows (indicator, action-menu,
screen-question). Before PHASE 3A each overlay duplicated ~30 lines of
`WebviewWindowBuilder` boilerplate and platform quirks.

| Helper | Purpose |
|--------|---------|
| `is_window_healthy(window, context)` | Cheap handle liveness check (`is_visible()` + `scale_factor()`). Used on every show to decide whether to recreate after a display-topology change. |
| `OverlayConfig<'a>` | Declarative struct (`title`, `width`, `height`, `focused`) capturing what varies between overlays. |
| `build_overlay_window(app, label, url_path, config)` | Builds a transparent, borderless, always-on-top overlay using the declarative config. Callers follow up with `apply_non_focusable` / `apply_mouse_no_activate` as appropriate. |
| `apply_non_focusable(app, label)` / `apply_indicator_non_focusable(app)` | Windows: `WS_EX_NOACTIVATE`; macOS: collection behavior flags. |
| `show_without_focus_steal(window)` / `hide_overlay_window(window)` | Platform-correct show/hide without stealing focus from the target app. |

### `errors.rs` — Unified Error Type

```rust
enum AppError {
    Io(#[from] std::io::Error),
    Network(#[from] reqwest::Error),
    Config(String),
    Serialization(#[from] serde_json::Error),
    General(String),
    ProviderError(String),              // legacy string-only variant
    Provider { kind: ProviderErrorKind, message: String },  // structured variant
}

enum ProviderErrorKind {
    Auth,                                   // 401/403
    RateLimit { retry_after_secs: Option<u64> },  // 429, parses Retry-After
    InvalidRequest,                         // 400
    Server { status: u16 },                 // 5xx
    Network,                                // transport-layer failure
    Parse,                                  // JSON/SSE/UTF-8 decode
    Other,
}
```

`AppError::Provider` serializes to JSON as
`{ error_type: "provider", kind, message }` — the frontend can branch on
`kind` instead of string-matching error messages (Auth → reconfigure,
RateLimit → backoff hint, etc.). Every other variant serializes as its
`Display` string for backward compatibility.

Migration to `AppError` is complete for history, hotkeys, and the provider
beachhead. The remaining `Result<_, String>` sites in `screen_capture.rs`,
`clipboard.rs`, `window_style.rs`, `indicator/*`, `config/commands.rs`, and
`transcription/*` are documented as a follow-up long-tail migration.

### `clipboard.rs` — Clipboard Operations

Uses `enigo` for cross-platform keyboard simulation.

**Key commands:**
```rust
paste_text(app, text)              // Write to clipboard + Ctrl+V
get_selection_with_format(app)     // Save → Ctrl+C → read (HTML→Markdown detection)
paste_formatted(app, text, format) // Paste with optional Markdown→HTML conversion
frontend_log(message)              // Log from frontend to Rust
```

### `screen_capture.rs` — Screen Capture & Question Window

**Image pipeline:** Capture (xcap) → Resize (max 2048px, Lanczos3) → RGBA→RGB → JPEG (quality 85) → Base64

Window management: unlike indicator/action-menu, screen-question window needs focus (user types). Uses `.show()` + `.set_focus()` instead of `show_without_focus_steal()`.

### `build_config.rs`

Minimal stub — no URL embedding. Production builds differ from dev only in optimization flags and code signing.

## Tauri Commands Reference

### Transform Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `transform_text` | `task: String, text: String` | `String` | Transform text via task-assigned provider |

`task` values: `"grammar"`, `"translate"`, `"improve"`.

The unified `transform_text(task, text)` command handles all LLM text transforms. The deprecated aliases (`fix_grammar`, `translate_text`, `improve_text`) and their frontend wrappers were removed in the PHASE 4 dead-code cleanup — only `transform_text` is registered in `lib.rs::invoke_handler!`.

**Removed commands** (previously registered, now deleted — unused by frontend): `get_user_config`, `is_action_menu_visible`, `hide_main_window`.

### Provider Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `get_providers_config` | — | `ProvidersConfig` | Get provider credentials + task assignments |
| `update_providers_config` | `providers: ProvidersConfig` | — | Save provider config |
| `get_provider_status` | — | `ProviderStatusReport` | Per-provider readiness (local config, no network) |

### Transcription Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `transcribe_audio` | `audioPath?, mimeType, languageHints, contextTerms, contextText?` | `TranscriptionData` | Transcribe via active engine |
| `get_model_status` | — | `ModelStatus` | Local model status |
| `download_model` | — | — | Start model download |
| `delete_model` | — | — | Delete model files |
| `cancel_model_download` | — | — | Cancel download |

### Configuration Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `get_config` | — | `AppConfig` | Get full config |
| `save_config` | `config: AppConfig` | — | Save full config |
| `update_user_config` | `user: UserConfig` | — | Update user profile |
| `update_dictation_config` | `dictation: DictationConfig` | — | Update dictation settings |
| `update_hotkeys_config` | `hotkeys: HotkeyConfig` | — | Update shortcut bindings |
| `update_app_settings_config` | `appSettings: AppSettingsConfig` | — | Update theme/language/debug |
| `update_improve_config` | `improve: ImproveConfig` | — | Update improve prompt + system_prompt |
| `update_grammar_config` | `grammar: GrammarConfig` | — | Update grammar prompt + system_prompt |
| `update_translate_config` | `translate: TranslateConfig` | — | Update translate prompt + system_prompt |
| `update_screen_question_config` | `screenQuestion: ScreenQuestionConfig` | — | Update screen-question system_prompt |
| `get_default_improve_config` | — | `ImproveConfig` | Default improve config (for reset) |
| `get_default_grammar_config` | — | `GrammarConfig` | Default grammar config (for reset) |
| `get_default_translate_config` | — | `TranslateConfig` | Default translate config (for reset) |
| `get_default_screen_question_config` | — | `ScreenQuestionConfig` | Default screen-question config (for reset) |
| `get_active_engine` | — | `String` | Active STT engine ID |
| `set_active_engine` | `engine: String` | — | Set active STT engine |
| `update_transcription_config` | `transcription: TranscriptionConfig` | — | Update transcription config |

### Shortcut Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `get_registered_shortcuts` | — | `ShortcutInfo[]` | All registered shortcuts |
| `update_shortcuts` | `hotkeys: HotkeyConfig` | — | Update shortcuts at runtime |
| `get_default_shortcuts` | — | `HotkeyConfig` | Default shortcut config |

### Clipboard Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `paste_text` | `text: String` | — | Paste text to focused app |
| `get_selection_with_format` | — | `SelectionResult` | Copy selection with HTML→Markdown detection |
| `paste_formatted` | `text, format` | — | Paste with optional Markdown→HTML conversion |
| `frontend_log` | `message: String` | — | Log from frontend to Rust |

### History Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `get_history` | `page, pageSize, query?` | `HistoryPage` | Get paginated history |
| `add_history_entry` | `text, durationMs, language?, engine?` | `HistoryEntry` | Add entry |
| `delete_history_entry` | `id: String` | — | Delete entry |
| `clear_history` | — | — | Clear all history |

### Window Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `show_indicator` | — | — | Show activity indicator |
| `hide_indicator` | — | — | Hide activity indicator |
| `reset_indicator` | — | — | Force-recreate indicator window |
| `toggle_action_menu` | — | — | Show at cursor / hide |
| `hide_action_menu` | — | — | Hide action menu |
| `screen_question` | — | — | Capture + show overlay (toggle) |
| `send_screen_question` | `imageBase64, imageMimeType, messages` | — | Stream AI response |
| `hide_screen_question` | — | — | Hide overlay |

## Plugin Configuration

In `tauri.conf.json`:
```json
{ "plugins": {} }
```

Do NOT use `"plugins": { "plugin-name": {} }` — causes crash.

Plugins initialized in `lib.rs`:
```rust
.plugin(tauri_plugin_single_instance::init(...))  // Must be first
.plugin(tauri_plugin_global_shortcut::Builder::new().build())
.plugin(tauri_plugin_clipboard_manager::init())
.plugin(tauri_plugin_positioner::init())
```

## Platform Notes

### Windows
- WebView2 required (pre-installed on Win10+)
- Uses SendInput for keyboard simulation
- Local STT available (`local-stt` feature)

### macOS
- Requires Accessibility permission (System Settings → Privacy → Accessibility)
- Uses CGEventPost for simulation
- `show_main_window` command forces window visible before `getUserMedia()` (WebKit visibility requirement)

### Linux
- X11: Uses libxdo (`libxdo-dev`)
- Wayland: Limited support via libei
