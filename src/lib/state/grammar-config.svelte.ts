/**
 * Grammar Configuration State
 *
 * Manages the grammar prompt template + system prompt with persistence via
 * backend config. Defaults live in Rust (single source of truth) and are
 * fetched via get_default_grammar_config.
 */

import type { GrammarConfig } from '$lib/types';
import { getConfig, getDefaultGrammarConfig, updateGrammarConfig } from '$lib/api/tauri';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

/** Per-field save-status keys exposed by `grammarConfigState.saveStatus`. */
type GrammarFieldKey = 'prompt' | 'system_prompt';

/** State for grammar configuration */
export const grammarConfigState = $state<{
  prompt: string;
  system_prompt: string;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  saveStatus: Record<GrammarFieldKey, SaveStatus>;
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

/** Build a full GrammarConfig from current state with optional overrides */
function buildConfig(overrides: Partial<GrammarConfig>): GrammarConfig {
  return {
    prompt: overrides.prompt ?? grammarConfigState.prompt,
    system_prompt: overrides.system_prompt ?? grammarConfigState.system_prompt,
  };
}

/**
 * Load grammar config from backend
 */
export async function loadGrammarConfig(): Promise<void> {
  await withAsyncState(grammarConfigState, async () => {
    const appConfig = await getConfig();
    const loaded = appConfig.grammar;
    if (loaded?.prompt && loaded?.system_prompt !== undefined) {
      grammarConfigState.prompt = loaded.prompt;
      grammarConfigState.system_prompt = loaded.system_prompt;
    } else {
      const defaults = await getDefaultGrammarConfig();
      grammarConfigState.prompt = loaded?.prompt ?? defaults.prompt;
      grammarConfigState.system_prompt = loaded?.system_prompt ?? defaults.system_prompt;
    }
    await log('[GrammarConfig] Loaded');
  }, { errorFallback: 'Failed to load grammar config' });
}

/**
 * Save grammar user prompt (persists full config)
 */
export async function saveGrammarPrompt(prompt: string): Promise<void> {
  await withAsyncState(grammarConfigState, async () => {
    const next = buildConfig({ prompt });
    await updateGrammarConfig(next);
    grammarConfigState.prompt = next.prompt;
    await log('[GrammarConfig] Saved prompt');
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save grammar config',
    onSaving: () => grammarConfigState.saveStatus.prompt.markSaving(),
    onSaved: () => grammarConfigState.saveStatus.prompt.markSaved(),
    onError: (m) => grammarConfigState.saveStatus.prompt.markError(m),
  });
}

/**
 * Save grammar system prompt (persists full config)
 */
export async function saveGrammarSystemPrompt(system_prompt: string): Promise<void> {
  await withAsyncState(grammarConfigState, async () => {
    const next = buildConfig({ system_prompt });
    await updateGrammarConfig(next);
    grammarConfigState.system_prompt = next.system_prompt;
    await log('[GrammarConfig] Saved system prompt');
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save grammar config',
    onSaving: () => grammarConfigState.saveStatus.system_prompt.markSaving(),
    onSaved: () => grammarConfigState.saveStatus.system_prompt.markSaved(),
    onError: (m) => grammarConfigState.saveStatus.system_prompt.markError(m),
  });
}

/**
 * Reset user prompt to default
 */
export async function resetGrammarPrompt(): Promise<void> {
  const defaults = await getDefaultGrammarConfig();
  await saveGrammarPrompt(defaults.prompt);
}

/**
 * Reset system prompt to default
 */
export async function resetGrammarSystemPrompt(): Promise<void> {
  const defaults = await getDefaultGrammarConfig();
  await saveGrammarSystemPrompt(defaults.system_prompt);
}
