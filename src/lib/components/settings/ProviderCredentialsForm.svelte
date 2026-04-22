<script lang="ts">
  /**
   * ProviderCredentialsForm — API key/URL inputs for the credential slots.
   *
   * PHASE 5 livesave: explicit Save button removed. Each input mutates the
   * in-memory `config` immediately (UI responsiveness) and, after a
   * `SAVE_DEBOUNCE_MS` window, persists via `saveProviderCredential`. Each
   * field's `<SaveIndicator>` renders inline through the FormField saveStatus
   * prop.
   *
   * PHASE 3 local-llm: adds Protocol dropdown (Auto / Ollama / OpenAI-compat),
   * optional API-key input (only when protocol resolves to openai_compatible),
   * and a "Detected: X" badge visible when protocol is Auto and detection ran.
   * On unmount, any pending debounce timer is flushed.
   */
  import ApiKeyInput from './ApiKeyInput.svelte';
  import TextInput from './TextInput.svelte';
  import SettingsSection from './SettingsSection.svelte';
  import { FormField } from '$lib/components/ui/patterns';
  import { Button, Select } from '$lib/components/ui/primitives';
  import {
    providersSettingsState,
    redetectLocalProtocol,
    saveProviderCredential,
    type CredentialProviderId,
  } from '$lib/state/providers-settings.svelte';
  import { SAVE_DEBOUNCE_MS } from '$lib/utils/save-status.svelte';
  import { t } from '$lib/i18n';

  let redetecting = $state(false);

  async function handleRedetect() {
    if (redetecting) return;
    redetecting = true;
    try {
      await redetectLocalProtocol();
    } finally {
      redetecting = false;
    }
  }

  const timers = new Map<string, ReturnType<typeof setTimeout>>();

  function scheduleCredentialSave(
    providerId: CredentialProviderId,
    field: 'apiKey' | 'baseUrl' | 'protocol',
  ) {
    return (value: string) => {
      // Update in-memory config immediately for UI responsiveness — the saver
      // will re-sync/normalize it on commit. Local is nested; cloud keys are
      // flat `<provider>_api_key` strings.
      const creds = providersSettingsState.config.credentials;
      if (providerId === 'local' && field === 'baseUrl') {
        creds.local.base_url = value;
        creds.local.detected_protocol = null;
      } else if (providerId === 'local' && field === 'protocol') {
        creds.local.protocol = value as typeof creds.local.protocol;
        creds.local.detected_protocol = null;
      } else if (providerId === 'local' && field === 'apiKey') {
        creds.local.api_key = value || null;
      } else if (providerId === 'openai' && field === 'apiKey') {
        creds.openai_api_key = value;
      } else if (providerId === 'anthropic' && field === 'apiKey') {
        creds.anthropic_api_key = value;
      } else if (providerId === 'gemini' && field === 'apiKey') {
        creds.gemini_api_key = value;
      } else if (providerId === 'grok' && field === 'apiKey') {
        creds.grok_api_key = value;
      } else if (providerId === 'soniox' && field === 'apiKey') {
        creds.soniox_api_key = value;
      }

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

  const protocolOptions = $derived([
    { value: 'auto', label: t('settings.field_local_protocol_auto') },
    { value: 'ollama', label: t('settings.field_local_protocol_ollama') },
    { value: 'openai_compatible', label: t('settings.field_local_protocol_openai_compatible') },
  ]);

  // Show the API-key input whenever the resolved protocol *could* be
  // openai_compatible — i.e. the user picked it explicitly, or they picked
  // Auto (which may land on openai_compatible after detection). Letting the
  // user volunteer a key proactively avoids the UX dead-end where detection
  // needs a key to succeed but the field is hidden until detection succeeds.
  const showApiKey = $derived.by(() => {
    const local = providersSettingsState.config.credentials.local;
    return local.protocol === 'openai_compatible' || local.protocol === 'auto';
  });

  // Show the "Detected: X" badge only when Auto is selected and detection ran.
  const detectedLabel = $derived.by(() => {
    const local = providersSettingsState.config.credentials.local;
    if (local.protocol !== 'auto' || !local.detected_protocol) return null;
    return local.detected_protocol === 'openai_compatible'
      ? t('settings.field_local_protocol_openai_compatible')
      : t('settings.field_local_protocol_ollama');
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
    value={providersSettingsState.config.credentials.local.base_url}
    placeholder={t('settings.field_local_url_placeholder')}
    saveStatus={providersSettingsState.saveStatus['local.baseUrl']}
    onchange={scheduleCredentialSave('local', 'baseUrl')}
  />

  <FormField
    label={t('settings.field_local_protocol')}
    id="local-protocol"
    saveStatus={providersSettingsState.saveStatus['local.protocol']}
  >
    <Select
      id="local-protocol"
      value={providersSettingsState.config.credentials.local.protocol}
      options={protocolOptions}
      onchange={scheduleCredentialSave('local', 'protocol')}
    />
  </FormField>
  {#if providersSettingsState.config.credentials.local.protocol === 'auto'}
    <div class="detected-row">
      {#if detectedLabel}
        <p class="detected-badge">
          {t('settings.field_local_detected_prefix')} {detectedLabel}
        </p>
      {/if}
      <Button variant="ghost" onclick={handleRedetect} disabled={redetecting}>
        {t('settings.local_redetect')}
      </Button>
    </div>
  {/if}

  {#if showApiKey}
    <ApiKeyInput
      label={t('settings.field_local_api_key')}
      value={providersSettingsState.config.credentials.local.api_key ?? ''}
      placeholder={t('settings.field_local_api_key_placeholder')}
      saveStatus={providersSettingsState.saveStatus['local.apiKey']}
      onchange={scheduleCredentialSave('local', 'apiKey')}
    />
  {/if}

  {#if providersSettingsState.discoveryErrors.local}
    <p class="discovery-error" role="alert">{providersSettingsState.discoveryErrors.local}</p>
  {/if}
</SettingsSection>

<style>
  .section-desc {
    margin: 0 0 var(--spacing-md);
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }

  .discovery-error {
    /* Surfaces the Local URL discovery failure inline — MASTER_PLAN D9: loud
       here on Settings → Providers only; quiet on Dashboard / Onboarding. */
    margin: calc(var(--spacing-xs) * -1) 0 var(--spacing-md);
    font-size: 0.75rem;
    color: var(--color-danger);
  }

  .detected-badge {
    margin: 0;
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .detected-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    margin: calc(var(--spacing-xs) * -1) 0 var(--spacing-md);
  }
</style>
