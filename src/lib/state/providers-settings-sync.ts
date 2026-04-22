/**
 * Task-assignment sync helpers for the providers settings page.
 *
 * Extracted from `providers-settings.svelte.ts` (Phase 3) to keep that file
 * under the 300-line hard cap while the Local-LLM feature grows additional
 * credential fields (protocol, apiKey, detected_protocol cache).
 *
 * These helpers mutate the shared `providersSettingsState.config` in place.
 * They never touch the backend; persistence is the caller's job.
 */

import {
  CUSTOM_MODEL_SENTINEL,
  getDefaultModel,
  type ModelOption,
  type ProviderId,
  type TaskKey,
} from '$lib/features/providers';
import { t } from '$lib/i18n';
import type { ProviderModelInfo } from '$lib/types';
import { providersSettingsState, taskRows } from './providers-settings.svelte';

export function syncTaskAssignmentsForProvider(providerId: ProviderId): void {
  for (const task of taskRows) {
    if (providersSettingsState.config.task_assignments[task.key].provider_id === providerId) {
      syncTaskAssignmentModel(task.key);
    }
  }
}

export function syncTaskAssignmentModel(taskKey: TaskKey): void {
  const assignment = providersSettingsState.config.task_assignments[taskKey];
  const options = getLiveModelOptions(taskKey, assignment.provider_id);

  if (!options.length) {
    // MASTER_PLAN D9 / G13: preserve user-picked model on empty discovery.
    // `getTaskModelOptions` surfaces it as a single-option fallback.
    if (!assignment.model.trim()) {
      assignment.model = getDefaultModel(taskKey, assignment.provider_id);
    }
    syncSupportsVisionForAssignment(taskKey);
    return;
  }

  // Phase 4 / US-10: keep custom Local model ids alive even when they're not
  // in the server's discovered list. The row surfaces a non-blocking warning.
  const isLocal = assignment.provider_id === 'local';
  if (!options.some((option) => option.value === assignment.model)) {
    if (isLocal && assignment.model.trim().length > 0) {
      // Custom id — don't mutate. The row handles warning + vision checkbox.
    } else {
      assignment.model =
        options.find((option) => option.value === getDefaultModel(taskKey, assignment.provider_id))?.value
        ?? options[0].value;
    }
  }
  syncSupportsVisionForAssignment(taskKey);
}

function syncSupportsVisionForAssignment(taskKey: TaskKey): void {
  const assignment = providersSettingsState.config.task_assignments[taskKey];
  const models = providersSettingsState.models[assignment.provider_id as ProviderId] ?? [];
  const match = models.find((m: ProviderModelInfo) => m.id === assignment.model);
  if (match) {
    // Discovery is authoritative when it knows the model.
    assignment.supports_vision = match.supports_vision;
  }
  // Otherwise (Phase 4 custom Local model) leave the user's checkbox value alone.
}

export function getTaskModelOptions(taskKey: TaskKey, providerId: string): ModelOption[] {
  const liveOptions = getLiveModelOptions(taskKey, providerId);
  const isLocal = providerId === 'local';

  const baseOptions = liveOptions.length > 0 ? liveOptions : (() => {
    const assignment = providersSettingsState.config.task_assignments[taskKey];
    const fallbackModel = assignment.provider_id === providerId
      ? assignment.model
      : getDefaultModel(taskKey, providerId);
    const model = fallbackModel.trim() || getDefaultModel(taskKey, providerId);
    return [{ value: model, label: model }];
  })();

  // Phase 4 / D5: append "Custom…" for Local only. Cloud providers have
  // authoritative discovery lists and don't expose the free-text escape hatch.
  if (isLocal) {
    return [
      ...baseOptions,
      { value: CUSTOM_MODEL_SENTINEL, label: t('settings.task_model_custom_option') },
    ];
  }
  return baseOptions;
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
