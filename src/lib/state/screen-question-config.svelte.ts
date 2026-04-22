/**
 * Screen-Question Configuration State
 *
 * Manages the screen-question system prompt with persistence via backend
 * config. Defaults live in Rust (single source of truth) and are fetched via
 * get_default_screen_question_config.
 *
 * Only field is `system_prompt` — the user message is built at dispatch time
 * in `screen_capture::send_screen_question` from the captured screenshot and
 * the user's typed question.
 */

import {
  getConfig,
  getDefaultScreenQuestionConfig,
  updateScreenQuestionConfig,
} from '$lib/api/tauri';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

/** Per-field save-status keys exposed by `screenQuestionConfigState.saveStatus`. */
type ScreenQuestionFieldKey = 'system_prompt';

/** State for screen-question configuration */
export const screenQuestionConfigState = $state<{
  system_prompt: string;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  saveStatus: Record<ScreenQuestionFieldKey, SaveStatus>;
}>({
  system_prompt: '',
  isLoading: false,
  isSaving: false,
  error: null,
  saveStatus: {
    system_prompt: createSaveStatus(),
  },
});

/**
 * Load screen-question config from backend
 */
export async function loadScreenQuestionConfig(): Promise<void> {
  await withAsyncState(screenQuestionConfigState, async () => {
    const appConfig = await getConfig();
    const loaded = appConfig.screen_question;
    if (loaded?.system_prompt !== undefined) {
      screenQuestionConfigState.system_prompt = loaded.system_prompt;
    } else {
      const defaults = await getDefaultScreenQuestionConfig();
      screenQuestionConfigState.system_prompt = defaults.system_prompt;
    }
    await log('[ScreenQuestionConfig] Loaded');
  }, { errorFallback: 'Failed to load screen-question config' });
}

/**
 * Save screen-question system prompt
 */
export async function saveScreenQuestionSystemPrompt(system_prompt: string): Promise<void> {
  await withAsyncState(screenQuestionConfigState, async () => {
    await updateScreenQuestionConfig({ system_prompt });
    screenQuestionConfigState.system_prompt = system_prompt;
    await log('[ScreenQuestionConfig] Saved system prompt');
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save screen-question config',
    onSaving: () => screenQuestionConfigState.saveStatus.system_prompt.markSaving(),
    onSaved: () => screenQuestionConfigState.saveStatus.system_prompt.markSaved(),
    onError: (m) => screenQuestionConfigState.saveStatus.system_prompt.markError(m),
  });
}

/**
 * Reset system prompt to default
 */
export async function resetScreenQuestionSystemPrompt(): Promise<void> {
  const defaults = await getDefaultScreenQuestionConfig();
  await saveScreenQuestionSystemPrompt(defaults.system_prompt);
}
