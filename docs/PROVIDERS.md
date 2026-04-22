# ShortCut Providers Guide

ShortCut calls AI providers directly from the Rust backend — no proxy, no token gate.
Configure providers in **Settings → AI Providers** and assign tasks in **Settings → Task Assignments**.

The **model dropdowns are loaded live from the provider APIs**, not from a fixed catalog:
- OpenAI: `GET /v1/models`
- Anthropic: `GET /v1/models`
- Gemini: `GET /v1beta/models`
- xAI: `GET /v1/language-models`
- Local/Ollama: `GET /api/tags` plus `POST /api/show`

The model examples below are current as of **April 15, 2026** and are only examples. The in-app dropdown is the source of truth for what your key or local server can actually use.

## LLM Providers

LLM providers handle text transformation tasks: grammar, translate, improve, and screen question.

---

### OpenAI

**Get key**: [platform.openai.com](https://platform.openai.com) → API Keys

**Recommended models:**
| Use Case | Model | Notes |
|----------|-------|-------|
| Grammar, Translate | `gpt-4o-mini` | Current strong low-latency default |
| Improve | `gpt-4o` | Higher quality |
| Screen Question | `gpt-4o` | Vision-capable and higher quality |

**Vision support**: Yes (current GPT-5.x / GPT-4.1 / GPT-4o text models accept image input)

**Settings fields:**
- **API Key** (required)

**Default task assignments**: All tasks default to OpenAI (`gpt-4o-mini` for grammar/translate, `gpt-4o` for improve/screen question).

---

### Anthropic

**Get key**: [console.anthropic.com](https://console.anthropic.com) → API Keys

**Recommended models:**
| Use Case | Model |
|----------|-------|
| Grammar, Translate | `claude-3-5-haiku-20241022` |
| Improve, Screen Question | `claude-sonnet-4-20250514` |

**Vision support**: Yes (all Claude 3+ models)

**Settings fields:**
- **API Key** (required)

---

### Gemini

**Get key**: [aistudio.google.com](https://aistudio.google.com) → Get API Key

**Recommended models:**
| Use Case | Model |
|----------|-------|
| Grammar, Translate | `gemini-2.5-flash` |
| Improve | `gemini-2.5-pro` |
| Screen Question | `gemini-2.5-flash` |

**Vision support**: Yes (current Gemini text-generation models returned by `models.list` support multimodal input)

**Auth**: API key is sent via the `x-goog-api-key` HTTP header (not in the URL query string), so it never appears in request URLs or server logs.

**Settings fields:**
- **API Key** (required)

---

### Grok (xAI)

**Get key**: [console.x.ai](https://console.x.ai) → API Keys

**Recommended models:**
| Use Case | Model |
|----------|-------|
| Grammar, Translate | `grok-4-fast-non-reasoning` |
| Improve | `grok-4` |
| Screen Question | `grok-4.20-beta-latest-non-reasoning` or another image-capable Grok model returned by your API key |

**Vision support**: Yes, for Grok models whose xAI metadata includes image input support.

**Settings fields:**
- **API Key** (required)

---

### Ollama (local)

Ollama runs AI models locally. No API key required, no data leaves your machine.

**Setup:**
1. Install Ollama: [ollama.com](https://ollama.com)
2. Start the server: `ollama serve`
3. Pull a model: `ollama pull gemma3` (or any supported model)

**Recommended models:**
| Use Case | Model | Notes |
|----------|-------|-------|
| Grammar, Translate | `gemma3` | Good general local default |
| Improve | `gpt-oss:20b` or a larger local reasoning model | Depends heavily on hardware |
| Screen Question | `gemma3`, `llava`, or another installed vision model | Must expose the `vision` capability |

**Vision support**: Yes, with locally installed models that advertise the `vision` capability.

**Settings fields:**
- **Local URL** — full chat endpoint, default `http://localhost:11434/api/chat`
- No API key required

**Note**: Performance depends heavily on your hardware. For acceptable speed, a GPU with 8+ GB VRAM is recommended for 7B+ models.

---

## STT Providers

STT (Speech-to-Text) providers handle voice dictation.

---

### Soniox (cloud)

Direct Soniox API integration — no proxy.

**Get key**: [soniox.com/dashboard](https://soniox.com/dashboard)

**Features:**
- Language auto-detection
- Custom vocabulary terms
- Background context text
- Translation mode (real-time speech translation)
- Multi-speaker diarization

**Flow (5-step API):**
1. Upload audio file (`POST /v1/files`)
2. Create transcription job (`POST /v1/transcriptions`)
3. Poll for completion (`GET /v1/transcriptions/{id}`)
4. Fetch transcript (`GET /v1/transcriptions/{id}/transcript`)
5. Delete file (`DELETE /v1/files/{file_id}`)

**Settings fields:**
- **API Key** (required)

**Engine ID**: `"soniox"` (default)

---

### Local (Parakeet ONNX)

On-device transcription using NVIDIA Parakeet TDT 0.6B v3. Audio never leaves your machine.

**Platform**: Windows only (macOS planned)
**No API key required**
**Works offline**

**Setup:**
1. Open Settings → Actions → Dictation
2. Under "Transcription Engine", find "Local (Windows)"
3. Click **Download model** (~670 MB from HuggingFace)
4. Once downloaded, click **Make active**

**Limitations:**
- English-primary accuracy
- No custom vocabulary, background context, or translation
- Slower on weak CPUs (RTF > 1.0 triggers a slowness warning)

**Engine ID**: `"local-windows"`

See [features/LOCAL_STT.md](./features/LOCAL_STT.md) for full details.

---

## Request Parameters

All five LLM providers (OpenAI, Anthropic, Gemini, Grok, Ollama) honor `temperature` and `max_tokens` from the `ChatRequest` when set. Previously only OpenAI and Grok forwarded them; Anthropic, Gemini, and Ollama silently dropped them.

Upstream API error bodies are never echoed to the user — the backend logs the full body at debug level and surfaces only `HTTP <status>` to the frontend to avoid leaking keys or prompt fragments.

### Shared HTTP/SSE scaffolding

Every provider's `complete()` and `stream()` path delegates to three helpers in `providers/http.rs` — **do not hand-roll a streaming loop**:

| Helper | Purpose |
|--------|---------|
| `ensure_ok(resp)` | Converts non-2xx responses into a classified `AppError::Provider { kind, message }` (`Auth`, `RateLimit { retry_after_secs }`, `InvalidRequest`, `Server { status }`, `Other`). Parses `Retry-After` for 429s. |
| `read_sse(resp, cancel, on_event)` | Buffers bytes across TCP chunks, yields complete `\n\n`-framed events, decodes UTF-8 only on full events (fixes multi-byte-char corruption). Cancellable via `Arc<AtomicBool>`. |
| `read_ndjson(resp, cancel, on_line)` | Same buffering contract but framed by `\n`, with typed deserialization. Parse errors on individual lines are logged at debug and skipped (matches Ollama's protocol). |

Vendor-specific code in `openai.rs`, `anthropic.rs`, `gemini.rs`, `grok.rs`, `ollama.rs` only handles request body shape + response chunk parsing — never the HTTP plumbing.

### Classified provider errors

`AppError::Provider` serializes to JSON as `{ error_type: "provider", kind, message }`, letting the frontend branch on the `kind` (Auth → reconfigure banner, RateLimit → backoff hint, Server → retry affordance, etc.) instead of string-matching on error messages.

---

## Task Assignments

Configure which provider handles each task in **Settings → Task Assignments**.
The model dropdown updates automatically based on the selected provider and is fetched live from that provider.

| Task | Requires Vision | Recommended Provider |
|------|----------------|---------------------|
| Grammar | No | OpenAI `gpt-4o-mini`, Anthropic `claude-3-5-haiku-20241022`, or Gemini `gemini-2.5-flash` |
| Translation | No | OpenAI `gpt-4o-mini`, Gemini `gemini-2.5-flash`, or Grok `grok-4-fast-non-reasoning` |
| Improve | No | OpenAI `gpt-4o`, Anthropic `claude-sonnet-4-20250514`, or Grok `grok-4` |
| Screen Question | **Yes** | OpenAI `gpt-4o`, Anthropic `claude-sonnet-4-20250514`, Gemini `gemini-2.5-flash`, Grok image-capable models, or a local vision model |

For **Screen Question**, ShortCut filters the dropdown to vision-capable models when the provider metadata exposes that capability. If a text-only local model is selected manually from older config, the backend returns a runtime error with instructions to reconfigure.

### Per-model vision flag

`TaskAssignment` carries an optional `supports_vision: Option<bool>` populated from each provider's live discovery endpoint (e.g. Ollama's `/api/show` `vision` capability, OpenAI's model listing). This overrides the provider-level default during dispatch:

| `supports_vision` | Effect |
|-------------------|--------|
| `Some(true)` | Vision allowed for this task even if the provider is generally text-only (e.g. Grok/Ollama with a multimodal model). |
| `Some(false)` | Vision rejected even if the provider is generally vision-capable (safety rail for a user who manually picked a text-only model). |
| `None` | Falls back to the provider-level `ProviderCapabilities::supports_vision`. |

`stream_screen_question` (`providers/mod.rs`) reads the flag before instantiating the provider and returns `AppError::Provider { kind: InvalidRequest, .. }` if the selected model cannot handle images.

---

## Prompt Templates

Grammar, translate, and improve tasks carry a **user-prompt template** with a `{text}` placeholder. Grammar, translate, improve, and screen-question each additionally carry a **system prompt** (role-setting message) rendered as a prepended `role: "system"` `ChatMessage` when non-empty. Translation target language is expressed entirely in the translate **system prompt** — there is no separate `target_language` setting.

**Placeholder validation**: `update_grammar_config`, `update_translate_config`, and `update_improve_config` reject user-prompts missing the `{text}` placeholder. **System prompts are never placeholder-validated** — they are sent verbatim. `update_screen_question_config` has no validation (system prompt only).

**Reset to defaults**: per-action settings page (`/actions/{grammar,translate,improve,screen-question}`) → Reset button. The button calls `get_default_{action}_config` and persists the returned default. Defaults are authored in Rust (`src-tauri/src/config/types/prompts.rs`) — single source of truth.

**Default grammar user-prompt:**
```
{text}
```

**Default translate user-prompt:**
```
{text}
```

## System-Role Handling per Provider

The two dispatch sites — `text_transform::transform_text` (grammar/translate/improve) and `screen_capture::send_screen_question` — each prepend a `ChatMessage { role: "system", content }` when the task's `system_prompt` is non-empty. Each provider translates that message into its native shape:

| Provider | Handling | Location |
|----------|----------|----------|
| **OpenAI** | `role: "system"` passed verbatim in the `messages` array (native support). | `providers/openai.rs` |
| **Anthropic** | `extract_system` pulls all `role: "system"` messages into the dedicated top-level `system` field on the request body; multiple system messages are joined with `\n`. | `providers/anthropic.rs::extract_system` |
| **Gemini** | System messages extracted into `systemInstruction.parts[0].text` (v1beta+); the `contents` array carries only user/assistant turns. | `providers/gemini.rs` |
| **Grok (xAI)** | Delegates to `OpenAiProvider` with a custom base URL — inherits OpenAI's system handling. | `providers/grok.rs` |
| **Ollama** | `role: "system"` passed verbatim in the `messages` array — Ollama's chat API supports it natively. | `providers/ollama.rs` |

No provider file needs editing when adding a new system-prompt consumer — only the dispatch site needs to construct the system-role `ChatMessage`.

---

## API Key Storage

API keys are stored in plain JSON at:
- Windows: `%APPDATA%\com.g-prompter.shortcut\config.json`
- macOS: `~/Library/Application Support/com.g-prompter.shortcut/config.json`
- Linux: `~/.local/share/com.g-prompter.shortcut/config.json`

For sensitive deployments, use OS-level disk encryption. A `tauri-plugin-stronghold`
integration can be added in a future release without breaking the config interface.
