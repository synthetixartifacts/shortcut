/**
 * Translate Configuration State
 *
 * Manages the translate prompt template + system prompt with persistence via
 * backend config. Defaults live in Rust (single source of truth) and are
 * fetched via get_default_translate_config.
 */

import type { TranslateConfig } from '$lib/types';
import { getConfig, getDefaultTranslateConfig, updateTranslateConfig } from '$lib/api/tauri';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

/** Per-field save-status keys exposed by `translateConfigState.saveStatus`. */
type TranslateFieldKey = 'prompt' | 'system_prompt';

/** State for translate configuration */
export const translateConfigState = $state<{
  prompt: string;
  system_prompt: string;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  saveStatus: Record<TranslateFieldKey, SaveStatus>;
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

/** Build a full TranslateConfig from current state with optional overrides */
function buildConfig(overrides: Partial<TranslateConfig>): TranslateConfig {
  return {
    prompt: overrides.prompt ?? translateConfigState.prompt,
    system_prompt: overrides.system_prompt ?? translateConfigState.system_prompt,
  };
}

/** Persist a full TranslateConfig and sync local state */
async function persist(next: TranslateConfig, logLabel: string): Promise<void> {
  await updateTranslateConfig(next);
  translateConfigState.prompt = next.prompt;
  translateConfigState.system_prompt = next.system_prompt;
  await log(`[TranslateConfig] ${logLabel}`);
}

/**
 * Load translate config from backend
 */
export async function loadTranslateConfig(): Promise<void> {
  await withAsyncState(translateConfigState, async () => {
    const appConfig = await getConfig();
    const loaded = appConfig.translate;
    const defaults = await getDefaultTranslateConfig();
    translateConfigState.prompt = loaded?.prompt ?? defaults.prompt;
    translateConfigState.system_prompt = loaded?.system_prompt ?? defaults.system_prompt;
    await log('[TranslateConfig] Loaded');
  }, { errorFallback: 'Failed to load translate config' });
}

/**
 * Save translate user prompt
 */
export async function saveTranslatePrompt(prompt: string): Promise<void> {
  await withAsyncState(translateConfigState, async () => {
    await persist(buildConfig({ prompt }), 'Saved prompt');
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save translate config',
    onSaving: () => translateConfigState.saveStatus.prompt.markSaving(),
    onSaved: () => translateConfigState.saveStatus.prompt.markSaved(),
    onError: (m) => translateConfigState.saveStatus.prompt.markError(m),
  });
}

/**
 * Save translate system prompt
 */
export async function saveTranslateSystemPrompt(system_prompt: string): Promise<void> {
  await withAsyncState(translateConfigState, async () => {
    await persist(buildConfig({ system_prompt }), 'Saved system prompt');
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save translate config',
    onSaving: () => translateConfigState.saveStatus.system_prompt.markSaving(),
    onSaved: () => translateConfigState.saveStatus.system_prompt.markSaved(),
    onError: (m) => translateConfigState.saveStatus.system_prompt.markError(m),
  });
}

/**
 * Reset user prompt to default
 */
export async function resetTranslatePrompt(): Promise<void> {
  const defaults = await getDefaultTranslateConfig();
  await saveTranslatePrompt(defaults.prompt);
}

/**
 * Reset system prompt to default
 */
export async function resetTranslateSystemPrompt(): Promise<void> {
  const defaults = await getDefaultTranslateConfig();
  await saveTranslateSystemPrompt(defaults.system_prompt);
}
