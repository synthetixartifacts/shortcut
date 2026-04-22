/**
 * Providers Settings state — orchestrates the settings page's provider form:
 * credentials, per-provider model discovery, and task→provider assignments.
 *
 * PHASE 5 (livesave): migrated from explicit Save button + dirty/saved
 * scaffolding to per-field auto-save. `config` is the single in-memory mirror
 * of the backend config (mutated in place by credential inputs and task rows);
 * every mutation routes through `saveProviderCredential` or
 * `saveTaskAssignment`, each of which flips its own `saveStatus` slot.
 */

import { getProviderModels, getProvidersConfig, updateProvidersConfig } from '$lib/api/tauri';
import {
  createDefaultProvidersConfig,
  getAllProviderIds,
  normalizeProvidersConfig,
  type ProviderId,
} from '$lib/features/providers';
import type { ProviderModelInfo, ProvidersConfig } from '$lib/types';
import { withAsyncState } from '$lib/utils/async-state';
import { extractErrorMessage } from '$lib/utils/error';
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

// Sync helpers live in `providers-settings-sync.ts` and per-row task handlers
// in `providers-settings-tasks.ts` to keep this file under the 300-line hard
// cap (CLAUDE.md file-size rule). Re-export the public surface so existing
// callers keep a single import site.
import { syncTaskAssignmentsForProvider } from './providers-settings-sync';
export { getTaskModelOptions, syncTaskAssignmentModel } from './providers-settings-sync';
export {
  handleTaskModelChange,
  handleTaskProviderChange,
  handleTaskVisionChange,
} from './providers-settings-tasks';

export const taskRows = [
  { key: 'grammar' as const, labelKey: 'settings.task_grammar', hintKey: null },
  { key: 'translate' as const, labelKey: 'settings.task_translate', hintKey: null },
  { key: 'improve' as const, labelKey: 'settings.task_improve', hintKey: null },
  { key: 'screen_question' as const, labelKey: 'settings.task_screen_question', hintKey: 'settings.task_screen_question_hint' },
];

/** Provider ids that own a credential input in the Providers page. */
export type CredentialProviderId = 'openai' | 'anthropic' | 'gemini' | 'grok' | 'soniox' | 'local';

/** Static key set for the save-status record; matches the MASTER_PLAN §Key Decisions table. */
const CREDENTIAL_SAVE_KEYS = [
  'openai.apiKey',
  'anthropic.apiKey',
  'gemini.apiKey',
  'grok.apiKey',
  'soniox.apiKey',
  'local.baseUrl',
  'local.protocol',
  'local.apiKey',
] as const;
const TASK_SAVE_KEYS = [
  'task.grammar',
  'task.translate',
  'task.improve',
  'task.screen_question',
] as const;

type ProvidersSaveKey = (typeof CREDENTIAL_SAVE_KEYS)[number] | (typeof TASK_SAVE_KEYS)[number];

function buildSaveStatusRecord(): Record<ProvidersSaveKey, SaveStatus> {
  const record = {} as Record<ProvidersSaveKey, SaveStatus>;
  for (const key of CREDENTIAL_SAVE_KEYS) record[key] = createSaveStatus();
  for (const key of TASK_SAVE_KEYS) record[key] = createSaveStatus();
  return record;
}

export const providersSettingsState = $state<{
  config: ProvidersConfig;
  isSaving: boolean;
  error: string | null;
  saveStatus: Record<ProvidersSaveKey, SaveStatus>;
  models: Partial<Record<ProviderId, ProviderModelInfo[]>>;
  refreshingModels: Partial<Record<ProviderId, boolean>>;
  /** Per-provider discovery error — Settings → Providers only (MASTER_PLAN D9). */
  discoveryErrors: Partial<Record<ProviderId, string>>;
}>({
  config: createDefaultProvidersConfig(),
  isSaving: false,
  error: null,
  saveStatus: buildSaveStatusRecord(),
  models: {},
  refreshingModels: {},
  discoveryErrors: {},
});

/** Load providers config + kick off model discovery for configured providers. */
export async function loadProvidersSettings(): Promise<void> {
  try {
    const cfg = await getProvidersConfig();
    providersSettingsState.config = normalizeProvidersConfig(cfg);
  } catch {
    // Fresh install — defaults already populated.
  }
  await refreshConfiguredProviderModels();
}

/**
 * Persist a single credential field. Caller has already mutated
 * `config.credentials[...]` for UI responsiveness; this writes the normalized
 * config to disk and flips the matching `saveStatus` slot. On success, also
 * re-runs model discovery for the provider whose credential changed — the
 * passive readiness re-check (MASTER_PLAN §Key Decisions #1).
 */
