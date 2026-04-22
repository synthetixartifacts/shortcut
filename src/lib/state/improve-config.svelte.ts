/**
 * Improve Configuration State
 *
 * Manages the improve prompt template + system prompt with persistence via
 * backend config. Defaults are defined in Rust (single source of truth) and
 * fetched via get_default_improve_config.
 */

import type { ImproveConfig } from '$lib/types';
import { getConfig, getDefaultImproveConfig, updateImproveConfig } from '$lib/api/tauri';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

/** Per-field save-status keys exposed by `improveConfigState.saveStatus`. */
type ImproveFieldKey = 'prompt' | 'system_prompt';

/** State for improve configuration */
export const improveConfigState = $state<{
  prompt: string;
  system_prompt: string;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  saveStatus: Record<ImproveFieldKey, SaveStatus>;
}>({
  prompt: '',
  system_prompt: '',
  isLoading: false,
  isSaving: false,
  error: null,
  saveStatus: {
    prompt: createSaveStatus(),
    system_prompt: createSaveStatus(),
  },
});

/** Build a full ImproveConfig from current state with optional overrides */
function buildConfig(overrides: Partial<ImproveConfig>): ImproveConfig {
  return {
    prompt: overrides.prompt ?? improveConfigState.prompt,
    system_prompt: overrides.system_prompt ?? improveConfigState.system_prompt,
  };
}

/**
 * Load improve config from backend
 */
export async function loadImproveConfig(): Promise<void> {
  await withAsyncState(improveConfigState, async () => {
    const appConfig = await getConfig();
    const loaded = appConfig.improve;
    if (loaded?.prompt && loaded?.system_prompt !== undefined) {
      improveConfigState.prompt = loaded.prompt;
      improveConfigState.system_prompt = loaded.system_prompt;
    } else {
      // Fall back to Rust defaults for missing fields
      const defaults = await getDefaultImproveConfig();
      improveConfigState.prompt = loaded?.prompt ?? defaults.prompt;
      improveConfigState.system_prompt = loaded?.system_prompt ?? defaults.system_prompt;
    }
    await log('[ImproveConfig] Loaded');
  }, { errorFallback: 'Failed to load improve config' });
}

/**
 * Save improve user prompt (persists full config)
 */
export async function saveImprovePrompt(prompt: string): Promise<void> {
  await withAsyncState(improveConfigState, async () => {
    const next = buildConfig({ prompt });
    await updateImproveConfig(next);
    improveConfigState.prompt = next.prompt;
    await log('[ImproveConfig] Saved prompt');
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save improve config',
    onSaving: () => improveConfigState.saveStatus.prompt.markSaving(),
    onSaved: () => improveConfigState.saveStatus.prompt.markSaved(),
    onError: (m) => improveConfigState.saveStatus.prompt.markError(m),
  });
}

/**
 * Save improve system prompt (persists full config)
 */
export async function saveImproveSystemPrompt(system_prompt: string): Promise<void> {
  await withAsyncState(improveConfigState, async () => {
    const next = buildConfig({ system_prompt });
    await updateImproveConfig(next);
    improveConfigState.system_prompt = next.system_prompt;
    await log('[ImproveConfig] Saved system prompt');
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save improve config',
    onSaving: () => improveConfigState.saveStatus.system_prompt.markSaving(),
    onSaved: () => improveConfigState.saveStatus.system_prompt.markSaved(),
    onError: (m) => improveConfigState.saveStatus.system_prompt.markError(m),
  });
}

/**
 * Reset user prompt to default (fetched from Rust backend)
 */
export async function resetImprovePrompt(): Promise<void> {
  const defaults = await getDefaultImproveConfig();
  await saveImprovePrompt(defaults.prompt);
}

/**
 * Reset system prompt to default (fetched from Rust backend)
 */
export async function resetImproveSystemPrompt(): Promise<void> {
  const defaults = await getDefaultImproveConfig();
  await saveImproveSystemPrompt(defaults.system_prompt);
}
