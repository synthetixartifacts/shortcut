<script lang="ts">
  /**
   * Dashboard Page - Quick overview and action launching
   *
   * Shows:
   * - Provider readiness status
   * - Microphone selector (quick device switching)
   * - Action shortcuts
   * - Recent transcriptions (last 3)
   */

  import { onMount } from 'svelte';
  import * as api from '$lib/api/tauri';

  // State
  import { historyState, loadHistory } from '$lib/state/history.svelte';
  import { dictationConfigState, loadDictationConfig, saveDictationConfig } from '$lib/state/dictation-config.svelte';
  import { providerReadiness } from '$lib/state/providers.svelte';

  // Features
  import { getCurrentPlatform } from '$lib/features/shortcuts';

  // Components
  import { PageHeader, ProviderStatusCard } from '$lib/components/ui';
  import { Icon } from '$lib/components/ui/primitives';
  import { SaveIndicator } from '$lib/components/ui/patterns';
  import { MicrophoneSelector } from '$lib/components/dictation';
  import { RecentEntries } from '$lib/components/history';
  import ActionsShortcutGrid from '$lib/components/actions/ActionsShortcutGrid.svelte';
  import { t } from '$lib/i18n';

  let platform = $state<'macos' | 'windows' | 'linux'>('windows');
  let copiedId = $state<string | null>(null);
  let resetStatus = $state<'idle' | 'resetting' | 'done'>('idle');

  async function handleResetDisplay(): Promise<void> {
    resetStatus = 'resetting';
    try {
      await api.resetIndicator();
      resetStatus = 'done';
      setTimeout(() => { resetStatus = 'idle'; }, 2000);
    } catch {
      resetStatus = 'idle';
    }
  }

  onMount(async () => {
    platform = await getCurrentPlatform();
    await loadHistory(1);
    loadDictationConfig();
  });

  const recentEntries = $derived(historyState.entries.slice(0, 3));

  function saveMicrophone(deviceId: string | null): void {
    void saveDictationConfig({ selectedMicrophoneId: deviceId }, 'microphone');
  }

  async function handleCopy(id: string, text: string): Promise<void> {
    await navigator.clipboard.writeText(text);
    copiedId = id;
    setTimeout(() => { copiedId = null; }, 1500);
  }

  const providerSummary = $derived.by(() => {
    if (!providerReadiness.providers_checked) return 'checking' as const;
    if (providerReadiness.any_llm_configured && providerReadiness.stt_configured) return 'ready' as const;
    if (!providerReadiness.any_llm_configured && !providerReadiness.stt_configured) return 'none' as const;
    return 'partial' as const;
  });
</script>

<div class="page-dashboard">
  <PageHeader title={t('dashboard.title')}>
    {#snippet actions()}
      <div class="header-status">
        <ProviderStatusCard summary={providerSummary} />
        <button
          class="reset-display-btn"
          onclick={handleResetDisplay}
          disabled={resetStatus === 'resetting'}
        >
          <span class="reset-icon" class:spinning={resetStatus === 'resetting'}>
            <Icon name="refresh" size={12} />
          </span>
          <span>{resetStatus === 'resetting'
            ? t('dashboard.reset_display_resetting')
            : resetStatus === 'done'
              ? t('dashboard.reset_display_done')
              : t('dashboard.reset_display')}</span>
        </button>
      </div>
    {/snippet}
  </PageHeader>

  {#if providerSummary === 'none'}
    <div class="no-providers-banner">
      <p>{t('dashboard.no_providers')}</p>
      <a href="/settings/providers" class="configure-link">{t('dashboard.configure_providers_link')}</a>
    </div>
  {/if}

  <section class="microphone-section">
    <div class="section-actions">
      <SaveIndicator status={dictationConfigState.saveStatus.microphone} />
      <a href="/actions/dictation" class="mic-settings-link">{t('dashboard.microphone_settings')}</a>
    </div>
    <MicrophoneSelector
      selectedDeviceId={dictationConfigState.config.selectedMicrophoneId}
      onSelect={saveMicrophone}
    />
  </section>

  <section class="shortcuts-section">
    <div class="section-header">
      <h2 class="section-title">{t('dashboard.shortcuts_heading')}</h2>
      <a href="/shortcuts" class="view-all">{t('dashboard.shortcuts_edit')}</a>
    </div>
    <ActionsShortcutGrid {platform} />
  </section>

  {#if recentEntries.length > 0}
    <RecentEntries
      entries={recentEntries}
      {copiedId}
      onCopy={handleCopy}
    />
  {/if}
</div>

<style>
  .page-dashboard {
    max-width: var(--page-max-width);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2xl);
  }

  .header-status {
    display: flex;
    align-items: stretch;
    gap: var(--spacing-sm);
  }

  .no-providers-banner {
    padding: var(--spacing-md) var(--spacing-lg);
    background: color-mix(in srgb, var(--color-danger) 6%, var(--color-card-bg));
    border: 1px solid color-mix(in srgb, var(--color-danger) 30%, var(--color-kbd-border));
    border-radius: var(--border-radius-md);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
  }

  .no-providers-banner p {
    margin: 0;
    font-size: 0.9rem;
    color: var(--color-text);
  }

  .configure-link {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-primary);
    text-decoration: none;
    white-space: nowrap;
  }

  .configure-link:hover,
  .mic-settings-link:hover,
  .view-all:hover { text-decoration: underline; }

  .microphone-section {
    position: relative;
  }

  .microphone-section :global(.microphone-selector) {
    margin-bottom: 0;
  }

  .section-actions {
    position: absolute;
    top: 0;
    right: 0;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .mic-settings-link {
    font-size: 0.8rem;
    color: var(--color-primary);
    text-decoration: none;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-md);
  }

  .section-header :global(.section-title) { margin: 0; }

  .view-all {
    font-size: 0.8rem;
    color: var(--color-primary);
    text-decoration: none;
  }

  .reset-display-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: 4px var(--spacing-sm);
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--color-text);
    background: var(--color-card-bg);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-md);
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .reset-display-btn:hover:not(:disabled) {
    border-color: var(--color-primary);
    background: color-mix(in srgb, var(--color-primary) 5%, var(--color-card-bg));
  }

  .reset-display-btn:disabled { cursor: default; opacity: 0.5; }
  .reset-icon { display: block; color: var(--color-text-muted); }
  .reset-icon.spinning { animation: spin 1s linear infinite; }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

</style>