export async function saveProviderCredential(
  providerId: CredentialProviderId,
  field: 'apiKey' | 'baseUrl' | 'protocol',
  value: string,
): Promise<void> {
  const creds = providersSettingsState.config.credentials;
  if (providerId === 'local' && field === 'baseUrl') {
    creds.local.base_url = value;
    // URL change invalidates the cached protocol detection (MASTER_PLAN R3).
    creds.local.detected_protocol = null;
  } else if (providerId === 'local' && field === 'protocol') {
    creds.local.protocol = value as LocalProtocol;
    // Protocol edit also clears the cache so the next discovery re-runs the
    // detection race (useful when the user flips Auto -> manual -> Auto).
    creds.local.detected_protocol = null;
  } else if (providerId === 'local' && field === 'apiKey') {
    creds.local.api_key = value || null;
  } else if (providerId === 'openai' && field === 'apiKey') {
    creds.openai_api_key = value;
  } else if (providerId === 'anthropic' && field === 'apiKey') {
    creds.anthropic_api_key = value;
  } else if (providerId === 'gemini' && field === 'apiKey') {
    creds.gemini_api_key = value;
  } else if (providerId === 'grok' && field === 'apiKey') {
    creds.grok_api_key = value;
  } else if (providerId === 'soniox' && field === 'apiKey') {
    creds.soniox_api_key = value;
  }
  const statusKey: ProvidersSaveKey = resolveStatusKey(providerId, field);

  await withAsyncState(providersSettingsState, async () => {
    const normalized = normalizeProvidersConfig(providersSettingsState.config);
    await updateProvidersConfig(normalized);
    providersSettingsState.config = normalized;
  }, {
    loadingKey: 'isSaving',
    errorFallback: 'Failed to save provider credential',
    onSaving: () => providersSettingsState.saveStatus[statusKey].markSaving(),
    onSaved: () => providersSettingsState.saveStatus[statusKey].markSaved(),
    onError: (m) => providersSettingsState.saveStatus[statusKey].markError(m),
  });

  // Passive readiness re-check — only LLM providers have live model lists.
  // Local refreshes after URL/protocol edits so the detection race re-runs
  // and the "Detected: X" badge reflects the latest server.
  if (providerId !== 'soniox' && shouldLoadProviderModels(providerId)) {
    void refreshProviderModels(providerId);
  }
}

type LocalProtocol = 'auto' | 'ollama' | 'openai_compatible';

function resolveStatusKey(
  providerId: CredentialProviderId,
  field: 'apiKey' | 'baseUrl' | 'protocol',
): ProvidersSaveKey {
  if (providerId === 'local') {
    if (field === 'baseUrl') return 'local.baseUrl';
    if (field === 'protocol') return 'local.protocol';
    return 'local.apiKey';
  }
  return `${providerId}.apiKey` as ProvidersSaveKey;
}

export async function refreshConfiguredProviderModels(): Promise<void> {
  for (const providerId of getAllProviderIds()) {
    if (!shouldLoadProviderModels(providerId)) {
      providersSettingsState.models[providerId] = [];
      syncTaskAssignmentsForProvider(providerId);
    }
  }

  await Promise.all(
    getAllProviderIds()
      .filter((providerId) => shouldLoadProviderModels(providerId))
      .map((providerId) => refreshProviderModels(providerId))
  );
}

export async function refreshProviderModels(providerId: ProviderId): Promise<void> {
  providersSettingsState.refreshingModels[providerId] = true;
  try {
    providersSettingsState.models[providerId] = await getProviderModels(providerId);
    delete providersSettingsState.discoveryErrors[providerId];
    // For Local, the backend may have written `detected_protocol` during the
    // auto-detect race. Re-read the nested `local` block so the "Detected: X"
    // badge updates immediately without waiting for the next page mount.
    // Other fields are left untouched (the user may have unsaved edits).
    if (providerId === 'local') {
      try {
        const fresh = await getProvidersConfig();
        providersSettingsState.config.credentials.local.detected_protocol =
          fresh.credentials.local?.detected_protocol ?? null;
      } catch {
        // Non-fatal — detection badge will lag one cycle.
      }
    }
  } catch (err) {
    providersSettingsState.models[providerId] = [];
    providersSettingsState.discoveryErrors[providerId] = extractErrorMessage(err);
    // MASTER_PLAN D9 / G13: never mutate task_assignments on probe failure.
    // `syncTaskAssignmentModel`'s empty-options branch preserves the user pick.
  } finally {
    providersSettingsState.refreshingModels[providerId] = false;
  }
  syncTaskAssignmentsForProvider(providerId);
}

/**
 * Clear the cached `detected_protocol` for Local and re-run discovery.
 *
 * User-triggered "Re-detect" affordance in the Providers page: when the
 * auto-detect cache points at the wrong adapter (e.g. stuck on "ollama" for
 * an LM Studio endpoint), the user clicks this to force the protocol race
 * to re-run without having to edit the URL as a workaround. Under the hood
 * it persists `detected_protocol = null`, then kicks off the same discovery
 * pipeline as a credential save.
 */
export async function redetectLocalProtocol(): Promise<void> {
  const creds = providersSettingsState.config.credentials;
  creds.local.detected_protocol = null;
  const statusKey: ProvidersSaveKey = 'local.baseUrl';

  await withAsyncState(providersSettingsState, async () => {
    const normalized = normalizeProvidersConfig(providersSettingsState.config);
    await updateProvidersConfig(normalized);
    providersSettingsState.config = normalized;
  }, {
    loadingKey: 'isSaving',
    errorFallback: 'Failed to clear detected protocol',
    onSaving: () => providersSettingsState.saveStatus[statusKey].markSaving(),
    onSaved: () => providersSettingsState.saveStatus[statusKey].markSaved(),
    onError: (m) => providersSettingsState.saveStatus[statusKey].markError(m),
  });

  if (shouldLoadProviderModels('local')) {
    await refreshProviderModels('local');
  }
}

/** Does this provider have the credential required to list its models? */
export function shouldLoadProviderModels(providerId: string): boolean {
  const { credentials } = providersSettingsState.config;
  switch (providerId) {
    case 'openai':
      return credentials.openai_api_key.trim().length > 0;
    case 'anthropic':
      return credentials.anthropic_api_key.trim().length > 0;
    case 'gemini':
      return credentials.gemini_api_key.trim().length > 0;
    case 'grok':
      return credentials.grok_api_key.trim().length > 0;
    case 'local':
      return credentials.local.base_url.trim().length > 0;
    default:
      return false;
  }
}

