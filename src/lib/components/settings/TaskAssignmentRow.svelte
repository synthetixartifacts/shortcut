<script lang="ts">
  import { t } from '$lib/i18n';
  import { Select, Icon, Input } from '$lib/components/ui/primitives';
  import { SaveIndicator } from '$lib/components/ui/patterns';
  import { CUSTOM_MODEL_SENTINEL, type ModelOption, type ProviderOption } from '$lib/features/providers';
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
    /**
     * Inline discovery error for the selected provider. Populated only on
     * Settings → Providers (MASTER_PLAN D9). Null = no error.
     */
    discoveryError?: string | null;
    /** True when the selected provider is the Local LLM. Gates the Custom… affordance. */
    providerIdIsLocal?: boolean;
    /** Per-assignment vision flag — user-controlled for custom Local model ids. */
    supportsVision?: boolean | null;
    onProviderChange: (value: string) => void;
    onModelChange: (value: string) => void;
    onSupportsVisionChange?: (value: boolean) => void;
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
    discoveryError = null,
    providerIdIsLocal = false,
    supportsVision = null,
    onProviderChange,
    onModelChange,
    onSupportsVisionChange,
    onRefreshModels,
  }: Props = $props();

  const loadingOptions = $derived<ModelOption[]>(
    [{ value: model, label: t('settings.task_model_loading') }],
  );

  /**
   * The row is "in custom mode" for Local when any of:
   *   - the persisted model id is empty (user picked Custom — sentinel is
   *     never persisted, so empty is the round-tripped form)
   *   - the user just picked the Custom sentinel in the dropdown (transient)
   *   - the persisted model id isn't in the server's discovered list (e.g., a
   *     typed id from a previous session, or a discovery list that shrank
   *     after a URL change)
   * The sentinel itself is excluded from the discovered-list check so the
   * "Custom…" option doesn't count as a real match.
   */
  const isCustom = $derived(
    providerIdIsLocal && (
      model === ''
      || model === CUSTOM_MODEL_SENTINEL
      || !modelOptions.some((o) => o.value === model && o.value !== CUSTOM_MODEL_SENTINEL)
    )
  );

  const selectValue = $derived(isCustom ? CUSTOM_MODEL_SENTINEL : model);

  /** Real (non-sentinel) discovered models from the current server. */
  const realModelOptions = $derived(modelOptions.filter((o) => o.value !== CUSTOM_MODEL_SENTINEL));

  /**
   * Non-blocking warning (US-4a): the typed id isn't among the server's
   * discovered models, but we still dispatch it as-is. Hidden when discovery
   * returned nothing (no authoritative list to compare against) or when the
   * id matches a discovered entry.
   */
  const showUnknownModelWarning = $derived(
    isCustom
      && model.trim().length > 0
      && model !== CUSTOM_MODEL_SENTINEL
      && realModelOptions.length > 0
      && !realModelOptions.some((o) => o.value === model)
  );

  function handleVisionToggle(event: Event): void {
    if (!onSupportsVisionChange) return;
    const checked = (event.currentTarget as HTMLInputElement).checked;
    onSupportsVisionChange(checked);
  }
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
          value={selectValue}
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
      {#if isCustom}
        <div class="custom-model-input">
          <Input
            type="text"
            monospace
            placeholder={t('settings.task_model_custom_placeholder')}
            value={model === CUSTOM_MODEL_SENTINEL ? '' : model}
            onchange={onModelChange}
          />
        </div>
        {#if taskKey === 'screen_question' && onSupportsVisionChange}
          <label class="vision-checkbox">
            <input
              type="checkbox"
              checked={supportsVision === true}
              onchange={handleVisionToggle}
            />
            {t('settings.task_model_supports_vision')}
          </label>
        {/if}
        {#if showUnknownModelWarning}
          <p class="custom-model-warning">{t('settings.task_model_custom_warning')}</p>
        {/if}
      {/if}
      {#if discoveryError}
        <p class="discovery-error" role="alert">{discoveryError}</p>
      {/if}
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

  .custom-model-input {
    margin-top: var(--spacing-xs);
  }

  .vision-checkbox {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-top: var(--spacing-xs);
    font-size: 0.85rem;
    color: var(--color-text);
    cursor: pointer;
  }

  .custom-model-warning {
    margin: var(--spacing-xs) 0 0;
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .discovery-error {
    margin: var(--spacing-xs) 0 0;
    font-size: 0.75rem;
    color: var(--color-danger);
  }
</style>
