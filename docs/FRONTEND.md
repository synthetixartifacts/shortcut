# Frontend Architecture

## Directory Structure

```
src/
├── routes/                       # SvelteKit pages
│   ├── +layout.svelte            # App shell with sidebar, theme, locale loading
│   ├── +page.svelte              # Dashboard (provider readiness cards)
│   ├── actions/                  # Actions hub
│   │   ├── +page.svelte          # Actions list (5 linked cards)
│   │   ├── dictation/+page.svelte # Dictation settings (engine, mic, terms)
│   │   ├── grammar/+page.svelte   # Grammar settings (system + user prompt + provider/model)
│   │   ├── translate/+page.svelte # Translate settings (system + user prompt + provider/model)
│   │   ├── improve/+page.svelte   # Improve settings (system + user prompt + provider/model)
│   │   └── screen-question/+page.svelte # Screen-question settings (system prompt + vision-filtered model)
│   ├── shortcuts/+page.svelte    # Shortcuts management
│   ├── settings/+page.svelte     # Settings (provider keys, task assignments)
│   ├── app-settings/+page.svelte # App Settings (theme, language, debug)
│   ├── history/+page.svelte      # History page
│   ├── debug/+page.svelte        # Debug log viewer + per-provider diagnostics
│   ├── indicator/+page.svelte    # Activity indicator window
│   ├── action-menu/+page.svelte  # Action wheel radial menu window
│   ├── screen-question/+page.svelte # Screen question overlay window
│   └── onboarding/+page.svelte  # First-run provider + engine setup
│
└── lib/
    ├── api/                      # Tauri IPC wrappers
    │   ├── index.ts
    │   └── tauri.ts              # Type-safe invoke() calls
    │
    ├── components/               # UI Components
    │   ├── debug/                # Debug log components
    │   │   ├── LogFilter.svelte
    │   │   ├── LogViewer.svelte
    │   │   └── index.ts
    │   ├── dictation/            # Dictation settings components
    │   │   ├── AudioSettingsPanel.svelte
    │   │   ├── MicrophoneSelector.svelte
    │   │   ├── MicrophoneTest.svelte
    │   │   ├── TermsList.svelte
    │   │   ├── TranslationConfig.svelte
    │   │   ├── EngineCard.svelte        # Individual transcription engine card
    │   │   ├── EngineSelector.svelte    # Engine selection container
    │   │   ├── ModelDownload.svelte     # Model download progress UI
    │   │   └── index.ts
    │   ├── history/              # History components
    │   │   ├── EmptyHistory.svelte
    │   │   ├── HistoryItem.svelte
    │   │   ├── HistoryList.svelte
    │   │   ├── Pagination.svelte
    │   │   ├── RecentEntries.svelte
    │   │   └── index.ts
    │   ├── onboarding/           # First-run flow (PHASE 3B split of onboarding/+page.svelte)
    │   │   ├── OnboardingStepLlm.svelte       # LLM provider selection step
    │   │   ├── OnboardingStepStt.svelte       # STT engine selection step
    │   │   ├── OnboardingStepComplete.svelte  # Confirmation step
    │   │   ├── OnboardingStepper.svelte       # Step indicator chrome
    │   │   └── index.ts
    │   ├── ui/
    │   │   ├── primitives/       # Base: Button, Kbd, Icon, Input, Select
    │   │   ├── patterns/         # Composite: Card, FormField, PageHeader, ErrorBanner, SaveIndicator, SettingsLinkCard
    │   │   ├── ActionCard.svelte
    │   │   ├── Modal.svelte
    │   │   ├── ShortcutEditorModal.svelte
    │   │   ├── ShortcutRecorder.svelte
    │   │   ├── ShortcutsList.svelte
    │   │   ├── RecordingArea.svelte
    │   │   ├── ValidationMessage.svelte
    │   │   └── index.ts
    │   ├── layout/               # App shell components
    │   │   ├── Sidebar.svelte
    │   │   ├── NavItem.svelte
    │   │   └── index.ts
    │   ├── settings/             # Settings page components
    │   │   ├── ApiKeyInput.svelte
    │   │   ├── TextInput.svelte
    │   │   ├── LanguageSelector.svelte
    │   │   ├── SettingsSection.svelte
    │   │   ├── SetupBanner.svelte
    │   │   ├── ProviderCredentialsForm.svelte   # Per-provider API key / URL inputs
    │   │   ├── TaskAssignmentsForm.svelte       # Provider+model picker per task (matrix view)
    │   │   ├── TaskAssignmentRow.svelte         # Single-row provider/model picker (reused by per-action pages)
    │   │   └── index.ts
    │   ├── actions/              # Per-action settings shared components
    │   │   └── PromptEditor.svelte # Textarea + Reset button (debounced, controlled input)
    │   ├── indicator/            # Activity indicator components
    │   │   ├── IndicatorWindow.svelte
    │   │   ├── IndicatorDots.svelte
    │   │   └── index.ts
    │   ├── action-menu/          # Action wheel components
    │   │   ├── PieMenu.svelte
    │   │   ├── PieWedge.svelte
    │   │   └── index.ts
    │   └── overlay-chat/          # Reusable overlay chat (used by screen-question)
    │       ├── OverlayChat.svelte
    │       ├── OverlayChatMessage.svelte
    │       ├── OverlayChatInput.svelte
    │       └── index.ts
    │
    ├── features/                 # Feature modules (business logic)
    │   ├── dictation/            # Voice recording (audio sent to active STT engine)
    │   │   ├── audio-helpers.ts       # Audio utility functions
    │   │   ├── audio-recorder.ts      # MediaRecorder wrapper
    │   │   ├── audio-startup.ts       # Recording startup logic
    │   │   ├── constants.ts
    │   │   ├── device-validation.ts   # Device validation
    │   │   ├── dictation-controller.ts # Recording lifecycle
    │   │   ├── microphone-test.ts
    │   │   ├── wav-encoder.ts         # PCM → WAV header encoding (PHASE 3A extract)
    │   │   ├── transcription-events.ts # Retry + diagnostic Tauri event listeners (PHASE 3A extract)
    │   │   ├── model-download.svelte.ts # Shared model-download controller (ModelDownload + EngineCard consumers)
    │   │   ├── types.ts
    │   │   └── index.ts
    │   ├── grammar/              # Grammar fix (uses text-transform)
    │   │   ├── grammar-controller.ts
    │   │   └── index.ts
    │   ├── translation/          # Translation (uses text-transform)
    │   │   ├── translation-controller.ts
    │   │   └── index.ts
    │   ├── improve/              # Improve (uses text-transform)
    │   │   ├── improve-controller.ts
    │   │   └── index.ts
    │   ├── indicator/            # Activity indicator system
    │   │   ├── constants.ts
    │   │   ├── helpers.ts
    │   │   ├── types.ts
    │   │   └── index.ts
    │   ├── action-menu/          # Action wheel radial menu
    │   │   ├── types.ts
    │   │   ├── constants.ts
    │   │   ├── menu-controller.ts
    │   │   └── index.ts
    │   ├── overlay-chat/          # Reusable overlay chat system
    │   │   ├── types.ts            # ChatMessage, ChatContext, OverlayChatConfig
    │   │   ├── overlay-chat-controller.ts  # sendMessage, initListeners, resetChat
    │   │   ├── constants.ts        # Sizing, scroll thresholds
    │   │   └── index.ts
    │   ├── shortcuts/            # Shortcut handling
    │   │   ├── constants.ts
    │   │   ├── shortcut-dispatcher.ts
    │   │   ├── shortcut-display.ts
    │   │   ├── shortcut-helpers.ts
    │   │   ├── shortcut-parser.ts
    │   │   ├── shortcut-recorder.svelte.ts
    │   │   ├── shortcut-validation.ts
    │   │   ├── types.ts
    │   │   └── index.ts
    │   ├── text-transform/       # Base controller factory
    │   │   ├── base-controller.ts
    │   │   └── index.ts
    │   └── index.ts
    │
    ├── i18n/                      # Internationalization (custom, no deps)
    │   ├── index.ts              # t() function, registerLocale()
    │   └── locales/              # Translation JSON files
    │       ├── en.json           # English (loaded eagerly)
    │       ├── fr.json           # French (lazy-loaded)
    │       └── es.json           # Spanish (lazy-loaded)
    │
    ├── services/                 # Service integrations
    │   ├── microphone-permission.ts # Permission checking
    │   ├── microphone.ts         # Microphone enumeration
    │   └── index.ts
    │
    ├── state/                    # Global reactive state (Svelte 5 runes)
    │   ├── app.svelte.ts         # App-wide state (recording, transcription)
    │   ├── activity.svelte.ts    # Activity indicator state
    │   ├── app-settings.svelte.ts # App preferences (theme, language, debug)
    │   ├── providers.svelte.ts   # Per-provider readiness (replaces auth.svelte.ts)
    │   ├── providers-settings.svelte.ts # Settings-page orchestration (per-field auto-save, saveStatus record)
    │   ├── providers-settings-sync.ts # Pure helpers: model-option merging, Custom sentinel (Local), vision filtering for screen_question
    │   ├── providers-settings-tasks.ts # Pure helpers: per-task assignment mutations (free-text custom models, vision flag)
    │   ├── onboarding.svelte.ts  # First-run flow step + selection state
    │   ├── debug.svelte.ts       # Debug log state
    │   ├── dictation-config.svelte.ts # Dictation settings state
    │   ├── history.svelte.ts     # History state
    │   ├── settings.svelte.ts    # API/config state
    │   ├── shortcuts.svelte.ts   # Shortcut bindings
    │   ├── improve-config.svelte.ts # Improve: prompt + system_prompt + reset helpers
    │   ├── grammar-config.svelte.ts # Grammar: prompt + system_prompt + reset helpers
    │   ├── translate-config.svelte.ts # Translate: prompt + system_prompt + reset helpers
    │   ├── screen-question-config.svelte.ts # Screen-question: system_prompt only + reset helper
    │   ├── overlay-chat.svelte.ts # Overlay chat state (messages, streaming)
    │   └── engine.svelte.ts      # Engine selection, capabilities, model status
    │
    ├── styles/                   # Global CSS
    │   └── app.css               # Design tokens (primitive→semantic→component + dark theme)
    │
    ├── types/                    # TypeScript definitions
    │   └── index.ts
    │
    └── utils/                    # Helpers
        ├── async-state.ts        # withAsyncState() wrapper (+ onSaving/onSaved/onError hooks)
        ├── save-status.svelte.ts # createSaveStatus() + SaveStatus type + SAVE_DEBOUNCE_MS
        ├── error.ts              # Error extraction
        ├── format.ts             # Formatting utilities
        ├── logger.ts             # Dual console + Rust backend logging
        └── index.ts
```

