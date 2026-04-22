/**
 * Settings State Management
 *
 * Manages application configuration.
 */

import type { AppConfig } from '$lib/types';
import { getConfig } from '$lib/api/tauri';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';

/**
 * Settings state
 */
export const settingsState = $state<{
  config: AppConfig | null;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
}>({
  config: null,
  isLoading: false,
  isSaving: false,
  error: null,
});

/**
 * Load settings from backend
 */
export async function loadSettings(): Promise<void> {
  await withAsyncState(settingsState, async () => {
    const config = await getConfig();
    settingsState.config = config;
    await log('[Settings] Loaded configuration');
  }, { errorFallback: 'Failed to load settings' });
}
