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
  getDefaultModel,
  normalizeProvidersConfig,
  type ModelOption,
  type ProviderId,
  type TaskKey,
} from '$lib/features/providers';
import type { ProviderModelInfo, ProvidersConfig } from '$lib/types';
import { withAsyncState } from '$lib/utils/async-state';
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

export const taskRows = [
  { key: 'grammar' as const, labelKey: 'settings.task_grammar', hintKey: null },
  { key: 'translate' as const, labelKey: 'settings.task_translate', hintKey: null },
  { key: 'improve' as const, labelKey: 'settings.task_improve', hintKey: null },
  { key: 'screen_question' as const, labelKey: 'settings.task_screen_question', hintKey: 'settings.task_screen_question_hint' },
];

/** Provider ids that own a credential input in the Providers page. */
export type CredentialProviderId = 'openai' | 'anthropic' | 'gemini' | 'grok' | 'soniox' | 'ollama';

/** Static key set for the save-status record; matches the MASTER_PLAN §Key Decisions table. */
const CREDENTIAL_SAVE_KEYS = [
  'openai.apiKey',
  'anthropic.apiKey',
  'gemini.apiKey',
  'grok.apiKey',
  'soniox.apiKey',
  'ollama.baseUrl',
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
}>({
  config: createDefaultProvidersConfig(),
  isSaving: false,
  error: null,
  saveStatus: buildSaveStatusRecord(),
  models: {},
  refreshingModels: {},
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

/** Map a `(providerId, field)` pair onto the persisted credential property. */
function credentialFieldName(providerId: CredentialProviderId, field: 'apiKey' | 'baseUrl'): keyof ProvidersConfig['credentials'] {
  if (field === 'baseUrl') return 'ollama_base_url';
  switch (providerId) {
    case 'openai': return 'openai_api_key';
    case 'anthropic': return 'anthropic_api_key';
    case 'gemini': return 'gemini_api_key';
    case 'grok': return 'grok_api_key';
    case 'soniox': return 'soniox_api_key';
    case 'ollama': return 'ollama_base_url';
  }
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
  field: 'apiKey' | 'baseUrl',
  value: string,
): Promise<void> {
  const fieldName = credentialFieldName(providerId, field);
  providersSettingsState.config.credentials[fieldName] = value;
  const statusKey: ProvidersSaveKey = field === 'baseUrl' ? 'ollama.baseUrl' : `${providerId}.apiKey` as ProvidersSaveKey;

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
  if (providerId !== 'soniox' && shouldLoadProviderModels(providerId)) {
    void refreshProviderModels(providerId);
  }
}

/**
 * Persist a single task assignment (provider + model pair). Caller has already
 * mutated `config.task_assignments[taskKey]` for UI responsiveness.
 */
async function saveTaskAssignment(taskKey: TaskKey): Promise<void> {
  const statusKey: ProvidersSaveKey = `task.${taskKey}`;
  await withAsyncState(providersSettingsState, async () => {
    const normalized = normalizeProvidersConfig(providersSettingsState.config);
    await updateProvidersConfig(normalized);
    providersSettingsState.config = normalized;
  }, {
    loadingKey: 'isSaving',
    errorFallback: 'Failed to save task assignment',
    onSaving: () => providersSettingsState.saveStatus[statusKey].markSaving(),
    onSaved: () => providersSettingsState.saveStatus[statusKey].markSaved(),
    onError: (m) => providersSettingsState.saveStatus[statusKey].markError(m),
  });
}

/** Called when the user changes the provider for a task. Resets model + persists + refetches. */
export function handleTaskProviderChange(taskKey: TaskKey, providerId: string): void {
  providersSettingsState.config.task_assignments[taskKey].provider_id = providerId;
  providersSettingsState.config.task_assignments[taskKey].model = getDefaultModel(taskKey, providerId);
  providersSettingsState.config.task_assignments[taskKey].supports_vision = null;
  syncTaskAssignmentModel(taskKey);

  void saveTaskAssignment(taskKey);

  if (shouldLoadProviderModels(providerId)) {
    void refreshProviderModels(providerId as ProviderId);
  }
}

/**
 * Called when the user picks a model for a task. Persists the discovered
 * `supports_vision` flag alongside the model id so the backend vision gate
 * can rely on per-model capability rather than coarse provider-level flags.
 */
export function handleTaskModelChange(taskKey: TaskKey, model: string): void {
  const providerId = providersSettingsState.config.task_assignments[taskKey].provider_id;
  providersSettingsState.config.task_assignments[taskKey].model = model;
  const models = providersSettingsState.models[providerId as ProviderId] ?? [];
  const match = models.find((m) => m.id === model);
  providersSettingsState.config.task_assignments[taskKey].supports_vision =
    match ? match.supports_vision : null;

  void saveTaskAssignment(taskKey);
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
  } catch {
    providersSettingsState.models[providerId] = [];
  } finally {
    providersSettingsState.refreshingModels[providerId] = false;
  }
  syncTaskAssignmentsForProvider(providerId);
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
    case 'ollama':
      return credentials.ollama_base_url.trim().length > 0;
    default:
      return false;
  }
}

function syncTaskAssignmentsForProvider(providerId: ProviderId): void {
  for (const task of taskRows) {
    if (providersSettingsState.config.task_assignments[task.key].provider_id === providerId) {
      syncTaskAssignmentModel(task.key);
    }
  }
}

function syncTaskAssignmentModel(taskKey: TaskKey): void {
  const assignment = providersSettingsState.config.task_assignments[taskKey];
  const options = getLiveModelOptions(taskKey, assignment.provider_id);

  if (!options.length) {
    if (!assignment.model.trim()) {
      assignment.model = getDefaultModel(taskKey, assignment.provider_id);
    }
    syncSupportsVisionForAssignment(taskKey);
    return;
  }

  if (!options.some((option) => option.value === assignment.model)) {
    assignment.model =
      options.find((option) => option.value === getDefaultModel(taskKey, assignment.provider_id))?.value
      ?? options[0].value;
  }
  syncSupportsVisionForAssignment(taskKey);
}

function syncSupportsVisionForAssignment(taskKey: TaskKey): void {
  const assignment = providersSettingsState.config.task_assignments[taskKey];
  const models = providersSettingsState.models[assignment.provider_id as ProviderId] ?? [];
  const match = models.find((m) => m.id === assignment.model);
  assignment.supports_vision = match ? match.supports_vision : null;
}

export function getTaskModelOptions(taskKey: TaskKey, providerId: string): ModelOption[] {
  const liveOptions = getLiveModelOptions(taskKey, providerId);
  if (liveOptions.length > 0) return liveOptions;

  const assignment = providersSettingsState.config.task_assignments[taskKey];
  const fallbackModel = assignment.provider_id === providerId
    ? assignment.model
    : getDefaultModel(taskKey, providerId);
  const model = fallbackModel.trim() || getDefaultModel(taskKey, providerId);
  return [{ value: model, label: model }];
}

function getLiveModelOptions(taskKey: TaskKey, providerId: string): ModelOption[] {
  const models = providersSettingsState.models[providerId as ProviderId] ?? [];
  return models
    .filter((model) => taskKey !== 'screen_question' || model.supports_vision)
    .map((model) => ({
      value: model.id,
      label: model.label,
    }));
}
