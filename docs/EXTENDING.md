# Extending ShortCut

Guide for adding new features, shortcuts, and AI providers.

## Adding a New Text Transform

Text transforms (like grammar fix, translate) follow a factory pattern.

### 1. Create Feature Module

```bash
src/lib/features/your-feature/
├── your-feature-controller.ts
└── index.ts
```

### 2. Implement Controller with Indicator

Use `withIndicator` for automatic visual feedback:

```typescript
// features/your-feature/your-feature-controller.ts
import { withIndicator } from '$lib/features/indicator';
import { getSelectedText, pasteText } from '$lib/api/tauri';

export async function handleYourFeature(): Promise<void> {
  await withIndicator('processing', async () => {
    const text = await getSelectedText();
    const result = await yourTransform(text);
    await pasteText(result);
  }, { successMessage: 'Done!' });
}
```

### Alternative: Use the Factory Pattern (Recommended)

```typescript
// For transforms that call the backend — same pattern as grammar, translate, improve
import { createTextTransformHandler } from '$lib/features/text-transform/base-controller';
import { yourApiCall } from '$lib/api/tauri';

export const handleYourFeature = createTextTransformHandler({
  name: 'YourFeature',
  statusMessage: 'Processing...',
  successMessage: 'Done!',
  transform: yourApiCall,
  activityType: 'processing',
});
```

### 3. Export from Index

```typescript
// features/your-feature/index.ts
export { yourFeatureController } from './your-feature-controller';
```

### 4. Register Shortcut Handler

In `features/shortcuts/shortcut-dispatcher.ts`:

```typescript
import { handleYourFeature } from '../your-feature';

const shortcutHandlers: Record<ShortcutAction, ShortcutHandler> = {
  // ... existing handlers
  your_feature: handleYourFeature,
};
```

Don't forget to add the action type to `types/index.ts`:

```typescript
export type ShortcutAction =
  | 'dictation_start'
  | 'dictation_stop'
  | 'grammar'
  | 'translate'
  | 'improve'
  | 'open_menu'       // Action Wheel (non-text-transform shortcut)
  | 'screen_question' // Screen Question (overlay chat shortcut)
  | 'your_feature';  // Add here
```

### 5. Add Backend Endpoint (if needed)

For new text tasks, extend `transform_text` in `config/commands.rs`. For tasks that
call a specific provider directly, add a new Tauri command in the relevant module.

## Adding a New LLM Provider

The `providers/` module in `src-tauri/src/` is the extension point for new AI providers.

### 1. Create Provider File

```rust
// src-tauri/src/providers/yourprovider.rs
use crate::{errors::AppError, providers::{ChatRequest, EventSinkFn, LlmProvider, ProviderCapabilities}};

pub struct YourProvider {
    client: reqwest::Client,
    api_key: String,
}

impl YourProvider {
    pub fn new(client: reqwest::Client, api_key: String) -> Self {
        Self { client, api_key }
    }
}

#[async_trait::async_trait]
impl LlmProvider for YourProvider {
    async fn complete(&self, req: &ChatRequest) -> Result<String, AppError> {
        // Call your provider's completion API
        todo!()
    }

    async fn stream(&self, req: &ChatRequest, sink: &EventSinkFn) -> Result<(), AppError> {
        // Stream chunks via sink(chunk)
        todo!()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities { supports_streaming: true, supports_vision: false }
    }

    fn provider_id(&self) -> &'static str { "yourprovider" }
}
```

### 2. Register in Factory

In `providers/mod.rs::get_llm_provider()`, add a match arm:

```rust
"yourprovider" => {
    if creds.yourprovider_api_key.is_empty() {
        return Err(AppError::ProviderError("YourProvider API key not configured".to_string()));
    }
    Ok(Box::new(yourprovider::YourProvider::new(client, creds.yourprovider_api_key)))
}
```

Don't forget to add `pub mod yourprovider;` at the top of `providers/mod.rs`.

