# Screen Question Feature

## Overview

AI-powered screen analysis. User presses `Alt+S` (Windows/Linux) or `Cmd+Shift+S` (macOS), the current monitor is captured, and a floating chat overlay appears. The user types a question about the screenshot, and the AI responds via streaming. Supports multi-turn conversation about the same screenshot. Can also be triggered from the Action Wheel.

**Key properties**: Focusable overlay (user types in it), transparent background, always-on-top, screenshot captured BEFORE overlay appears, ephemeral conversation (cleared on close).

**Key architectural difference from indicator/action-menu**: The screen-question window NEEDS focus (user must type). No `WS_EX_NOACTIVATE`, no `apply_non_focusable()`. Uses standard `.show()` + `.set_focus()`.

**Two routes, one feature**:

| Route | Purpose |
|-------|---------|
| `/screen-question` | Overlay window (separate Tauri webview) — capture + chat UI |
| `/actions/screen-question` | In-app **settings page** — edit system prompt + pick vision-capable provider/model |

The overlay route path `/screen-question` is load-bearing (hardcoded in `screen_capture.rs` as the window URL). The settings page lives under the actions hub at `/actions/screen-question` and edits the same `TaskAssignment` that `/settings` edits — single source of truth via `providersSettingsState`.

## Architecture

```
┌─────────────────┐                    ┌──────────────────────┐
│   Main Window   │                    │ Screen Question      │
│                 │                    │ Window               │
│  ShortcutDisp.  │                    │                      │
│  dispatches     │                    │  ┌────────────────┐  │
│  screen_question│                    │  │  OverlayChat   │  │
│                 │                    │  │  (reusable)    │  │
│  invoke(        │───invoke──────────►│  │  - messages    │  │
│  "screen_       │  screen_question   │  │  - input       │  │
│   question")    │                    │  │  - thumbnail   │  │
│                 │                    │  └────────────────┘  │
└─────────────────┘                    └──────────────────────┘
        │                                      │
        │                                      │ invoke("send_screen_question")
        ▼                                      ▼
┌──────────────────────────────────────────────────────┐
│   Rust Backend                                       │
│                                                      │
│  screen_capture.rs        providers/mod.rs           │
│  - capture_screen_as_base64 stream_screen_question() │
│  - show_screen_question   - reads task_assignments   │
│  - hide_screen_question   - validates vision support │
│                           - calls provider.stream()  │
└──────────────────┬───────────────────────────────────┘
                   │
                   │ provider.stream() — SSE or streamed HTTP
                   ▼
┌──────────────────────────────────────────────────────┐
│  Vision Provider (configured in Settings)            │
│  OpenAI / Anthropic / Gemini / xAI / Local model     │
│  (the Settings dropdown only shows current           │
│   vision-capable models for Screen Question)         │
└──────────────────────────────────────────────────────┘
```

## Interaction Flow

```
User presses Alt+S (or selects from Action Wheel)
  -> Rust: global shortcut handler emits "shortcut-triggered: screen_question"
  -> Frontend: ShortcutDispatcher.dispatch("screen_question")
  -> invoke("screen_question")
  -> Rust: screen_capture::screen_question()
       1. Toggle check: if overlay already visible, hide and return
       2. Capture screenshot (xcap -> resize -> JPEG -> base64)
       3. Show overlay window (centered on monitor, WITH focus)
       4. Emit "screen-captured" event with { image_base64, image_mime_type }

Frontend: +page.svelte receives "screen-captured" event
  -> Sets context (imageBase64, imageMimeType)
  -> Renders OverlayChat with screenshot context

User types question, presses Enter
  -> OverlayChatInput -> overlay-chat-controller.ts
  -> invoke("send_screen_question", { image_base64, image_mime_type, messages })
  -> Rust: screen_capture::send_screen_question()
       -> Reads config.screen_question.system_prompt
       -> Prepends { role: "system", content: system_prompt } to `messages` when non-empty
       -> Calls providers::stream_screen_question(app, img, full_messages)
  -> Rust: providers::stream_screen_question()
       -> Reads task_assignments.screen_question.{provider_id, model} from config
       -> Validates provider supports vision (returns error if not)
       -> Calls provider.stream() with image attachment
       -> Emits "screen-answer-chunk" { content } per chunk
       -> Emits "screen-answer-complete" when done
       -> Emits "screen-answer-error" { error } on failure
  -> Frontend: listens for chunks, appends to streaming message

User presses Escape or clicks close button
  -> invoke("hide_screen_question")
  -> Rust: hides overlay window
  -> Frontend: clears conversation state
```

## Reusable OverlayChat System

The chat UI is a **generic, reusable component system** designed for future features (Quick Ask, Contextual Help, etc.). Screen Question is the first consumer.

### How It Works

OverlayChat is driven by two configuration objects:

- **`ChatContext`**: What the AI is analyzing (screenshot, text, or none)
- **`OverlayChatConfig`**: Event names and command names for the specific use case

To add a new OverlayChat consumer:
1. Create a new route (e.g., `/quick-ask`)
2. Listen for a context event
3. Render `<OverlayChat>` with a different `ChatContext` and `OverlayChatConfig`
4. Implement the corresponding Rust command and streaming events

