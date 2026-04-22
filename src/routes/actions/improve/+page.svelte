<script lang="ts">
  /**
   * Improve Settings - Configure the Improve system prompt, user prompt,
   * and provider/model assignment.
   */
  import { onMount } from 'svelte';
  import { PageHeader } from '$lib/components/ui';
  import { SettingsSection, TaskAssignmentRow } from '$lib/components/settings';
  import { ErrorBanner } from '$lib/components/ui/patterns';
  import PromptEditor from '$lib/components/actions/PromptEditor.svelte';
  import ShortcutSection from '$lib/components/actions/ShortcutSection.svelte';
  import { t } from '$lib/i18n';
  import {
    improveConfigState,
    loadImproveConfig,
    saveImprovePrompt,
    resetImprovePrompt,
    saveImproveSystemPrompt,
    resetImproveSystemPrompt,
  } from '$lib/state/improve-config.svelte';
  import { getDefaultImproveConfig } from '$lib/api/tauri';
  import {
    providersSettingsState,
    loadProvidersSettings,
    getTaskModelOptions,
    handleTaskProviderChange,
    handleTaskModelChange,
    refreshProviderModels,
  } from '$lib/state/providers-settings.svelte';
  import { getConfiguredProviderOptions, type ProviderId } from '$lib/features/providers';
  import { SAVE_DEBOUNCE_MS } from '$lib/utils/save-status.svelte';

  let defaultPrompt = $state('');
  let defaultSystemPrompt = $state('');

  onMount(async () => {
    const [, , defaults] = await Promise.all([
      loadImproveConfig(),
      loadProvidersSettings(),
      getDefaultImproveConfig(),
    ]);
    defaultPrompt = defaults.prompt;
    defaultSystemPrompt = defaults.system_prompt;
  });

  let promptTimeout: ReturnType<typeof setTimeout> | null = null;
  let systemPromptTimeout: ReturnType<typeof setTimeout> | null = null;

  function onPromptInput(value: string): void {
    improveConfigState.prompt = value;
    if (promptTimeout) clearTimeout(promptTimeout);
    promptTimeout = setTimeout(() => { saveImprovePrompt(value); }, SAVE_DEBOUNCE_MS);
  }

  function onSystemPromptInput(value: string): void {
    improveConfigState.system_prompt = value;
    if (systemPromptTimeout) clearTimeout(systemPromptTimeout);
    systemPromptTimeout = setTimeout(() => { saveImproveSystemPrompt(value); }, SAVE_DEBOUNCE_MS);
  }

  async function onPromptReset(): Promise<void> {
    if (promptTimeout) clearTimeout(promptTimeout);
    await resetImprovePrompt();
  }

  async function onSystemPromptReset(): Promise<void> {
    if (systemPromptTimeout) clearTimeout(systemPromptTimeout);
    await resetImproveSystemPrompt();
  }

  const assignment = $derived(providersSettingsState.config.task_assignments.improve);
  const providerOptions = $derived(
    getConfiguredProviderOptions(
      'improve',
      providersSettingsState.config.credentials,
      providersSettingsState.models,
    ),
  );
  const modelOptions = $derived(getTaskModelOptions('improve', assignment.provider_id));

  function onProviderChange(value: string): void {
    handleTaskProviderChange('improve', value);
  }
  function onModelChange(value: string): void {
    handleTaskModelChange('improve', value);
  }
  function onRefreshModels(): void {
    void refreshProviderModels(assignment.provider_id as ProviderId);
  }
  const isRefreshing = $derived(
    !!providersSettingsState.refreshingModels[assignment.provider_id as ProviderId],
  );
</script>

<div class="page-improve">
  <PageHeader
    title={t('improve_settings.title')}
    subtitle={t('improve_settings.subtitle')}
    backHref="/settings"
    backLabel={t('improve_settings.back_label')}
  />

  {#if improveConfigState.isLoading}
    <p class="loading">{t('common.loading')}</p>
  {:else if improveConfigState.error}
    <ErrorBanner message={improveConfigState.error} />
  {:else}
    <ShortcutSection actionKey="improve" translationNamespace="improve_settings" />

    <SettingsSection title={t('improve_settings.section_system')}>
      <p class="section-desc">{t('improve_settings.system_prompt_description')}</p>
      <PromptEditor
        label={t('improve_settings.field_system_prompt')}
        hint={t('improve_settings.field_system_prompt_hint')}
        value={improveConfigState.system_prompt}
        defaultValue={defaultSystemPrompt}
        onInput={onSystemPromptInput}
        onReset={onSystemPromptReset}
        resetLabel={t('improve_settings.button_reset')}
        saveStatus={improveConfigState.saveStatus.system_prompt}
      />
    </SettingsSection>

    <SettingsSection title={t('improve_settings.section_prompt')}>
      <p class="section-desc">{t('improve_settings.prompt_description')}</p>
      <PromptEditor
        label={t('improve_settings.field_prompt')}
        hint={t('improve_settings.field_prompt_hint')}
        value={improveConfigState.prompt}
        defaultValue={defaultPrompt}
        onInput={onPromptInput}
        onReset={onPromptReset}
        resetLabel={t('improve_settings.button_reset')}
        saveStatus={improveConfigState.saveStatus.prompt}
        isModified={defaultPrompt !== '' && improveConfigState.prompt !== defaultPrompt}
      />
    </SettingsSection>

    <SettingsSection title={t('improve_settings.section_provider')}>
      <p class="section-desc">{t('improve_settings.provider_description')}</p>
      {#if providerOptions.length === 0}
        <div class="empty-state">
          <p class="empty-title">{t('actions.no_providers_title')}</p>
          <p class="empty-desc">{t('actions.no_providers_desc')}</p>
          <a class="empty-link" href="/settings/providers">{t('actions.configure_providers_link')}</a>
        </div>
      {:else}
        <TaskAssignmentRow
          taskKey="improve"
          label={t('improve_settings.task_label')}
          providerId={assignment.provider_id}
          model={assignment.model}
          saveStatus={providersSettingsState.saveStatus['task.improve']}
          {providerOptions}
          {modelOptions}
          {isRefreshing}
          {onProviderChange}
          {onModelChange}
          {onRefreshModels}
        />
      {/if}
    </SettingsSection>

    <SettingsSection title={t('improve_settings.section_info')}>
      <p class="section-desc">{t('improve_settings.info_description')}</p>
      <div class="info-items">
        <p class="info-item">{t('improve_settings.info_placeholder')}</p>
        <p class="info-item">{t('improve_settings.info_provider')}</p>
      </div>
    </SettingsSection>
  {/if}
</div>

<style>
  .page-improve {
    max-width: var(--page-max-width);
  }
  .loading {
    color: var(--color-text-muted);
    font-size: 0.9rem;
  }
  .section-desc {
    margin: 0 0 var(--spacing-md);
    font-size: 0.9rem;
    color: var(--color-text-muted);
  }
  .info-items {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }
  .info-item {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text-muted);
    padding-left: var(--spacing-md);
    border-left: 2px solid var(--color-kbd-border);
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
    margin: 0 0 var(--spacing-sm);
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }
  .empty-link {
    font-size: 0.9rem;
    color: var(--color-primary);
    text-decoration: none;
  }
  .empty-link:hover {
    text-decoration: underline;
  }
</style>
