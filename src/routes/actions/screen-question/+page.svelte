<script lang="ts">
  /**
   * Screen Question Settings - Configure the Screen Question system prompt
   * and provider/model assignment (vision-capable models only).
   */
  import { onMount } from 'svelte';
  import { PageHeader } from '$lib/components/ui';
  import { SettingsSection, TaskAssignmentRow } from '$lib/components/settings';
  import { ErrorBanner } from '$lib/components/ui/patterns';
  import PromptEditor from '$lib/components/actions/PromptEditor.svelte';
  import ShortcutSection from '$lib/components/actions/ShortcutSection.svelte';
  import { t } from '$lib/i18n';
  import {
    screenQuestionConfigState,
    loadScreenQuestionConfig,
    saveScreenQuestionSystemPrompt,
    resetScreenQuestionSystemPrompt,
  } from '$lib/state/screen-question-config.svelte';
  import { getDefaultScreenQuestionConfig } from '$lib/api/tauri';
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

  let defaultSystemPrompt = $state('');

  onMount(async () => {
    const [, , defaults] = await Promise.all([
      loadScreenQuestionConfig(),
      loadProvidersSettings(),
      getDefaultScreenQuestionConfig(),
    ]);
    defaultSystemPrompt = defaults.system_prompt;
  });

  let systemPromptTimeout: ReturnType<typeof setTimeout> | null = null;

  function onSystemPromptInput(value: string): void {
    screenQuestionConfigState.system_prompt = value;
    if (systemPromptTimeout) clearTimeout(systemPromptTimeout);
    systemPromptTimeout = setTimeout(() => { saveScreenQuestionSystemPrompt(value); }, SAVE_DEBOUNCE_MS);
  }

  async function onSystemPromptReset(): Promise<void> {
    if (systemPromptTimeout) clearTimeout(systemPromptTimeout);
    await resetScreenQuestionSystemPrompt();
  }

  const assignment = $derived(providersSettingsState.config.task_assignments.screen_question);
  const providerOptions = $derived(
    getConfiguredProviderOptions(
      'screen_question',
      providersSettingsState.config.credentials,
      providersSettingsState.models,
    ),
  );
  const modelOptions = $derived(
    getTaskModelOptions('screen_question', assignment.provider_id),
  );

  function onProviderChange(value: string): void {
    handleTaskProviderChange('screen_question', value);
  }
  function onModelChange(value: string): void {
    handleTaskModelChange('screen_question', value);
  }
  function onRefreshModels(): void {
    void refreshProviderModels(assignment.provider_id as ProviderId);
  }
  const isRefreshing = $derived(
    !!providersSettingsState.refreshingModels[assignment.provider_id as ProviderId],
  );
</script>

<div class="page-screen-question">
  <PageHeader
    title={t('screen_question_settings.title')}
    subtitle={t('screen_question_settings.subtitle')}
    backHref="/settings"
    backLabel={t('screen_question_settings.back_label')}
  />

  {#if screenQuestionConfigState.isLoading}
    <p class="loading">{t('common.loading')}</p>
  {:else if screenQuestionConfigState.error}
    <ErrorBanner message={screenQuestionConfigState.error} />
  {:else}
    <ShortcutSection actionKey="screen_question" translationNamespace="screen_question_settings" />

    <SettingsSection title={t('screen_question_settings.section_system')}>
      <p class="section-desc">{t('screen_question_settings.system_prompt_description')}</p>
      <PromptEditor
        label={t('screen_question_settings.field_system_prompt')}
        hint={t('screen_question_settings.field_system_prompt_hint')}
        value={screenQuestionConfigState.system_prompt}
        defaultValue={defaultSystemPrompt}
        onInput={onSystemPromptInput}
        onReset={onSystemPromptReset}
        resetLabel={t('screen_question_settings.button_reset')}
        saveStatus={screenQuestionConfigState.saveStatus.system_prompt}
      />
    </SettingsSection>

    <SettingsSection title={t('screen_question_settings.section_provider')}>
      <p class="section-desc">{t('screen_question_settings.provider_description')}</p>
      {#if providerOptions.length === 0}
        <div class="empty-state">
          <p class="empty-title">{t('actions.no_providers_title')}</p>
          <p class="empty-desc">{t('actions.no_providers_desc')}</p>
          <a class="empty-link" href="/settings/providers">{t('actions.configure_providers_link')}</a>
        </div>
      {:else}
        <TaskAssignmentRow
          taskKey="screen_question"
          label={t('screen_question_settings.task_label')}
          hint={t('screen_question_settings.task_hint')}
          providerId={assignment.provider_id}
          model={assignment.model}
          saveStatus={providersSettingsState.saveStatus['task.screen_question']}
          {providerOptions}
          {modelOptions}
          {isRefreshing}
          {onProviderChange}
          {onModelChange}
          {onRefreshModels}
        />
      {/if}
    </SettingsSection>

    <SettingsSection title={t('screen_question_settings.section_info')}>
      <p class="section-desc">{t('screen_question_settings.info_description')}</p>
      <div class="info-items">
        <p class="info-item">{t('screen_question_settings.info_vision')}</p>
        <p class="info-item">{t('screen_question_settings.info_provider')}</p>
      </div>
    </SettingsSection>
  {/if}
</div>

<style>
  .page-screen-question {
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