### 3. Add Credentials Field

In `config/types/providers.rs::ProviderCredentials` (the `config/types/` directory holds the split type modules — see BACKEND.md for the full layout):

```rust
#[serde(default)]
pub yourprovider_api_key: String,
```

### 4. Implement Discovery

Live model lists feed the Settings task-assignment dropdowns. Every provider needs a discovery fetcher:

```rust
// src-tauri/src/providers/discovery/yourprovider.rs
pub async fn fetch_yourprovider_models(
    client: &reqwest::Client,
    api_key: &str,
) -> Result<Vec<ProviderModelInfo>, AppError> {
    // GET /v1/models (or whatever the provider exposes) → map to ProviderModelInfo
    todo!()
}
```

Register it in `src-tauri/src/providers/discovery/mod.rs::get_provider_models` with a new `"yourprovider"` arm.

### 5. Surface in Settings UI

- Add API key input field in `src/lib/components/settings/ProviderCredentialsForm.svelte`
- Add provider option to task assignment dropdowns via `src/lib/features/providers/provider-catalog.ts`
- Add i18n keys in `src/lib/i18n/locales/{en,fr,es}.json` (en/fr/es parity is enforced — same key count in each)

> **Local is a special case**: it dispatches to either the Ollama-native branch or the OpenAI-compatible branch based on a runtime-resolved `protocol` field. The code lives in `src-tauri/src/providers/local.rs` (factory adapter + `normalize_local_base_url`) and `src-tauri/src/providers/discovery/local.rs` + `.../discovery/openai_compat.rs` (discovery race). New providers do NOT need this pattern — copy from `openai.rs` for cloud providers, or mirror `local.rs` only when modeling another multi-protocol provider.

> **If you are adding another multi-protocol / auto-detect provider, follow these conventions from the Local implementation:**
>
> - **Shape-check, not just status**: each probe must accept only a 2xx response whose body has the expected JSON shape (e.g. `{"models": [...]}`). Permissive servers respond 2xx to unknown paths — a plain status check will pick the wrong adapter. See `parse_ollama_probe` / `parse_openai_probe` in `providers/discovery/local.rs`.
> - **On total failure, don't cache a guess**: if every probe fails, leave `detected_protocol` as `None` and return a typed `AppError::Provider { kind: Network }` whose message names every URL tried. The next probe re-runs fresh instead of sticking on a bad guess.
> - **Schema-version guard for detection-cache changes**: when the probe logic changes in a way that invalidates old cached results, bump a schema-version marker in `AppConfig` and clear the cache exactly once in `migrate_providers_config`. See `local_detection_schema_version` + step 4 in `config/mod.rs::migrate_providers_config`.
> - **Include URL + body preview in error messages**: `providers/http.rs::ensure_ok` and `truncate_preview` already format errors as `"<provider> <URL> failed: HTTP <status> — body: <preview>"`. Any new transport/parse errors you add in the provider body should follow suit (see `ollama.rs`, `openai.rs`, `discovery/mod.rs::parse_json_response` for the pattern) so Debug logs point directly at the failing endpoint.

See [docs/PROVIDERS.md](./PROVIDERS.md) for documentation of existing providers as a reference.

---

## Adding a Per-Action Settings Page

Every AI action (grammar, translate, improve, screen-question) has a dedicated settings page at `/actions/<action>` that edits its system prompt, user prompt (if any), and provider/model assignment. To add one for a new action:

### 1. Extend the Rust config

In `src-tauri/src/config/types/prompts.rs`:

```rust
pub fn default_<action>_system_prompt() -> String {
    "...".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct <Action>Config {
    #[serde(default = "default_<action>_prompt")]      // omit if no user template
    pub prompt: String,
    #[serde(default = "default_<action>_system_prompt")]
    pub system_prompt: String,
}
impl Default for <Action>Config { /* ... */ }
```

