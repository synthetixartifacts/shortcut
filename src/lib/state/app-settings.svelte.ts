/**
 * App Settings State Management
 *
 * Manages UI preferences: theme, language, debug visibility.
 * Loaded as part of the main settings load and persisted via Tauri commands.
 *
 * Single source of truth: `settingsState.config.app_settings`.
 * `appSettingsState` is a thin reactive facade whose `theme`/`language`/
 * `debugEnabled` are getter/setter-backed properties delegating to the
 * underlying `AppConfig` — no mirrored storage, no drift.
 */

import type { AppConfig, AppSettingsConfig } from '$lib/types';
import { updateAppSettingsConfig } from '$lib/api/tauri';
import { settingsState } from './settings.svelte';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

/** Per-field save-status keys exposed by `appSettingsState.saveStatus`. */
type AppSettingsFieldKey = 'theme' | 'language' | 'debug';

/** Defaults for fields that may be absent before settings finish loading. */
const DEFAULTS: AppSettingsConfig = {
  theme: 'dark',
  language: 'en',
  debug_enabled: false,
};

/**
 * Non-persisted UI state (loading / error / per-field save status) lives on
 * its own `$state` object. `saveStatus` is per-field (`theme | language |
 * debug`) — pages pass the field key into `saveAppSettings()` so the correct
 * indicator flips.
 */
const uiState = $state<{
  isSaving: boolean;
  error: string | null;
  saveStatus: Record<AppSettingsFieldKey, SaveStatus>;
}>({
  isSaving: false,
  error: null,
  saveStatus: {
    theme: createSaveStatus(),
    language: createSaveStatus(),
    debug: createSaveStatus(),
  },
});

/**
 * Read app_settings from the shared config (or defaults if not loaded yet).
 *
 * Reads `settingsState.config.app_settings` — since `settingsState` is a
 * `$state` proxy, every access is reactive and Svelte tracks dependencies.
 */
function readField<K extends keyof AppSettingsConfig>(key: K): AppSettingsConfig[K] {
  return settingsState.config?.app_settings?.[key] ?? DEFAULTS[key];
}

/**
 * Write an app_settings field back into the shared config.
 *
 * Lazily initializes `config` / `config.app_settings` if they haven't loaded
 * yet so setters don't crash when called before `loadSettings` resolves
 * (e.g. on a hot-reload transient).
 */
function writeField<K extends keyof AppSettingsConfig>(
  key: K,
  value: AppSettingsConfig[K]
): void {
  const config = settingsState.config;
  if (!config) {
    // Config not loaded yet — stash a partial AppConfig so the write survives
    // until loadSettings() overwrites it. This is a cold-start edge case.
    settingsState.config = {
      hotkeys: {} as AppConfig['hotkeys'],
      user: { name: '', preferred_language: 'en' },
      app_settings: { ...DEFAULTS, [key]: value },
    } as AppConfig;
    return;
  }
  if (!config.app_settings) {
    config.app_settings = { ...DEFAULTS, [key]: value };
    return;
  }
  config.app_settings[key] = value;
}

/**
 * Reactive facade over `settingsState.config.app_settings`.
 *
 * Consumers can read/write `appSettingsState.theme` exactly as before, but
 * every access delegates to the shared config so there is only one source
 * of truth and no mirror to keep in sync.
 */
export const appSettingsState = {
  get theme(): string {
    return readField('theme');
  },
  set theme(value: string) {
    writeField('theme', value);
  },
  get language(): string {
    return readField('language');
  },
  set language(value: string) {
    writeField('language', value);
  },
  get debugEnabled(): boolean {
    return readField('debug_enabled');
  },
  set debugEnabled(value: boolean) {
    writeField('debug_enabled', value);
  },
  get isSaving(): boolean {
    return uiState.isSaving;
  },
  set isSaving(value: boolean) {
    uiState.isSaving = value;
  },
  get error(): string | null {
    return uiState.error;
  },
  set error(value: string | null) {
    uiState.error = value;
  },
  get saveStatus(): Record<AppSettingsFieldKey, SaveStatus> {
    return uiState.saveStatus;
  },
};

/**
 * No-op kept for backwards compatibility with existing onMount callers.
 *
 * Previously this copied `settingsState.config.app_settings` into a parallel
 * `appSettingsState` — now that they share storage, no sync is needed.
 */
export function syncAppSettings(): void {
  // Intentionally empty: appSettingsState is a live view of settingsState.
}

/**
 * Save app settings to backend.
 *
 * `fieldKey` is optional for backwards compatibility: when provided, the
 * corresponding `saveStatus[fieldKey]` entry flips through
 * `saving → saved` (or `error`); when omitted, no indicator flips. Phase 4
 * migrates the three page-level call sites (theme / language / debug) to
 * pass their field key explicitly.
 */
export async function saveAppSettings(fieldKey?: AppSettingsFieldKey): Promise<void> {
  const payload: AppSettingsConfig = {
    theme: appSettingsState.theme,
    language: appSettingsState.language,
    debug_enabled: appSettingsState.debugEnabled,
  };

  await withAsyncState(uiState, async () => {
    await updateAppSettingsConfig(payload);
    await log('[AppSettings] Saved app settings');
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save app settings',
    onSaving: fieldKey ? () => uiState.saveStatus[fieldKey].markSaving() : undefined,
    onSaved: fieldKey ? () => uiState.saveStatus[fieldKey].markSaved() : undefined,
    onError: fieldKey ? (m) => uiState.saveStatus[fieldKey].markError(m) : undefined,
  });
}
