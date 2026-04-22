<script lang="ts">
  /**
   * App Settings Page — UI preferences
   *
   * Debug toggle, theme, and language settings.
   */
  import { onMount } from 'svelte';
  import { loadSettings, settingsState } from '$lib/state/settings.svelte';
  import { appSettingsState, syncAppSettings, saveAppSettings } from '$lib/state/app-settings.svelte';
  import { engineState, loadEngineState } from '$lib/state/engine.svelte';
  import { PageHeader } from '$lib/components/ui/patterns';
  import { SettingsSection } from '$lib/components/settings';
  import { ErrorBanner, SaveIndicator } from '$lib/components/ui/patterns';
  import { t } from '$lib/i18n';

  onMount(async () => {
    await loadSettings();
    syncAppSettings();
    await loadEngineState();
  });

  async function handleThemeChange(theme: string): Promise<void> {
    appSettingsState.theme = theme;
    try {
      await saveAppSettings('theme');
    } catch {
      // Error displayed via appSettingsState.error
    }
  }

  async function handleLanguageChange(lang: string): Promise<void> {
    appSettingsState.language = lang;
    try {
      await saveAppSettings('language');
    } catch {
      // Error displayed via appSettingsState.error
    }
  }

  async function handleToggleDebug(): Promise<void> {
    appSettingsState.debugEnabled = !appSettingsState.debugEnabled;
    try {
      await saveAppSettings('debug');
    } catch {
      // Error displayed via appSettingsState.error
    }
  }
</script>

<div class="page-app-settings">
  <PageHeader
    title={t('app_settings.title')}
    subtitle={t('app_settings.subtitle')}
    backHref="/settings"
    backLabel={t('settings_hub.back_label')}
  />

  {#if settingsState.isLoading}
    <p class="loading">{t('common.loading')}</p>
  {:else if appSettingsState.error}
    <ErrorBanner message={appSettingsState.error} />
  {:else}
    <SettingsSection title={t('app_settings.section_appearance')}>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{t('app_settings.theme_label')}</span>
          <span class="setting-description">{t('app_settings.theme_description')}</span>
        </div>
        <div class="setting-control">
          <div class="theme-selector">
            <button
              class="theme-option"
              class:active={appSettingsState.theme === 'light'}
              onclick={() => handleThemeChange('light')}
              disabled={appSettingsState.isSaving}
            >
              {t('app_settings.theme_light')}
            </button>
            <button
              class="theme-option"
              class:active={appSettingsState.theme === 'dark'}
              onclick={() => handleThemeChange('dark')}
              disabled={appSettingsState.isSaving}
            >
              {t('app_settings.theme_dark')}
            </button>
          </div>
          <SaveIndicator status={appSettingsState.saveStatus.theme} />
        </div>
      </div>
    </SettingsSection>

    <SettingsSection title={t('app_settings.section_language')}>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{t('app_settings.language_label')}</span>
          <span class="setting-description">{t('app_settings.language_description')}</span>
        </div>
        <div class="setting-control">
          <select
            class="language-select"
            value={appSettingsState.language}
            onchange={(e) => handleLanguageChange(e.currentTarget.value)}
            disabled={appSettingsState.isSaving}
          >
            <option value="en">{t('app_settings.language.english')}</option>
            <option value="fr">{t('app_settings.language.french')}</option>
            <option value="es">{t('app_settings.language.spanish')}</option>
          </select>
          <SaveIndicator status={appSettingsState.saveStatus.language} />
        </div>
      </div>
    </SettingsSection>

    <SettingsSection title={t('app_settings.section_developer')}>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{t('app_settings.debug_label')}</span>
          <span class="setting-description">{t('app_settings.debug_description')}</span>
        </div>
        <div class="setting-control">
          <button
            class="toggle-switch"
            class:active={appSettingsState.debugEnabled}
            onclick={handleToggleDebug}
            disabled={appSettingsState.isSaving}
            aria-checked={appSettingsState.debugEnabled}
            aria-label={t('app_settings.debug_label')}
            role="switch"
          >
            <span class="toggle-knob"></span>
          </button>
          <SaveIndicator status={appSettingsState.saveStatus.debug} />
        </div>
      </div>
    </SettingsSection>

    {#if engineState.platform === 'windows'}
      <SettingsSection title={t('app_settings.section_credits')}>
        <p class="credits-text">{t('engine.attribution')}</p>
      </SettingsSection>
    {/if}

    <div class="version-info">
      <span class="version-label">v0.1.0</span>
    </div>
  {/if}
</div>

<style>
  .page-app-settings {
    max-width: var(--page-max-width);
  }

  .loading {
    color: var(--color-text-muted);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) 0;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-label {
    font-weight: 500;
    font-size: 0.9rem;
    color: var(--color-text);
  }

  .setting-description {
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .setting-control {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .theme-selector {
    display: flex;
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-sm);
    overflow: hidden;
  }

  .theme-option {
    padding: var(--spacing-xs) var(--spacing-md);
    border: none;
    background: var(--color-kbd-bg);
    color: var(--color-text);
    font-size: 0.85rem;
    cursor: pointer;
    transition: background 0.2s, color 0.2s;
  }

  .theme-option.active {
    background: var(--color-primary);
    color: var(--color-text-on-primary);
  }

  .theme-option:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .language-select {
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-sm);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.85rem;
    cursor: pointer;
  }

  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    border-radius: 12px;
    border: none;
    background: var(--color-kbd-border);
    cursor: pointer;
    transition: background 0.2s ease;
    flex-shrink: 0;
    padding: 0;
  }

  .toggle-switch.active {
    background: var(--color-primary);
  }

  .toggle-switch:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--color-surface);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
    transition: transform 0.2s ease;
  }

  .toggle-switch.active .toggle-knob {
    transform: translateX(20px);
  }

  .credits-text {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .version-info {
    margin-top: var(--spacing-lg);
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--color-kbd-border);
    text-align: center;
  }

  .version-label {
    font-size: 0.8rem;
    color: var(--color-text-hint);
  }
</style>
