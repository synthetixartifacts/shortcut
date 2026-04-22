/**
 * Task-assignment handlers for the providers settings page.
 *
 * Extracted from `providers-settings.svelte.ts` (Phase 4) to keep that file
 * under the 300-line hard cap. These handlers mutate
 * `providersSettingsState.config.task_assignments[taskKey]` and persist
 * through the shared `saveTaskAssignment` helper.
 */

import { updateProvidersConfig } from '$lib/api/tauri';
import {
  CUSTOM_MODEL_SENTINEL,
  getDefaultModel,
  normalizeProvidersConfig,
  type ProviderId,
  type TaskKey,
} from '$lib/features/providers';
import { withAsyncState } from '$lib/utils/async-state';
import { providersSettingsState, shouldLoadProviderModels, refreshProviderModels } from './providers-settings.svelte';
import { syncTaskAssignmentModel } from './providers-settings-sync';

/**
 * Persist a single task assignment (provider + model pair). Caller has already
 * mutated `config.task_assignments[taskKey]` for UI responsiveness.
 */
async function saveTaskAssignment(taskKey: TaskKey): Promise<void> {
  const statusKey = `task.${taskKey}` as const;
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

  // For Local, getDefaultModel returns '' (no hardcoded default — the user's
  // server may run any model). If discovery already cached a model list, pick
  // the first one so the assignment is immediately usable. If not, leave it
  // empty — the UI renders the "Custom" flow and discovery kicks off below.
  if (providerId === 'local' && !providersSettingsState.config.task_assignments[taskKey].model) {
    const localModels = providersSettingsState.models.local ?? [];
    if (localModels.length > 0) {
      providersSettingsState.config.task_assignments[taskKey].model = localModels[0].id;
      providersSettingsState.config.task_assignments[taskKey].supports_vision = localModels[0].supports_vision;
    }
  }

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
 *
 * Phase 4 / D5: handles the `__custom__` sentinel from Local rows. Picking
 * "Custom…" clears the model so the free-text input renders empty and resets
 * `supports_vision` to null (user must opt-in via the checkbox).
 */
export function handleTaskModelChange(taskKey: TaskKey, model: string): void {
  const assignment = providersSettingsState.config.task_assignments[taskKey];

  if (model === CUSTOM_MODEL_SENTINEL) {
    assignment.model = '';
    assignment.supports_vision = null;
    void saveTaskAssignment(taskKey);
    return;
  }

  assignment.model = model;
  const providerId = assignment.provider_id;
  const models = providersSettingsState.models[providerId as ProviderId] ?? [];
  const match = models.find((m) => m.id === model);
  // Only overwrite supports_vision when discovery has authoritative info.
  // Custom Local ids keep the user's checkbox value.
  if (match) {
    assignment.supports_vision = match.supports_vision;
  }

  void saveTaskAssignment(taskKey);
}

/**
 * Persist a user-supplied vision flag for a custom Local model. Only invoked
 * from rows where the typed id isn't in the discovered list — when discovery
 * knows the model, its flag is authoritative and this override is hidden.
 */
export function handleTaskVisionChange(taskKey: TaskKey, value: boolean | null): void {
  providersSettingsState.config.task_assignments[taskKey].supports_vision = value;
  void saveTaskAssignment(taskKey);
}