### Type Definitions

```typescript
interface ChatContext {
  type: 'screenshot' | 'text' | 'none';
  imageBase64?: string;
  imageMimeType?: string;
  selectedText?: string;
}

interface OverlayChatConfig {
  placeholder: string;       // Input placeholder text
  chunkEvent: string;        // e.g., "screen-answer-chunk"
  completeEvent: string;     // e.g., "screen-answer-complete"
  errorEvent: string;        // e.g., "screen-answer-error"
  sendCommand: string;       // e.g., "send_screen_question"
}
```

### Component Props

```svelte
<OverlayChat
  context={chatContext}
  config={chatConfig}
  onClose={handleClose}
/>
```

## File Structure

```
src/lib/
├── features/overlay-chat/            # Generic feature module
│   ├── index.ts                      # Public exports
│   ├── types.ts                      # ChatMessage, ChatContext, OverlayChatConfig
│   ├── overlay-chat-controller.ts    # sendMessage, initListeners, resetChat
│   └── constants.ts                  # MAX_VISIBLE_MESSAGES, AUTO_SCROLL_THRESHOLD
│
├── components/overlay-chat/          # Reusable UI components
│   ├── index.ts                      # Component exports
│   ├── OverlayChat.svelte            # Chat container (messages + input + thumbnail)
│   ├── OverlayChatMessage.svelte     # Single message bubble (user/assistant)
│   └── OverlayChatInput.svelte       # Textarea + send button
│
└── state/
    ├── overlay-chat.svelte.ts        # Reactive state (messages[], isStreaming, error)
    └── screen-question-config.svelte.ts # Screen-question system_prompt state (for /actions/screen-question)

src/routes/
├── screen-question/
│   ├── +page.svelte                  # Overlay page: listens for screenshot, renders OverlayChat
│   └── +layout.ts                    # SSR disabled
└── actions/screen-question/
    └── +page.svelte                  # In-app settings page: system prompt + provider/model

src-tauri/src/
├── screen_capture.rs                 # Capture + resize + base64 + window management + commands (reads screen_question.system_prompt)
├── config/types/prompts.rs           # ScreenQuestionConfig + default_screen_question_system_prompt()
└── providers/mod.rs                  # stream_screen_question() — vision provider dispatch
```

## Window Configuration

From `tauri.conf.json`:

```json
{
  "label": "screen-question",
  "title": "ShortCut Screen Question",
  "url": "/screen-question",
  "width": 520,
  "height": 480,
  "resizable": false,
  "decorations": false,
  "transparent": true,
  "shadow": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "visible": false,
  "center": false,
  "focus": true
}
```

**Note**: `"focus": true` -- unlike indicator and action-menu which use `"focus": false`. This window needs keyboard input.

## Image Processing Pipeline

In `screen_capture.rs`:

1. **Capture**: `xcap::Monitor::capture_image()` captures the current monitor (matched by cursor position)
2. **Resize**: If either dimension exceeds 2048px, scale down maintaining aspect ratio (Lanczos3 filter)
3. **Convert**: RGBA to RGB (JPEG has no alpha channel)
4. **Encode**: JPEG at quality 85
5. **Base64**: Standard base64 encoding for transport

Constants: `MAX_IMAGE_DIMENSION = 2048`, `JPEG_QUALITY = 85`

## Vision Provider Requirements

Screen Question requires a **vision-capable provider** assigned in Settings → Task Assignments → Screen Question.

**Vision-capable providers:**

| Provider | Model examples |
|----------|---------------|
| OpenAI | `gpt-4o`, `gpt-4o-mini`, `gpt-4.1` |
| Anthropic | `claude-sonnet-4-20250514`, `claude-3-5-haiku-20241022` |
| Gemini | `gemini-2.5-flash`, `gemini-2.5-pro` |
| xAI | `grok-4.20-beta-latest-non-reasoning` and other image-capable Grok models returned by `/v1/language-models` |
| Local | `gemma3`, `llava`, or another installed model with the `vision` capability |

The Settings dropdown is populated live from provider metadata and filters Screen Question to vision-capable models when that metadata is available. If a non-vision model is still assigned from older config, `send_screen_question` returns an `AppError::Provider { kind: InvalidRequest, .. }` with instructions to reconfigure.

