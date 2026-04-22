<script lang="ts">
  /**
   * Grammar Settings - Configure the Grammar Fix system prompt, user prompt,
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
    grammarConfigState,
    loadGrammarConfig,
    saveGrammarPrompt,
    resetGrammarPrompt,
    saveGrammarSystemPrompt,
    resetGrammarSystemPrompt,
  } from '$lib/state/grammar-config.svelte';
  import { getDefaultGrammarConfig } from '$lib/api/tauri';
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
      loadGrammarConfig(),
      loadProvidersSettings(),
      getDefaultGrammarConfig(),
    ]);
    defaultPrompt = defaults.prompt;
    defaultSystemPrompt = defaults.system_prompt;
  });

  let promptTimeout: ReturnType<typeof setTimeout> | null = null;
  let systemPromptTimeout: ReturnType<typeof setTimeout> | null = null;

  function onPromptInput(value: string): void {
    grammarConfigState.prompt = value;
    if (promptTimeout) clearTimeout(promptTimeout);
    promptTimeout = setTimeout(() => { saveGrammarPrompt(value); }, SAVE_DEBOUNCE_MS);
  }

  function onSystemPromptInput(value: string): void {
    grammarConfigState.system_prompt = value;
    if (systemPromptTimeout) clearTimeout(systemPromptTimeout);
    systemPromptTimeout = setTimeout(() => { saveGrammarSystemPrompt(value); }, SAVE_DEBOUNCE_MS);
  }

  async function onPromptReset(): Promise<void> {
    if (promptTimeout) clearTimeout(promptTimeout);
    await resetGrammarPrompt();
  }

  async function onSystemPromptReset(): Promise<void> {
    if (systemPromptTimeout) clearTimeout(systemPromptTimeout);
    await resetGrammarSystemPrompt();
  }

  const assignment = $derived(providersSettingsState.config.task_assignments.grammar);
  const providerOptions = $derived(
    getConfiguredProviderOptions(
      'grammar',
      providersSettingsState.config.credentials,
      providersSettingsState.models,
    ),
  );
  const modelOptions = $derived(getTaskModelOptions('grammar', assignment.provider_id));

  function onProviderChange(value: string): void {
    handleTaskProviderChange('grammar', value);
  }
  function onModelChange(value: string): void {
    handleTaskModelChange('grammar', value);
  }
  function onRefreshModels(): void {
    void refreshProviderModels(assignment.provider_id as ProviderId);
  }
  const isRefreshing = $derived(
    !!providersSettingsState.refreshingModels[assignment.provider_id as ProviderId],
  );
</script>

<div class="page-grammar">
  <PageHeader
    title={t('grammar_settings.title')}
    subtitle={t('grammar_settings.subtitle')}
    backHref="/settings"
    backLabel={t('grammar_settings.back_label')}
  />

  {#if grammarConfigState.isLoading}
    <p class="loading">{t('common.loading')}</p>
  {:else if grammarConfigState.error}
    <ErrorBanner message={grammarConfigState.error} />
  {:else}
    <ShortcutSection actionKey="grammar" translationNamespace="grammar_settings" />

    <SettingsSection title={t('grammar_settings.section_system')}>
      <p class="section-desc">{t('grammar_settings.system_prompt_description')}</p>
      <PromptEditor
        label={t('grammar_settings.field_system_prompt')}
        hint={t('grammar_settings.field_system_prompt_hint')}
        value={grammarConfigState.system_prompt}
        defaultValue={defaultSystemPrompt}
        onInput={onSystemPromptInput}
        onReset={onSystemPromptReset}
        resetLabel={t('grammar_settings.button_reset')}
        saveStatus={grammarConfigState.saveStatus.system_prompt}
      />
    </SettingsSection>

    <SettingsSection title={t('grammar_settings.section_prompt')}>
      <p class="section-desc">{t('grammar_settings.prompt_description')}</p>
      <PromptEditor
        label={t('grammar_settings.field_prompt')}
        hint={t('grammar_settings.field_prompt_hint')}
        value={grammarConfigState.prompt}
        defaultValue={defaultPrompt}
        onInput={onPromptInput}
        onReset={onPromptReset}
        resetLabel={t('grammar_settings.button_reset')}
        saveStatus={grammarConfigState.saveStatus.prompt}
        isModified={defaultPrompt !== '' && grammarConfigState.prompt !== defaultPrompt}
      />
    </SettingsSection>

    <SettingsSection title={t('grammar_settings.section_provider')}>
      <p class="section-desc">{t('grammar_settings.provider_description')}</p>
      {#if providerOptions.length === 0}
        <div class="empty-state">
          <p class="empty-title">{t('actions.no_providers_title')}</p>
          <p class="empty-desc">{t('actions.no_providers_desc')}</p>
          <a class="empty-link" href="/settings/providers">{t('actions.configure_providers_link')}</a>
        </div>
      {:else}
        <TaskAssignmentRow
          taskKey="grammar"
          label={t('grammar_settings.task_label')}
          providerId={assignment.provider_id}
          model={assignment.model}
          saveStatus={providersSettingsState.saveStatus['task.grammar']}
          {providerOptions}
          {modelOptions}
          {isRefreshing}
          {onProviderChange}
          {onModelChange}
          {onRefreshModels}
        />
      {/if}
    </SettingsSection>

    <SettingsSection title={t('grammar_settings.section_info')}>
      <p class="section-desc">{t('grammar_settings.info_description')}</p>
      <div class="info-items">
        <p class="info-item">{t('grammar_settings.info_placeholder')}</p>
        <p class="info-item">{t('grammar_settings.info_provider')}</p>
      </div>
    </SettingsSection>
  {/if}
</div>

<style>
  .page-grammar {
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
