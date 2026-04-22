<script lang="ts">
  /**
   * Translate Settings - Configure the Translation system prompt, user prompt,
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
    translateConfigState,
    loadTranslateConfig,
    saveTranslatePrompt,
    resetTranslatePrompt,
    saveTranslateSystemPrompt,
    resetTranslateSystemPrompt,
  } from '$lib/state/translate-config.svelte';
  import { getDefaultTranslateConfig } from '$lib/api/tauri';
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
      loadTranslateConfig(),
      loadProvidersSettings(),
      getDefaultTranslateConfig(),
    ]);
    defaultPrompt = defaults.prompt;
    defaultSystemPrompt = defaults.system_prompt;
  });

  let promptTimeout: ReturnType<typeof setTimeout> | null = null;
  let systemPromptTimeout: ReturnType<typeof setTimeout> | null = null;

  function onPromptInput(value: string): void {
    translateConfigState.prompt = value;
    if (promptTimeout) clearTimeout(promptTimeout);
    promptTimeout = setTimeout(() => { saveTranslatePrompt(value); }, SAVE_DEBOUNCE_MS);
  }

  function onSystemPromptInput(value: string): void {
    translateConfigState.system_prompt = value;
    if (systemPromptTimeout) clearTimeout(systemPromptTimeout);
    systemPromptTimeout = setTimeout(() => { saveTranslateSystemPrompt(value); }, SAVE_DEBOUNCE_MS);
  }

  async function onPromptReset(): Promise<void> {
    if (promptTimeout) clearTimeout(promptTimeout);
    await resetTranslatePrompt();
  }

  async function onSystemPromptReset(): Promise<void> {
    if (systemPromptTimeout) clearTimeout(systemPromptTimeout);
    await resetTranslateSystemPrompt();
  }

  const assignment = $derived(providersSettingsState.config.task_assignments.translate);
  const providerOptions = $derived(
    getConfiguredProviderOptions(
      'translate',
      providersSettingsState.config.credentials,
      providersSettingsState.models,
    ),
  );
  const modelOptions = $derived(getTaskModelOptions('translate', assignment.provider_id));

  function onProviderChange(value: string): void {
    handleTaskProviderChange('translate', value);
  }
  function onModelChange(value: string): void {
    handleTaskModelChange('translate', value);
  }
  function onRefreshModels(): void {
    void refreshProviderModels(assignment.provider_id as ProviderId);
  }
  const isRefreshing = $derived(
    !!providersSettingsState.refreshingModels[assignment.provider_id as ProviderId],
  );
</script>

<div class="page-translate">
  <PageHeader
    title={t('translate_settings.title')}
    subtitle={t('translate_settings.subtitle')}
    backHref="/settings"
    backLabel={t('translate_settings.back_label')}
  />

  {#if translateConfigState.isLoading}
    <p class="loading">{t('common.loading')}</p>
  {:else if translateConfigState.error}
    <ErrorBanner message={translateConfigState.error} />
  {:else}
    <ShortcutSection actionKey="translate" translationNamespace="translate_settings" />

    <SettingsSection title={t('translate_settings.section_system')}>
      <p class="section-desc">{t('translate_settings.system_prompt_description')}</p>
      <PromptEditor
        label={t('translate_settings.field_system_prompt')}
        hint={t('translate_settings.field_system_prompt_hint')}
        value={translateConfigState.system_prompt}
        defaultValue={defaultSystemPrompt}
        onInput={onSystemPromptInput}
        onReset={onSystemPromptReset}
        resetLabel={t('translate_settings.button_reset')}
        saveStatus={translateConfigState.saveStatus.system_prompt}
      />
    </SettingsSection>

    <SettingsSection title={t('translate_settings.section_prompt')}>
      <p class="section-desc">{t('translate_settings.prompt_description')}</p>
      <PromptEditor
        label={t('translate_settings.field_prompt')}
        hint={t('translate_settings.field_prompt_hint')}
        value={translateConfigState.prompt}
        defaultValue={defaultPrompt}
        onInput={onPromptInput}
        onReset={onPromptReset}
        resetLabel={t('translate_settings.button_reset')}
        saveStatus={translateConfigState.saveStatus.prompt}
        isModified={defaultPrompt !== '' && translateConfigState.prompt !== defaultPrompt}
      />
    </SettingsSection>

    <SettingsSection title={t('translate_settings.section_provider')}>
      <p class="section-desc">{t('translate_settings.provider_description')}</p>
      {#if providerOptions.length === 0}
        <div class="empty-state">
          <p class="empty-title">{t('actions.no_providers_title')}</p>
          <p class="empty-desc">{t('actions.no_providers_desc')}</p>
          <a class="empty-link" href="/settings/providers">{t('actions.configure_providers_link')}</a>
        </div>
      {:else}
        <TaskAssignmentRow
          taskKey="translate"
          label={t('translate_settings.task_label')}
          providerId={assignment.provider_id}
          model={assignment.model}
          saveStatus={providersSettingsState.saveStatus['task.translate']}
          {providerOptions}
          {modelOptions}
          {isRefreshing}
          {onProviderChange}
          {onModelChange}
          {onRefreshModels}
        />
      {/if}
    </SettingsSection>

    <SettingsSection title={t('translate_settings.section_info')}>
      <p class="section-desc">{t('translate_settings.info_description')}</p>
      <div class="info-items">
        <p class="info-item">{t('translate_settings.info_placeholder')}</p>
        <p class="info-item">{t('translate_settings.info_provider')}</p>
      </div>
    </SettingsSection>
  {/if}
</div>

<style>
  .page-translate {
    max-width: var(--form-max-width);
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