**Per-model vision gate**: `TaskAssignment.supports_vision: Option<bool>` records the per-model capability observed at discovery time. `stream_screen_question` consults this before falling back to the provider-level capability — see [PROVIDERS.md](../PROVIDERS.md#per-model-vision-flag).

**Internal request shape (all providers):**
```rust
ChatRequest {
    model: "gpt-4o",         // from task_assignments.screen_question.model
    messages: [...],          // full conversation history
    image: Some(ImageAttachment {
        base64: "...",         // JPEG base64
        mime_type: "image/jpeg",
    }),
    max_tokens: None,
    temperature: Some(0.7),
}
```

**Timeout**: Provider-specific (typically 30-120s; vision models can be slow).

## Conversation Model

- **Client-side state**: Messages stored in `overlay-chat.svelte.ts` as `ChatMessage[]`
- **Full history sent each request**: Every `send_screen_question` call includes the full conversation history
- **Ephemeral**: Conversation cleared when overlay is closed (not persisted)
- **Streaming**: Empty assistant message created with `isStreaming: true`, chunks appended, then marked complete

## Event Communication

| Event | Direction | Payload | Purpose |
|-------|-----------|---------|---------|
| `screen-captured` | Rust -> Screen Question window | `{ image_base64, image_mime_type }` | Screenshot data after capture |
| `screen-answer-chunk` | Rust -> Screen Question window | `{ content: String }` | Streaming text chunk |
| `screen-answer-complete` | Rust -> Screen Question window | `()` | Stream finished |
| `screen-answer-error` | Rust -> Screen Question window | `{ error: String }` | Stream error |

## Tauri Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `screen_question` | - | - | Capture screen + show overlay (toggle) |
| `send_screen_question` | `image_base64, image_mime_type, messages` | - | Prepend `screen_question.system_prompt` (when non-empty) then stream via configured vision provider |
| `hide_screen_question` | - | - | Hide the overlay window |
| `get_default_screen_question_config` | - | `ScreenQuestionConfig` | Default system prompt (for Reset on settings page) |
| `update_screen_question_config` | `screenQuestion: ScreenQuestionConfig` | - | Persist edited system prompt |

## System Prompt Configuration

The screen-question system prompt lives at `config.screen_question.system_prompt` and is authored via `default_screen_question_system_prompt()` in `src-tauri/src/config/types/prompts.rs`. It is prepended as a `role: "system"` `ChatMessage` before the user's conversation history in `screen_capture::send_screen_question` — when empty, no system message is emitted (backward compat).

**Editing surface**: `/actions/screen-question` (settings page). Debounced textarea + Reset button; provider/model picker reuses `TaskAssignmentRow` against the same `TaskAssignment` row `/settings` edits.

**Default system prompt** (authored in Rust):

```
You are a screen-grounded assistant. A screenshot of the user's screen is provided.
Answer the user's question using only what is visible in the screenshot. If the
answer is not visible, say so plainly. Keep answers short, factual, and direct —
no preamble, no speculation, no commentary.
```

**Per-provider routing of the system message**: see [PROVIDERS.md](../PROVIDERS.md#system-role-handling-per-provider).

## Dismiss Behaviors

| Trigger | What Happens |
|---------|-------------|
| Press `Alt+S` again | Toggle: hides the overlay |
| Press `Escape` | `onClose` callback hides overlay, clears conversation |
| Click close button (X) | Same as Escape |
| Direct shortcut (e.g., `Alt+G`) | `ShortcutDispatcher` hides screen question before dispatching |

### Stream cancellation on close

A shared `Arc<AtomicBool>` cancellation flag is held in app state. `hide_screen_question` flips it to `true`; the `read_sse` / `read_ndjson` helpers in `providers/http.rs` check it on every event boundary and abort the in-flight provider stream without waiting for a timeout. The flag is reset when a new `send_screen_question` starts. This avoids wasting tokens + bandwidth after the user dismisses the overlay mid-answer.

## Display Resilience

Same pattern as indicator and action-menu:
1. **Health check on show**: `ensure_screen_question_window()` validates handle before every show
2. **System event response**: `handle_display_change_screen_question()` called on `RunEvent::Resumed` and `ScaleFactorChanged`
3. **Window recreation**: Destroys stale window, waits for cleanup, rebuilds via `WebviewWindowBuilder`

## Known Limitations

1. **Screenshot is static**: The screenshot is captured once when the shortcut is pressed. If the screen changes, the AI still sees the original capture. User must re-trigger to get a fresh screenshot.

2. **No region selection**: Captures the entire monitor where the cursor is located. No crop or region selection UI.

3. **Image size**: Large monitors produce large base64 strings even after resize. The 2048px cap and JPEG compression mitigate this, but payloads can still be significant.

4. **macOS permissions**: Screen capture via `xcap` requires Screen Recording permission on macOS. When the permission is denied, `xcap` silently returns an all-black image instead of raising an error. `screen_capture.rs` runs a pixel heuristic on the captured frame and, if it looks TCC-blocked, returns `screen_question.permission_denied_macos` so the frontend can show an actionable banner directing the user to **System Settings → Privacy & Security → Screen Recording**. Note: this is a heuristic (not a Core Graphics preflight) — documented trade-off to avoid pulling in `core-graphics` as a build-graph dependency.

5. **Vision model latency**: The first response chunk may take several seconds depending on the vision model and provider. Vision models are inherently slower than text-only models.

## Future Extensions

The OverlayChat system is designed for reuse:

- **Quick Ask**: No screenshot context, just a floating chat overlay for general questions
- **Contextual Help**: Selected text as context instead of a screenshot
- **Multi-modal**: Could support additional context types (clipboard images, files)

Each extension needs: a new route, a new Rust command for its specific API call, and a new `OverlayChatConfig` with appropriate event names.