## Design Principles

1. **Single Responsibility**: Each file has ONE clear purpose (target <200 lines)
2. **Feature-Based Organization**: Features are self-contained modules in `features/`
3. **Separation of Concerns**: UI components vs business logic vs state
4. **Dependency Inversion**: Components depend on controllers and state, not implementations
5. **Open/Closed**: Add new shortcuts via registry without modifying existing code

## Svelte 5 Patterns

### State Management with Runes

Use `.svelte.ts` files with `$state` runes instead of stores:

```typescript
// state/app.svelte.ts
export const appState = $state({
  status: 'Ready' as AppStatus,
  isRecording: false,
  lastTranscription: '',
  transcriptionBuffer: '',
  selectedMicrophoneId: null as string | null
});
```

### State Files Overview

| File | Purpose |
|------|---------|
| `app.svelte.ts` | Recording state, transcription buffer, microphone |
| `activity.svelte.ts` | Activity indicator visibility and state |
| `app-settings.svelte.ts` | App preferences (theme, language, debug toggle) |
| `providers.svelte.ts` | Per-provider readiness (has API key; for Local: `local_ready` reads `creds.local.base_url.trim().length > 0`, not a key) |
| `providers-settings.svelte.ts` | Settings page orchestration: in-memory `config` mirror, `saveStatus` record (8 credential keys for Local's 3 fields + 4 task keys), `saveProviderCredential` / `saveTaskAssignment` helpers, typed discovery-error slot |
| `providers-settings-sync.ts` | Pure helpers extracted from `providers-settings.svelte.ts`: `syncTaskAssignmentsForProvider`, `syncTaskAssignmentModel`, `getTaskModelOptions` (Custom sentinel merge for Local, vision filter for `screen_question`) |
| `providers-settings-tasks.ts` | Pure helpers for task-assignment mutations: `handleTaskProviderChange`, `handleTaskModelChange` (Custom-sentinel → free-text round-trip), `handleTaskVisionChange` |
| `onboarding.svelte.ts` | First-run flow step + provider/engine selection |
| `debug.svelte.ts` | Debug log entries and filtering |
| `dictation-config.svelte.ts` | Dictation settings (audio, languages, terms) |
| `history.svelte.ts` | History entries, pagination, CRUD operations |
| `settings.svelte.ts` | API/config settings from backend |
| `shortcuts.svelte.ts` | Registered keyboard shortcuts |
| `improve-config.svelte.ts` | Improve: `prompt` + `system_prompt` + `saveImprovePrompt`/`saveImproveSystemPrompt`/`resetImprovePrompt`/`resetImproveSystemPrompt` |
| `grammar-config.svelte.ts` | Grammar: `prompt` + `system_prompt` + save/reset helpers (mirrors improve) |
| `translate-config.svelte.ts` | Translate: `prompt` + `system_prompt` + save/reset helpers |
| `screen-question-config.svelte.ts` | Screen-question: `system_prompt` only + `saveScreenQuestionSystemPrompt` / `resetScreenQuestionSystemPrompt` |
| `overlay-chat.svelte.ts` | Overlay chat messages, streaming status, error state |
| `engine.svelte.ts` | Engine selection, capabilities, model status, slowness tracking |

Action wheel visibility/hovered-item state lives inside `features/action-menu/` (no dedicated `state/action-menu.svelte.ts` exists).

### Component Props

Use `$props()` for type-safe props:

```svelte
<script lang="ts">
  interface Props {
    label: string;
    disabled?: boolean;
    onclick?: () => void;
  }

  let { label, disabled = false, onclick }: Props = $props();
</script>
```

### Reactive Derivations

Use `$derived` for computed values:

```typescript
export const isReady = $derived(
  appState.status === 'idle' && !appState.isRecording
);
```

## Feature Modules

Each feature in `features/` follows this pattern:

```
features/
└── grammar/
    ├── grammar-controller.ts   # Business logic
    └── index.ts                # Public exports
```

### Controller Pattern

Controllers use the text transform factory for consistent behavior:

```typescript
// features/grammar/grammar-controller.ts
import { createTextTransformHandler } from '$lib/features/text-transform/base-controller';
import { transformText } from '$lib/api/tauri';

export const handleGrammarFix = createTextTransformHandler({
  name: 'Grammar',
  statusMessage: 'Fixing grammar...',
  successMessage: 'Grammar fixed!',
  transform: (text) => transformText('grammar', text),
  activityType: 'grammar',
});
```

The `transformText(task, text)` command dispatches to the task-assigned provider in Rust backend.

### Text Transform Factory

The `text-transform/base-controller.ts` provides a factory for all transform features:

```typescript
export function createTextTransformHandler(options: {
  name: string;
  statusMessage: string;
  successMessage: string;
  transform: (text: string) => Promise<string>;
  activityType?: ActivityType;
}) {
  return async function handleTransform(): Promise<void> {
    const selection = await getSelectionWithFormat();
    if (!selection.text?.trim()) return;
    const result = await options.transform(selection.text);
    await pasteFormatted(result, selection.format);
  };
}
```

All transform features (grammar, translate, improve) use this factory pattern.

## Settings → AI Providers

The providers page (`/settings/providers`) is composed of two stacked sections:

1. **`ProviderCredentialsForm`** — one collapsible row per provider (OpenAI, Anthropic, Gemini, Grok, Soniox, Local). Each row renders the credential inputs plus an inline `discoveryError` banner when the last model-list probe failed (D9 — loud errors scoped to this surface only).

2. **`TaskAssignmentsForm`** — matrix of `TaskAssignmentRow` (one per task: grammar, translate, improve, screen_question). Each row has a provider select, a model select, and a **Supports vision** checkbox (screen-question row only renders vision; the other rows hide it).

### The Local provider row

When provider = Local, `ProviderCredentialsForm` renders:

- **Base URL** (`TextInput`) — persists to `creds.local.base_url` on debounced input via `saveStatus['local.baseUrl']`. Editing the URL clears `detected_protocol` so the next save re-runs detection. The frontend's `normalizeLocalChatUrl` only trims whitespace and trailing slashes — suffix stripping happens in Rust (`normalize_local_base_url`), so the user's pasted value is preserved verbatim in config.
- **Protocol** (`Select` with `auto` / `ollama` / `openai_compatible` options) — saves immediately on change to `creds.local.protocol` via `saveStatus['local.protocol']`. Changing the protocol also clears `detected_protocol`.
- **Detected: X** badge — shown only when `protocol === 'auto'` and `detected_protocol` is set.
- **Re-detect** button (`Button variant="ghost"`) — rendered next to the Detected row whenever `protocol === 'auto'` (independent of whether the badge is visible). Handler `handleRedetect` in `ProviderCredentialsForm.svelte` delegates to `redetectLocalProtocol()` in `providers-settings.svelte.ts`, which clears `detected_protocol`, persists via `updateProvidersConfig`, then calls `refreshProviderModels('local')` to re-run discovery. Uses `saveStatus['local.baseUrl']` for save feedback. i18n key: `settings.local_redetect` (en/fr/es).
- **API key** (`ApiKeyInput`) — rendered proactively whenever `protocol === 'openai_compatible' || protocol === 'auto'` (`showApiKey` derivation). Not gated on `detected_protocol` — letting the user volunteer a key up front avoids the dead-end where detection needs the key to succeed but the field is hidden until detection succeeds. Persists to `creds.local.api_key` via `saveStatus['local.apiKey']`.

The `LocalCredentials` interface in `src/lib/types/index.ts` mirrors the Rust `LocalCredentials` struct exactly (base_url, protocol literal union, detected_protocol nullable, api_key nullable).

### TaskAssignmentRow — "Custom…" sentinel + per-row vision

`TaskAssignmentRow` is reused by both the Settings matrix and the per-action pages. For every task, the model dropdown includes a `"Custom…"` sentinel option; selecting it reveals a free-text input so the user can type a model id that isn't in the discovered list. The typed id round-trips through config verbatim. A non-blocking inline warning appears when the typed id isn't in `modelOptions`.

The per-row **Supports vision** checkbox:
- For Local custom ids, the user explicitly opts in (D5). Backend reads `TaskAssignment.supports_vision` as the source of truth for vision dispatch.
- For discovered ids that advertise vision capability (Ollama `/api/show`, OpenAI-compat filter), the checkbox is pre-populated from discovery metadata but remains user-overridable.
- For tasks that don't need vision (grammar, translate, improve), the checkbox is hidden.

**Local Custom-sentinel / empty-model nuance** (`provider-catalog.ts`): `DEFAULT_TASK_MODELS.local.*` is all empty strings — Local has no hardcoded default model. `normalizeTaskAssignment` preserves an empty model verbatim for Local (vs. cloud providers where empty is replaced by the documented default). Empty model = Custom mode with no typed id yet. When the user switches a task to Local via `handleTaskProviderChange` (`providers-settings-tasks.ts`) and `providersSettingsState.models.local` is already populated, the handler auto-picks the first discovered model (id + `supports_vision` flag) so the assignment is immediately usable. If discovery hasn't run yet, the assignment stays empty and the UI renders the Custom free-text flow while discovery kicks off in the background.

## Per-Action Settings Pages

Each of the four AI actions has a dedicated settings page at `/actions/{grammar,translate,improve,screen-question}`. All four pages follow the same recipe:

1. **State module** (`lib/state/{action}-config.svelte.ts`) — exposes the config `$state` object plus `load*`, `save*`, and `reset*` helpers that wrap the `update_{action}_config` / `get_default_{action}_config` Tauri commands.
2. **Page** (`routes/actions/{action}/+page.svelte`) — renders on mount: parallel `Promise.all` of the action's `loadConfig`, `loadProvidersSettings`, and `getDefault*Config` fetches.
3. **Prompt editing** — uses the shared `$lib/components/actions/PromptEditor.svelte` (debounced 500ms save, Reset button hidden when `value === defaultValue`). The page owns the state; `PromptEditor` is a controlled input.
4. **Provider/model picker** — reuses the existing `TaskAssignmentRow` from `$lib/components/settings/` with the action's `taskKey`. Changes call `handleTaskProviderChange` / `handleTaskModelChange`, which each auto-save via `saveTaskAssignment` inside `providers-settings.svelte.ts` — the SAME module `/settings` uses, so task-assignment edits from either surface converge on a single source of truth. The row receives `saveStatus={providersSettingsState.saveStatus['task.<key>']}` so feedback renders next to the row label.

**Page shape (simplified):**

```svelte
<SettingsSection title="System prompt">
  <PromptEditor
    value={grammarConfigState.system_prompt}
    defaultValue={defaultSystemPrompt}
    onInput={onSystemPromptInput}
    onReset={onSystemPromptReset}
    ...
  />
</SettingsSection>

<SettingsSection title="User prompt">
  <PromptEditor ... />
</SettingsSection>

<SettingsSection title="Provider / model">
  <TaskAssignmentRow
    taskKey="grammar"
    providerId={assignment.provider_id}
    model={assignment.model}
    {providerOptions}
    {modelOptions}
    {onProviderChange}
    {onModelChange}
  />
</SettingsSection>
```

Screen-question omits the user-prompt section (no template — the user's typed question IS the user message at dispatch time). Improve, grammar, and translate all include both sections.

See [EXTENDING.md](./EXTENDING.md#adding-a-per-action-settings-page) for the full add-a-new-action recipe.

## Shortcuts System

Located in `features/shortcuts/`:

| File | Purpose |
|------|---------|
| `types.ts` | Type definitions |
| `shortcut-parser.ts` | Parse keyboard shortcuts |
| `shortcut-display.ts` | Format shortcuts for display |
| `shortcut-validation.ts` | Validate shortcut combinations |
| `shortcut-recorder.svelte.ts` | Recording state (runes) |
| `shortcut-dispatcher.ts` | Route shortcuts to handlers |

### Adding a New Shortcut Handler

Register in `shortcut-dispatcher.ts`:

```typescript
const shortcutHandlers: Record<ShortcutAction, ShortcutHandler> = {
  dictation_start: async () => controller.startRecording(),
  dictation_stop: async () => controller.stopRecording(),
  dictation: async () => controller.toggleRecording(),
  grammar: handleGrammarFix,
  translate: handleTranslation,
  improve: handleImproveText,
  open_menu: async () => toggleActionMenu(),
  screen_question: async () => screenQuestion(),
  // Add new handlers here
};
```

## Activity Indicator System

The indicator feature provides visual feedback during operations:

### Files

```
features/indicator/
├── index.ts              # Exports
├── types.ts              # ActivityType, ActivityState, ActivityInfo
├── constants.ts          # Colors, animation timing
└── helpers.ts            # withIndicator(), startIndicator(), etc.

state/
└── activity.svelte.ts    # startActivity, updateActivity, endActivity

components/indicator/
├── IndicatorWindow.svelte
└── IndicatorDots.svelte
```

### Usage Pattern

```typescript
import { withIndicator } from '$lib/features/indicator';

// Simple pattern - handles show/hide automatically
await withIndicator('grammar', async () => {
  const text = await getSelectedText();
  const fixed = await fixGrammar(text);
  await pasteText(fixed);
}, { successMessage: 'Fixed!' });
```

### Activity Types

| Type | Color | Label |
|------|-------|-------|
| `dictation` | Red (#ef4444) | Recording |
| `grammar` | Blue (#3b82f6) | Fixing |
| `translate` | Purple (#8b5cf6) | Translating |
| `improve` | Teal (#10b981) | Improving |
| `processing` | Gray (#6b7280) | Processing |

## Action Wheel System

The action wheel provides a radial pie menu for visual action selection. Unlike the indicator (one-way events from main to overlay), the action wheel uses **two-way event communication**:

- **Main -> Menu**: `action-menu-show` (resets auto-dismiss timer)
- **Menu -> Main**: `menu-action-selected` (user picked an action)

The `ShortcutDispatcher` listens for `menu-action-selected` events and dispatches to the same handlers used by keyboard shortcuts.

### Files

```
features/action-menu/
├── index.ts              # Exports
├── types.ts              # MenuItem, MenuVisibility
├── constants.ts          # MENU_ITEMS, MENU_SIZE, AUTO_DISMISS_MS
└── menu-controller.ts    # selectAction() — emit event + hide window (visibility + hoveredItem state lives here)

components/action-menu/
├── PieMenu.svelte        # SVG container (renders wedges)
└── PieWedge.svelte       # Individual SVG arc wedge
```

See [features/ACTION_WHEEL.md](./features/ACTION_WHEEL.md) for full details.

## Overlay Chat System

Reusable chat overlay used by the Screen Question feature. Designed for future reuse (Quick Ask, Contextual Help).

### Key Design: Context + Config

The system is **generic** -- zero feature-specific logic in the overlay-chat components, features, or state. Each consumer provides:

- **`ChatContext`**: What the AI is analyzing (`screenshot`, `text`, or `none`)
- **`OverlayChatConfig`**: Event names for streaming + Tauri command to invoke

### Files

```
features/overlay-chat/
├── index.ts                      # Exports
├── types.ts                      # ChatMessage, ChatContext, OverlayChatConfig
├── overlay-chat-controller.ts    # sendMessage, initListeners, resetChat
└── constants.ts                  # MAX_VISIBLE_MESSAGES, AUTO_SCROLL_THRESHOLD

state/
└── overlay-chat.svelte.ts        # messages[], isStreaming, error

components/overlay-chat/
├── OverlayChat.svelte            # Container: header, thumbnail, messages, input
├── OverlayChatMessage.svelte     # Message bubble (user/assistant, streaming cursor)
└── OverlayChatInput.svelte       # Textarea + send button (Enter to send, Shift+Enter for newline)
```

See [features/SCREEN_QUESTION.md](./features/SCREEN_QUESTION.md) for the first consumer implementation.

## Primitive Components

Located in `components/ui/primitives/`:

| Component | Props | Usage |
|-----------|-------|-------|
| `Button.svelte` | variant, size, disabled, onclick | Action buttons |
| `Kbd.svelte` | key | Keyboard key display |
| `Icon.svelte` | name, size | SVG icons |
| `Select.svelte` | options, value, onchange | Dropdown select |

## Pattern Components

Located in `components/ui/patterns/`. Composite components built from primitives.

| Component | Props | Usage |
|-----------|-------|-------|
| `Card.svelte` | title?, children | Section card wrapper |
| `ErrorBanner.svelte` | message | Page-level error surface |
| `FormField.svelte` | label, id?, error?, hint?, `saveStatus?`, children | Label + input + message slot. When `saveStatus && status !== 'idle'`, renders `<SaveIndicator>` in place of the hint/error line. |
| `PageHeader.svelte` | title, subtitle?, backHref?, backLabel?, actions? | Page title bar |
| `SaveIndicator.svelte` | `status: SaveStatus` | Inline per-field save feedback — 4 visual states (`idle` reserved spacer, `saving` spinner, `saved` check, `error` message). `aria-live="polite"` on the always-present root. |
| `SettingsLinkCard.svelte` | title, description, href | Navigation tile used on the Settings hub |

## Save Feedback Pattern

All settings surfaces (Providers, Actions, App-Settings, Shortcuts, Dashboard) share a single per-field save-feedback primitive pair: the `createSaveStatus()` factory and the `<SaveIndicator>` component.

### Data flow

```
user event (input / select / toggle)
   │
   ▼
page handler calls save<Field>(value)
   │
   ▼
save<Field>() wraps work in withAsyncState(state, fn, {
  loadingKey: 'isSaving',
  onSaving:  () => state.saveStatus[key].markSaving(),
  onSaved:   () => state.saveStatus[key].markSaved(),
  onError:   (m) => state.saveStatus[key].markError(m),
})
   │
   ▼
saveStatus[key] transitions:
  idle → saving → saved (2 s linger) → idle
  (or) saving → error (sticky until next save)
   │
   ▼
<FormField saveStatus={state.saveStatus[key]} /> OR
<SaveIndicator status={state.saveStatus[key]} />
```

### Primitives

- **`$lib/utils/save-status.svelte`** — exports `createSaveStatus()` (reactive `$state` proxy with `status`, `message`, `markSaving()`, `markSaved()`, `markError(msg)`, `reset()`), the `SaveStatus` type, and `SAVE_DEBOUNCE_MS = 500`. A `SAVED_LINGER_MS = 2000` internal constant controls the auto-revert after `markSaved`. `markError` does NOT auto-revert — the error sticks until the next transition.
- **`$lib/components/ui/patterns/SaveIndicator.svelte`** — renders the four visual states. Idle keeps a reserved invisible spacer to prevent layout shift.
- **`$lib/utils/async-state.ts`** — `withAsyncState(state, fn, options)` accepts optional `onSaving`, `onSaved`, `onError` callbacks. Backward-compatible: legacy callers that omit them are unchanged.
- **`$lib/components/ui/patterns/FormField.svelte`** — extended with an optional `saveStatus` prop. When non-idle, the indicator replaces the hint/error line; otherwise the existing hint/error branch runs.

### State module convention

Every state module that persists user input exposes a `saveStatus: Record<FieldKey, SaveStatus>` record, pre-populated at module load via one `createSaveStatus()` call per key (statically known per domain). Example keys used today:

| Module | Keys |
|--------|------|
| `grammar-config`, `translate-config`, `improve-config` | `prompt`, `system_prompt` |
| `screen-question-config` | `system_prompt` |
| `app-settings` | `theme`, `language`, `debug` |
| `dictation-config` | 15 keys (`microphone`, `audio_settings`, `custom_terms`, `topic`, `names`, `background_text`, `language_hints`, `language_identification`, translation_*, etc.). The `DictationFieldKey` union is exported for the page's `saveField<K>()` helper. |
| `shortcuts` | 6 action keys (`dictation`, `grammar`, `translate`, `improve`, `open_menu`, `screen_question`) on the standalone `shortcutsSaveStatus` export |
| `providers-settings` | Credential keys (`openai.apiKey`, `anthropic.apiKey`, `gemini.apiKey`, `grok.apiKey`, `soniox.apiKey`, `local.baseUrl`, `local.protocol`, `local.apiKey`) + 4 task keys (`task.grammar`, `task.translate`, `task.improve`, `task.screen_question`) |

### Wiring example

```typescript
// state/grammar-config.svelte.ts
export const grammarConfigState = $state<{
  prompt: string;
  system_prompt: string;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  saveStatus: Record<'prompt' | 'system_prompt', SaveStatus>;
}>({
  prompt: '',
  system_prompt: '',
  isLoading: false,
  isSaving: false,
  error: null,
  saveStatus: { prompt: createSaveStatus(), system_prompt: createSaveStatus() },
});

export async function saveGrammarPrompt(prompt: string): Promise<void> {
  await withAsyncState(grammarConfigState, async () => {
    await updateGrammarConfig({ prompt, system_prompt: grammarConfigState.system_prompt });
    grammarConfigState.prompt = prompt;
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save grammar config',
    onSaving: () => grammarConfigState.saveStatus.prompt.markSaving(),
    onSaved:  () => grammarConfigState.saveStatus.prompt.markSaved(),
    onError:  (m) => grammarConfigState.saveStatus.prompt.markError(m),
  });
}
```

```svelte
<!-- routes/actions/grammar/+page.svelte -->
<PromptEditor
  value={grammarConfigState.prompt}
  defaultValue={defaultPrompt}
  onInput={onPromptInput}
  onReset={onPromptReset}
  saveStatus={grammarConfigState.saveStatus.prompt}
  ...
/>
```

### Where the pattern applies

- `/settings/providers` — credentials auto-save with `SAVE_DEBOUNCE_MS` debounce; task-assignment rows save immediately (no Save buttons anywhere on this surface).
- `/actions/{grammar, translate, improve, screen-question}` — prompt textareas auto-save with 500 ms debounce via `PromptEditor`; provider/model picker reuses `<TaskAssignmentRow>` with the per-task `saveStatus`.
- `/app-settings` — `<SaveIndicator>` rendered adjacent to each control (theme buttons, language select, debug toggle). `saveAppSettings(fieldKey)` takes the field key as an argument.
- `/shortcuts` — per-row `<SaveIndicator>` on `ShortcutsList`. The canonical `ShortcutEditorModal` keeps its internal `isSaving` / `error` UI unchanged (modal is transient).
- Dashboard (`/`) — `<SaveIndicator>` after the `MicrophoneSelector`.
- Dictation settings page — per-field indicators for topic/names/background (via `PromptEditor`-style `FormField` wiring) plus section-level indicators for `custom_terms` and `audio_settings` (bulk-mutation slots).

### Error recovery

On save error the user's input stays in the UI (no reload from backend). The indicator's error message persists until the next edit triggers a new save; a successful save clears it. There is no explicit Retry button — the next keystroke / change IS the retry.

## CSS Architecture

### Design Tokens in `app.css`

Three-tier token architecture:

```css
/* Tier 1: Primitive palette (never used directly in components) */
:root {
  --brand-500: #ddca4d;
  --neutral-100: #f5f5f5;
}

/* Tier 2: Semantic tokens (used in components) */
:root {
  --color-primary: var(--brand-500);
  --color-background: var(--neutral-100);
  --color-surface: white;
  --color-text: var(--neutral-900);
  --color-text-on-primary: var(--neutral-900);
  --color-text-on-status: white;
}

/* Tier 3: Component tokens (reference semantics) */
:root {
  --sidebar-bg: var(--color-surface);
  --card-shadow: var(--shadow-card);
}
```

### Dark Theme

Dark mode overrides Tier 2 semantic tokens via `[data-theme="dark"]`:

```css
[data-theme="dark"] {
  color-scheme: dark;
  --color-text: #e5e7eb;
  --color-background: #111827;
  --color-surface: #1f2937;
  --color-card-bg: #1f2937;
  /* ... */
}
```

Theme is applied reactively in `+layout.svelte`:

```typescript
$effect(() => {
  document.documentElement.setAttribute('data-theme', appSettingsState.theme);
});
```

**Rules**: Never use hardcoded colors. Always use CSS variables. Use `--color-text-on-primary` on brand surfaces and `--color-text-on-status` on success or danger surfaces.

### Component Styles

Styles in component `<style>` blocks using variables:

```svelte
<style>
  .button {
    background: var(--color-primary);
    padding: var(--spacing-sm);
  }
</style>
```

## i18n System

Lightweight custom i18n — no dependencies, ~40 lines.

### Usage

```typescript
import { t } from '$lib/i18n';

// Simple translation
t('nav.dashboard')  // → "Dashboard" (en) / "Tableau de bord" (fr)

// With interpolation
t('shortcuts.editor_title', { action: 'Dictation' })  // → "Edit Shortcut: Dictation"
```

### How It Works

- Locale read reactively from `appSettingsState.language`
- English loaded eagerly; French/Spanish lazy-loaded on demand in `+layout.svelte`
- Fallback chain: current locale → English → raw key
- Translation files: flat JSON with dot-notation keys in `src/lib/i18n/locales/`

### Adding Translations

1. Add key to `en.json`: `"area.key": "English text"`
2. Add same key to `fr.json` and `es.json`
3. Use `t('area.key')` in components

**All user-visible strings must use `t()`** — no hardcoded English in templates.

## File Size Guidelines

| Type | Target | Hard Limit |
|------|--------|------------|
| All files | <200 lines | 300 lines |

If a file exceeds the hard limit, split it.
