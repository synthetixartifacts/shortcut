<script lang="ts">
  /**
   * Dictation Settings - Full configuration for voice recognition
   */
  import { onMount } from 'svelte';

  // Components
  import { PageHeader } from '$lib/components/ui';
  import { SettingsSection } from '$lib/components/settings';
  import { TermsList, AudioSettingsPanel, EngineSelector } from '$lib/components/dictation';
  import { LanguageSelector } from '$lib/components/settings';
  import { Input } from '$lib/components/ui/primitives';
  import { FormField, ErrorBanner, SaveIndicator } from '$lib/components/ui/patterns';
  import ShortcutSection from '$lib/components/actions/ShortcutSection.svelte';
  import { t } from '$lib/i18n';

  // State
  import {
    dictationConfigState,
    loadDictationConfig,
    saveDictationConfig,
    addCustomTerm,
    removeCustomTerm,
    type DictationFieldKey,
  } from '$lib/state/dictation-config.svelte';
  import {
    engineState,
    loadEngineState,
    switchEngine,
    completePendingSwitch,
  } from '$lib/state/engine.svelte';

  // Types
  import type { DictationConfig } from '$lib/features/dictation';

  onMount(async () => {
    await Promise.all([loadDictationConfig(), loadEngineState()]);
  });

  // Local form state synced with config
  let formState = $derived(dictationConfigState.config);

  /**
   * Generic save helper for simple fields.
   *
   * Binds a `DictationConfig` key to a matching `DictationFieldKey` so the
   * correct per-field `saveStatus` entry flips through `saving → saved`.
   */
  function saveField<K extends keyof DictationConfig>(
    configKey: K,
    fieldKey: DictationFieldKey,
  ) {
    return (value: DictationConfig[K]) =>
      saveDictationConfig({ [configKey]: value } as Partial<DictationConfig>, fieldKey);
  }

  // Special save function for names (comma-separated string → string[])
  async function saveNames(namesStr: string): Promise<void> {
    const names = namesStr.split(',').map((n) => n.trim()).filter(Boolean);
    await saveDictationConfig({ names }, 'names');
  }

  // Derived values for display
  const namesDisplay = $derived(formState.names.join(', '));

  // Engine handlers — switchEngine returns false if model download is needed,
  // setting pendingEngine which triggers inline download UI in the EngineCard
  async function handleEngineSwitch(engineId: string): Promise<void> {
    await switchEngine(engineId);
  }

  async function handleModelComplete(): Promise<void> {
    // Model download finished — complete the pending engine switch
    if (engineState.pendingEngine) {
      await completePendingSwitch();
    }
    await loadEngineState();
  }

  function handleModelCancel(): void {
    engineState.pendingEngine = null;
  }

  const showSonioxSections = $derived(
    engineState.engines.length > 0 && engineState.activeEngine === 'soniox'
  );
</script>

<div class="page-dictation">
  <PageHeader
    title={t('dictation_settings.title')}
    subtitle={t('dictation_settings.subtitle')}
    backHref="/settings"
    backLabel={t('dictation_settings.back_label')}
  />

  {#if dictationConfigState.isLoading}
    <p class="loading">{t('dictation_settings.loading')}</p>
  {:else if dictationConfigState.error}
    <ErrorBanner message={dictationConfigState.error} />
  {:else}
    <ShortcutSection actionKey="dictation" translationNamespace="dictation_settings" />

    <SettingsSection title={t('dictation_settings.section_engine')}>
      <EngineSelector
        engines={engineState.engines}
        pendingEngine={engineState.pendingEngine}
        onActivate={handleEngineSwitch}
        onDownloadComplete={handleModelComplete}
        onDownloadCancel={handleModelCancel}
        isSwitching={engineState.isSwitching}
      />
    </SettingsSection>

    <SettingsSection title={t('dictation_settings.section_audio')}>
      <AudioSettingsPanel
        microphoneId={formState.selectedMicrophoneId}
        settings={formState.audioSettings}
        onMicrophoneChange={saveField('selectedMicrophoneId', 'microphone')}
        onSettingsChange={saveField('audioSettings', 'audio_settings')}
        microphoneSaveStatus={dictationConfigState.saveStatus.microphone}
        audioSettingsSaveStatus={dictationConfigState.saveStatus.audio_settings}
      />
    </SettingsSection>

    {#if showSonioxSections}
      <SettingsSection title={t('dictation_settings.section_vocab')}>
        <p class="section-desc">
          {t('dictation_settings.vocab_description')}
        </p>
        <TermsList
          terms={formState.customTerms}
          onAdd={addCustomTerm}
          onRemove={removeCustomTerm}
          disabled={dictationConfigState.isSaving}
        />
        <SaveIndicator status={dictationConfigState.saveStatus.custom_terms} />
      </SettingsSection>

      <SettingsSection title={t('dictation_settings.section_context')}>
        <FormField
          label={t('dictation_settings.field_topic')}
          hint={t('dictation_settings.field_topic_hint')}
          saveStatus={dictationConfigState.saveStatus.topic}
        >
          <Input
            type="text"
            value={formState.topic}
            placeholder={t('dictation_settings.field_topic_placeholder')}
            onchange={saveField('topic', 'topic')}
          />
        </FormField>

        <FormField
          label={t('dictation_settings.field_names')}
          hint={t('dictation_settings.field_names_hint')}
          saveStatus={dictationConfigState.saveStatus.names}
        >
          <Input
            type="text"
            value={namesDisplay}
            placeholder={t('dictation_settings.field_names_placeholder')}
            onchange={saveNames}
          />
        </FormField>

        <FormField
          label={t('dictation_settings.field_background')}
          hint={t('dictation_settings.field_background_hint')}
          saveStatus={dictationConfigState.saveStatus.background_text}
        >
          <textarea
            class="textarea"
            value={formState.backgroundText}
            placeholder={t('dictation_settings.field_background_placeholder')}
            oninput={(e) => saveField('backgroundText', 'background_text')((e.target as HTMLTextAreaElement).value)}
          ></textarea>
        </FormField>
      </SettingsSection>

      <SettingsSection title={t('dictation_settings.section_languages')}>
        <p class="section-desc">
          {t('dictation_settings.languages_description')}
        </p>
        <LanguageSelector
          selected={formState.languageHints}
          onchange={saveField('languageHints', 'language_hints')}
        />
        <SaveIndicator status={dictationConfigState.saveStatus.language_hints} />
        <label class="checkbox-option">
          <input
            type="checkbox"
            checked={formState.enableLanguageIdentification}
            onchange={(e) => saveField('enableLanguageIdentification', 'language_identification')((e.target as HTMLInputElement).checked)}
          />
          <span>{t('dictation_settings.auto_detect')}</span>
        </label>
        <SaveIndicator status={dictationConfigState.saveStatus.language_identification} />
      </SettingsSection>
    {/if}
  {/if}
</div>

<style>
  .page-dictation {
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

  .textarea {
    width: 100%;
    min-height: 80px;
    padding: var(--spacing-sm) var(--spacing-md);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-sm);
    background: var(--color-surface);
    color: var(--color-text);
    font-family: inherit;
    font-size: 0.9rem;
    resize: vertical;
  }

  .textarea:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary-light);
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    cursor: pointer;
    margin-top: var(--spacing-sm);
  }

  .checkbox-option input {
    margin: 0;
  }

  .checkbox-option span {
    font-size: 0.9rem;
  }
</style>
