<script lang="ts">
  /**
   * TaskAssignmentsForm — matrix of (task → provider + model) rows.
   *
   * PHASE 5 (livesave): the explicit Save button is gone. Each row's
   * `handleTaskProviderChange` / `handleTaskModelChange` persists immediately
   * and flips the row's `<SaveIndicator>`. Page-level errors still surface
   * through the parent page's `<ErrorBanner>` (providersSettingsState.error).
   */
  import SettingsSection from './SettingsSection.svelte';
  import TaskAssignmentRow from './TaskAssignmentRow.svelte';
  import { ErrorBanner } from '$lib/components/ui/patterns';
  import {
    providersSettingsState,
    handleTaskProviderChange,
    handleTaskModelChange,
    getTaskModelOptions,
    refreshProviderModels,
    taskRows,
  } from '$lib/state/providers-settings.svelte';
  import { getConfiguredProviderOptions, hasAnyConfiguredProvider, type ProviderId } from '$lib/features/providers';
  import { t } from '$lib/i18n';

  const anyConfigured = $derived(
    hasAnyConfiguredProvider(providersSettingsState.config.credentials, providersSettingsState.models),
  );
</script>

<SettingsSection title={t('settings.section_task_assignments')}>
  <p class="section-desc">{t('settings.task_assignments_desc')}</p>

  {#if !anyConfigured}
    <div class="empty-state">
      <p class="empty-title">{t('settings.task_assignments_empty_title')}</p>
      <p class="empty-desc">{t('settings.task_assignments_empty_desc')}</p>
    </div>
  {:else}
    {#each taskRows as task}
      <TaskAssignmentRow
        taskKey={task.key}
        label={t(task.labelKey)}
        hint={task.hintKey ? t(task.hintKey) : null}
        providerId={providersSettingsState.config.task_assignments[task.key].provider_id}
        model={providersSettingsState.config.task_assignments[task.key].model}
        providerOptions={getConfiguredProviderOptions(task.key, providersSettingsState.config.credentials, providersSettingsState.models)}
        modelOptions={getTaskModelOptions(task.key, providersSettingsState.config.task_assignments[task.key].provider_id)}
        isRefreshing={!!providersSettingsState.refreshingModels[providersSettingsState.config.task_assignments[task.key].provider_id as ProviderId]}
        saveStatus={providersSettingsState.saveStatus[`task.${task.key}`]}
        onProviderChange={(v) => handleTaskProviderChange(task.key, v)}
        onModelChange={(v) => handleTaskModelChange(task.key, v)}
        onRefreshModels={() => void refreshProviderModels(providersSettingsState.config.task_assignments[task.key].provider_id as ProviderId)}
      />
    {/each}

    <a class="link-row" href="/actions/dictation">
      <div class="link-row-text">
        <span class="link-row-label">{t('settings.task_dictation')}</span>
        <span class="link-row-hint">{t('settings.task_dictation_hint')}</span>
      </div>
      <span class="link-row-arrow" aria-hidden="true">→</span>
    </a>
  {/if}

  {#if providersSettingsState.error}
    <ErrorBanner message={providersSettingsState.error} />
  {/if}
</SettingsSection>

<style>
  .section-desc {
    margin: 0 0 var(--spacing-md);
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }

  .empty-state {
    padding: var(--spacing-lg);
    background: var(--color-kbd-bg);
    border: 1px dashed var(--color-kbd-border);
    border-radius: var(--border-radius-md);
    text-align: center;
  }

  .empty-title {
    margin: 0 0 var(--spacing-xs);
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .empty-desc {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .link-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-sm);
    border-top: 1px solid var(--color-kbd-border);
    text-decoration: none;
    color: inherit;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .link-row:hover {
    background: var(--color-kbd-bg);
  }

  .link-row-text {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .link-row-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .link-row-hint {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .link-row-arrow {
    font-size: 1rem;
    color: var(--color-text-muted);
  }
</style>
