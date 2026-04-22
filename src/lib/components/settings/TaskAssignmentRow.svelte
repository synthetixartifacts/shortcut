<script lang="ts">
  import { t } from '$lib/i18n';
  import { Select, Icon } from '$lib/components/ui/primitives';
  import { SaveIndicator } from '$lib/components/ui/patterns';
  import type { ModelOption, ProviderOption } from '$lib/features/providers';
  import type { SaveStatus } from '$lib/utils/save-status.svelte';

  interface Props {
    taskKey: string;
    label: string;
    hint?: string | null;
    providerId: string;
    model: string;
    providerOptions: ProviderOption[];
    modelOptions: ModelOption[];
    isRefreshing?: boolean;
    saveStatus?: SaveStatus;
    onProviderChange: (value: string) => void;
    onModelChange: (value: string) => void;
    onRefreshModels?: () => void;
  }

  let {
    taskKey,
    label,
    hint = null,
    providerId,
    model,
    providerOptions,
    modelOptions,
    isRefreshing = false,
    saveStatus,
    onProviderChange,
    onModelChange,
    onRefreshModels,
  }: Props = $props();

  const loadingOptions = $derived<ModelOption[]>(
    [{ value: model, label: t('settings.task_model_loading') }],
  );
</script>

<div class="task-row">
  <div class="task-heading">
    <span class="task-label">{label}</span>
    {#if saveStatus}
      <SaveIndicator status={saveStatus} />
    {/if}
  </div>
  {#if hint}
    <span class="task-hint">{hint}</span>
  {/if}
  <div class="task-fields">
    <div class="task-field">
      <label class="field-label" for="provider-{taskKey}">
        {t('settings.task_provider')}
      </label>
      <Select
        id="provider-{taskKey}"
        value={providerId}
        options={providerOptions}
        onchange={onProviderChange}
      />
    </div>
    <div class="task-field">
      <label class="field-label" for="model-{taskKey}">
        {t('settings.task_model')}
      </label>
      <div class="model-row">
        <Select
          id="model-{taskKey}"
          value={model}
          options={isRefreshing ? loadingOptions : modelOptions}
          disabled={isRefreshing}
          onchange={onModelChange}
        />
        {#if onRefreshModels}
          <button
            type="button"
            class="refresh-btn"
            class:spinning={isRefreshing}
            onclick={onRefreshModels}
            disabled={isRefreshing}
            aria-label={t('settings.task_model_refresh')}
            title={t('settings.task_model_refresh')}
          >
            <Icon name="refresh" size={16} />
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .task-row {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) 0;
    border-bottom: 1px solid var(--color-kbd-border);
  }

  .task-row:last-of-type {
    border-bottom: none;
  }

  .task-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
  }

  .task-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .task-hint {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin-top: calc(var(--spacing-xs) * -0.5);
  }

  .task-fields {
    display: flex;
    gap: var(--spacing-md);
    flex-wrap: wrap;
  }

  .task-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    flex: 1;
    min-width: 120px;
  }

  .field-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .model-row {
    display: flex;
    align-items: stretch;
    gap: var(--spacing-xs);
  }

  .model-row :global(.select) {
    flex: 1;
  }

  .refresh-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--input-height, 40px);
    height: var(--input-height, 40px);
    padding: 0;
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-sm);
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease, background 0.15s ease;
    flex-shrink: 0;
  }

  .refresh-btn:hover:not(:disabled) {
    color: var(--color-text);
    border-color: var(--color-primary);
  }

  .refresh-btn:disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .refresh-btn.spinning :global(.icon) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