Re-export it from `config/types/mod.rs` and add the field to `AppConfig`:
```rust
#[serde(default)]
pub <action>: <Action>Config,
```

### 2. Add Tauri commands

In `src-tauri/src/config/commands.rs`:

```rust
#[tauri::command]
pub fn get_default_<action>_config() -> <Action>Config { <Action>Config::default() }

#[tauri::command]
pub fn update_<action>_config(app, state, <action>: <Action>Config) -> Result<(), String> {
    // optional {text} placeholder validation on the user prompt (NOT on system_prompt)
    // then: config.<action> = <action>; persist_config(&app, &config)
}
```

Register both in `src-tauri/src/lib.rs::invoke_handler!`.

### 3. Prepend the system message at dispatch

In `text_transform.rs` (for text-transform actions) or the action's own dispatch site:

```rust
let mut messages = Vec::with_capacity(2);
if !system_prompt.is_empty() {
    messages.push(ChatMessage { role: "system".to_string(), content: system_prompt });
}
messages.push(ChatMessage { role: "user".to_string(), content: rendered_prompt });
```

No provider file edits are needed — each provider already handles `role: "system"` correctly (see [PROVIDERS.md#system-role-handling-per-provider](./PROVIDERS.md#system-role-handling-per-provider)).

### 4. Add the TypeScript type

In `src/lib/types/index.ts`:
```typescript
export interface <Action>Config {
  prompt: string;         // omit if no user template
  system_prompt: string;
}
```
And add `<action>?: <Action>Config` to `AppConfig`.

### 5. Add API wrappers

In `src/lib/api/config.ts`:
```typescript
export async function update<Action>Config(<action>: <Action>Config): Promise<void> {
  await invokeWithErrorHandling<void>('update_<action>_config', { <action> });
}
export async function getDefault<Action>Config(): Promise<<Action>Config> {
  return invokeWithErrorHandling<<Action>Config>('get_default_<action>_config');
}
```
Re-export through `src/lib/api/tauri.ts`.

### 6. Create the state module

`src/lib/state/<action>-config.svelte.ts` — mirror `improve-config.svelte.ts`:
- `$state` object with the fields + `isLoading` / `isSaving` / `error` + a pre-populated `saveStatus: Record<FieldKey, SaveStatus>` record (one `createSaveStatus()` per field key — see [FRONTEND.md → Save Feedback Pattern](./FRONTEND.md#save-feedback-pattern)).
- `load<Action>Config` (uses `getConfig` → fallback to defaults)
- `save<Action>Prompt` / `save<Action>SystemPrompt` — each wraps `withAsyncState` with `onSaving` / `onSaved` / `onError` hooks that flip the matching `saveStatus[key]`.
- `reset<Action>Prompt` / `reset<Action>SystemPrompt`

### 7. Create the page

`src/routes/actions/<action>/+page.svelte` — use `routes/actions/grammar/+page.svelte` as the template:
- `onMount` → `Promise.all([load<Action>Config(), loadProvidersSettings(), getDefault<Action>Config()])`
- Debounced input handlers using `SAVE_DEBOUNCE_MS` (imported from `$lib/utils/save-status.svelte`) rather than a hardcoded `500`.
- `<PromptEditor>` from `$lib/components/actions/PromptEditor.svelte` for each prompt. Pass `saveStatus={<action>ConfigState.saveStatus.prompt}` (or `.system_prompt`) so the inline `<SaveIndicator>` renders.
- `<TaskAssignmentRow taskKey="<action>" ... saveStatus={providersSettingsState.saveStatus['task.<action>']}>` driven by `providersSettingsState` (same contract as `/settings`).

### 8. Link from the actions hub

In `src/routes/actions/+page.svelte`, add an `<a href="/actions/<action>">` card.

### 9. Expose the per-action keyboard shortcut

Every per-action page mounts the shared `ShortcutSection.svelte` wrapper to display the current binding and open the canonical `ShortcutEditorModal`:

```svelte
<ShortcutSection actionKey="<action>" translationNamespace="<action>_settings" />
```

The component reuses `shortcuts.svelte.ts::updateShortcut`, `getDefaultShortcut`, and `ShortcutEditorModal` — do NOT invent a new editor or state module. Edits go through the same code path as `/shortcuts`, so the two surfaces always agree.

The wrapper expects three keys on the namespace: `section_shortcut`, `shortcut_label`, `button_edit_shortcut` (add them in step 11).

### 10. Register the shortcut handler (if applicable)

If the action is triggered by a keyboard shortcut, register its handler in `features/shortcuts/shortcut-dispatcher.ts` — see [Adding a New Shortcut](#adding-a-new-shortcut) below.

### 11. Add i18n keys

Add the `<action>_settings.*` namespace to all three locales (`src/lib/i18n/locales/{en,fr,es}.json`) — follow the shape of `grammar_settings.*`. Don't forget the three shortcut-section keys consumed by `ShortcutSection.svelte`: `section_shortcut`, `shortcut_label`, `button_edit_shortcut`.

---

## Adding a New Shortcut

### Frontend Only

1. Register handler in `shortcut-dispatcher.ts`
2. Add to shortcuts state in `state/shortcuts.svelte.ts`
3. Update UI if needed

### With Backend Support

1. Add constant in `src-tauri/src/hotkeys/mod.rs`:
```rust
pub const DEFAULT_YOUR_FEATURE_SHORTCUT: &str = "Alt+F";
```

2. Add to `HotkeyConfig` in `config/types/hotkeys.rs` (re-exported from `config/mod.rs`)
3. Register handler in frontend

> **Note**: Not all shortcuts need to be text transforms. The `open_menu` shortcut opens the Action Wheel UI, and `screen_question` opens the Screen Question overlay. See `shortcut-dispatcher.ts` for examples of non-text-transform shortcut handlers.

## Adding a New OverlayChat Feature

The `overlay-chat` system is a reusable chat overlay. To create a new feature that uses it (e.g., Quick Ask, Contextual Help):

### 1. Create a Route

```
src/routes/your-feature/
├── +page.svelte    # Listen for context event, render OverlayChat
└── +layout.ts      # export const ssr = false;
```

### 2. Configure OverlayChat

```svelte
<script lang="ts">
  import { OverlayChat } from '$lib/components/overlay-chat';
  import type { ChatContext, OverlayChatConfig } from '$lib/features/overlay-chat/types';

  const context: ChatContext = { type: 'text', selectedText: '...' };
  const config: OverlayChatConfig = {
    placeholder: 'Ask a question...',
    chunkEvent: 'your-answer-chunk',
    completeEvent: 'your-answer-complete',
    errorEvent: 'your-answer-error',
    sendCommand: 'send_your_question'
  };
</script>

<OverlayChat {context} {config} onClose={handleClose} />
```

### 3. Implement Rust Backend

- Add a Tauri command matching `sendCommand` (e.g., `send_your_question`)
- Emit events matching `chunkEvent`, `completeEvent`, `errorEvent`
- Add window management similar to `screen_capture.rs`

### 4. Register in Layout

Add route check in `+layout.svelte` so overlay renders without the app shell.

See [features/SCREEN_QUESTION.md](./features/SCREEN_QUESTION.md) for the reference implementation.

## Adding a UI Component

### Primitive Component

Add to `components/ui/primitives/`:

```svelte
<!-- primitives/Badge.svelte -->
<script lang="ts">
  interface Props {
    variant?: 'default' | 'success' | 'error';
    children: import('svelte').Snippet;
  }

  let { variant = 'default', children }: Props = $props();
</script>

<span class="badge badge-{variant}">
  {@render children()}
</span>

<style>
  .badge {
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
  }
  .badge-default { background: var(--color-surface-alt); }
  .badge-success { background: var(--color-success); }
  .badge-error { background: var(--color-error); }
</style>
```

Export in `primitives/index.ts`:
```typescript
export { default as Badge } from './Badge.svelte';
```

### Feature Component

Larger components go in `components/ui/`:

```svelte
<!-- components/ui/YourComponent.svelte -->
<script lang="ts">
  import { Button, Icon } from './primitives';
</script>
```

## Adding Save Feedback to a New Field

Follow this recipe whenever you add a new persisted field (input, select, toggle, or bulk section) to any settings surface. The canonical example primitive pair is `createSaveStatus()` + `<SaveIndicator>`.

### 1. Reserve a key on the state module

In the relevant `state/*.svelte.ts`, extend the `saveStatus` record:

```typescript
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

type FieldKey = 'existingField' | 'newField';
export const yourState = $state<{
  /* ... */
  saveStatus: Record<FieldKey, SaveStatus>;
}>({
  /* ... */
  saveStatus: {
    existingField: createSaveStatus(),
    newField: createSaveStatus(),
  },
});
```

### 2. Wire hooks into the save function

```typescript
export async function saveYourField(value: string): Promise<void> {
  await withAsyncState(yourState, async () => {
    await update<Feature>Config({ /* ... */ });
  }, {
    loadingKey: 'isSaving',
    onSaving: () => yourState.saveStatus.newField.markSaving(),
    onSaved:  () => yourState.saveStatus.newField.markSaved(),
    onError:  (m) => yourState.saveStatus.newField.markError(m),
    errorFallback: 'Failed to save <feature>',
  });
}
```

### 3. Render the indicator

Prefer passing `saveStatus` to `<FormField>` when you already use that pattern:

```svelte
<FormField label={t('…')} saveStatus={yourState.saveStatus.newField}>
  <input .../>
</FormField>
```

Or drop a standalone `<SaveIndicator>` next to the control:

```svelte
<SaveIndicator status={yourState.saveStatus.newField} />
```

### 4. Debounce only typed input

For typed input, debounce with `SAVE_DEBOUNCE_MS` from `$lib/utils/save-status.svelte`. For selects, toggles, and single-gesture controls, save immediately on change — no debounce.

See [FRONTEND.md → Save Feedback Pattern](./FRONTEND.md#save-feedback-pattern) for full details and the list of surfaces already using the pattern.

## Adding State

Use `.svelte.ts` files with runes:

```typescript
// state/your-state.svelte.ts
export const yourState = $state({
  isActive: false,
  data: [] as string[]
});

// Derived values
export const hasData = $derived(yourState.data.length > 0);
```

## File Size Guidelines

| Type | Target | Max |
|------|--------|-----|
| All files | <200 | 300 |

**If exceeding max, split the file.**

## Adding Translations (i18n)

When adding user-visible text:

1. Add key to all 3 locale files (`src/lib/i18n/locales/{en,fr,es}.json`)
2. Use `t('area.key')` in components instead of hardcoded strings
3. Key convention: `area.specific_key` (e.g., `nav.dashboard`, `settings.provider_openai_key`)

```typescript
import { t } from '$lib/i18n';

// In template
<h1>{t('your_feature.title')}</h1>

// With interpolation
<p>{t('your_feature.count', { count: items.length })}</p>
```

**All user-visible strings must be translated** — no hardcoded English in Svelte templates.

## Checklist

Before submitting changes:

- [ ] `docker compose run --rm check` passes
- [ ] If you intentionally used a narrower validation profile, note the `CHECK_PROFILE` in the PR
- [ ] No file exceeds 300 lines (hard limit)
- [ ] New exports added to `index.ts` files
- [ ] CSS uses variables from `app.css` (no hardcoded colors)
- [ ] All user-visible strings use `t()` i18n function
- [ ] Translations added to all 3 locale files (en, fr, es)
- [ ] Follows existing patterns
