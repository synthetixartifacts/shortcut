<script lang="ts">
  /**
   * ProviderCredentialsForm — API key/URL inputs for the 6 credential slots.
   *
   * PHASE 5 (livesave): explicit Save button removed. Each input mutates the
   * in-memory `config` immediately (UI responsiveness) and, after a
   * `SAVE_DEBOUNCE_MS` window, persists via `saveProviderCredential`. Each
   * field's `<SaveIndicator>` renders inline through the FormField saveStatus
   * prop. On unmount, any pending debounce timer is flushed.
   */
  import ApiKeyInput from './ApiKeyInput.svelte';
  import TextInput from './TextInput.svelte';
  import SettingsSection from './SettingsSection.svelte';
  import {
    providersSettingsState,
    saveProviderCredential,
    type CredentialProviderId,
  } from '$lib/state/providers-settings.svelte';
  import { SAVE_DEBOUNCE_MS } from '$lib/utils/save-status.svelte';
  import { t } from '$lib/i18n';

  const timers = new Map<string, ReturnType<typeof setTimeout>>();

  function scheduleCredentialSave(providerId: CredentialProviderId, field: 'apiKey' | 'baseUrl') {
    return (value: string) => {
      // Update in-memory config immediately for UI responsiveness — the saver
      // will re-sync/normalize it on commit.
      const fieldName = field === 'baseUrl'
        ? 'ollama_base_url'
        : `${providerId}_api_key` as keyof typeof providersSettingsState.config.credentials;
      providersSettingsState.config.credentials[fieldName] = value;

      const key = `${providerId}.${field}`;
      const existing = timers.get(key);
      if (existing) clearTimeout(existing);
      timers.set(key, setTimeout(() => {
        timers.delete(key);
        void saveProviderCredential(providerId, field, value);
      }, SAVE_DEBOUNCE_MS));
    };
  }

  $effect(() => () => {
    for (const timer of timers.values()) clearTimeout(timer);
    timers.clear();
  });
</script>

<SettingsSection title={t('settings.section_providers')}>
  <p class="section-desc">{t('settings.providers_desc')}</p>

  <ApiKeyInput
    label={t('settings.field_openai_key')}
    value={providersSettingsState.config.credentials.openai_api_key}
    placeholder={t('settings.field_openai_key_placeholder')}
    saveStatus={providersSettingsState.saveStatus['openai.apiKey']}
    onchange={scheduleCredentialSave('openai', 'apiKey')}
  />
  <ApiKeyInput
    label={t('settings.field_anthropic_key')}
    value={providersSettingsState.config.credentials.anthropic_api_key}
    placeholder={t('settings.field_anthropic_key_placeholder')}
    saveStatus={providersSettingsState.saveStatus['anthropic.apiKey']}
    onchange={scheduleCredentialSave('anthropic', 'apiKey')}
  />
  <ApiKeyInput
    label={t('settings.field_gemini_key')}
    value={providersSettingsState.config.credentials.gemini_api_key}
    placeholder={t('settings.field_gemini_key_placeholder')}
    saveStatus={providersSettingsState.saveStatus['gemini.apiKey']}
    onchange={scheduleCredentialSave('gemini', 'apiKey')}
  />
  <ApiKeyInput
    label={t('settings.field_grok_key')}
    value={providersSettingsState.config.credentials.grok_api_key}
    placeholder={t('settings.field_grok_key_placeholder')}
    saveStatus={providersSettingsState.saveStatus['grok.apiKey']}
    onchange={scheduleCredentialSave('grok', 'apiKey')}
  />
  <ApiKeyInput
    label={t('settings.field_soniox_key')}
    value={providersSettingsState.config.credentials.soniox_api_key}
    placeholder={t('settings.field_soniox_key_placeholder')}
    saveStatus={providersSettingsState.saveStatus['soniox.apiKey']}
    onchange={scheduleCredentialSave('soniox', 'apiKey')}
  />
  <TextInput
    label={t('settings.field_local_url')}
    value={providersSettingsState.config.credentials.ollama_base_url}
    placeholder={t('settings.field_local_url_placeholder')}
    saveStatus={providersSettingsState.saveStatus['ollama.baseUrl']}
    onchange={scheduleCredentialSave('ollama', 'baseUrl')}
  />
</SettingsSection>

<style>
  .section-desc {
    margin: 0 0 var(--spacing-md);
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }
</style>
