<script lang="ts">
  /**
   * Debug Logs Page
   *
   * Displays in-memory logs for debugging issues in production.
   * Allows filtering, copying, and clearing logs.
   * Shows per-provider status below the logs (Phase 6 — replaces service endpoints).
   */
  import { onMount, onDestroy } from 'svelte';
  import { PageHeader } from '$lib/components/ui';
  import { t } from '$lib/i18n';
  import { LogFilter, LogViewer } from '$lib/components/debug';
  import {
    debugState,
    clearLogs,
    getLogsAsText,
    refreshDebugLogs,
    type LogLevel,
  } from '$lib/state/debug.svelte';
  import { getProviderStatus } from '$lib/api/tauri';
  import type { ProviderStatusReport } from '$lib/types';

  // Filter state
  let showInfo = $state(true);
  let showWarn = $state(true);
  let showError = $state(true);
  let autoScroll = $state(true);
  let copySuccess = $state(false);

  // Provider status
  let providerStatus = $state<ProviderStatusReport | null>(null);
  let providerStatusError = $state<string | null>(null);

  // Filtered logs based on selected levels
  const filteredLogs = $derived.by(() => {
    const levels: LogLevel[] = [];
    if (showInfo) levels.push('info');
    if (showWarn) levels.push('warn');
    if (showError) levels.push('error');

    if (levels.length === 3) return debugState.logs;
    return debugState.logs.filter(entry => levels.includes(entry.level));
  });

  // Count by level
  const infoCount = $derived(debugState.logs.filter(l => l.level === 'info').length);
  const warnCount = $derived(debugState.logs.filter(l => l.level === 'warn').length);
  const errorCount = $derived(debugState.logs.filter(l => l.level === 'error').length);

  // Refresh interval for updating logs
  let refreshInterval: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    refreshDebugLogs();
    refreshInterval = setInterval(() => refreshDebugLogs(), 500);
    loadProviderStatus();
  });

  onDestroy(() => {
    if (refreshInterval) clearInterval(refreshInterval);
  });

  async function loadProviderStatus(): Promise<void> {
    try {
      providerStatus = await getProviderStatus();
    } catch (e) {
      providerStatusError = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleCopyLogs(): Promise<void> {
    const text = getLogsAsText(filteredLogs);
    await navigator.clipboard.writeText(text);
    copySuccess = true;
    setTimeout(() => copySuccess = false, 2000);
  }

  function configuredLabel(configured: boolean): string {
    return configured ? t('debug.provider_key_set') : t('debug.provider_key_missing');
  }
</script>

<div class="page-debug">
  <PageHeader
    title={t('debug.title')}
    subtitle={t('debug.subtitle')}
    backHref="/settings"
    backLabel={t('settings_hub.back_label')}
  />

  <LogFilter
    {showInfo}
    {showWarn}
    {showError}
    {autoScroll}
    {copySuccess}
    {infoCount}
    {warnCount}
    {errorCount}
    onToggleInfo={(v) => showInfo = v}
    onToggleWarn={(v) => showWarn = v}
    onToggleError={(v) => showError = v}
    onToggleAutoScroll={(v) => autoScroll = v}
    onCopy={handleCopyLogs}
    onClear={clearLogs}
  />

  <LogViewer
    logs={filteredLogs}
    {autoScroll}
    totalCount={debugState.logs.length}
  />

  <!-- Provider Status panel (Phase 6: replaces service endpoints) -->
  <section class="provider-status">
    <h3 class="section-title">{t('debug.providers_title')}</h3>

    {#if providerStatusError}
      <p class="status-error">{providerStatusError}</p>
    {:else if !providerStatus}
      <p class="status-loading">{t('debug.providers_loading')}</p>
    {:else}
      <!-- Credentials -->
      <div class="status-group">
        <h4 class="group-title">{t('debug.providers_credentials')}</h4>
        <div class="status-table">
          <div class="status-row">
            <span class="col-name">OpenAI</span>
            <span class="col-badge" class:badge-ok={providerStatus.openai_configured} class:badge-missing={!providerStatus.openai_configured}>
              {configuredLabel(providerStatus.openai_configured)}
            </span>
          </div>
          <div class="status-row">
            <span class="col-name">Anthropic</span>
            <span class="col-badge" class:badge-ok={providerStatus.anthropic_configured} class:badge-missing={!providerStatus.anthropic_configured}>
              {configuredLabel(providerStatus.anthropic_configured)}
            </span>
          </div>
          <div class="status-row">
            <span class="col-name">Gemini</span>
            <span class="col-badge" class:badge-ok={providerStatus.gemini_configured} class:badge-missing={!providerStatus.gemini_configured}>
              {configuredLabel(providerStatus.gemini_configured)}
            </span>
          </div>
          <div class="status-row">
            <span class="col-name">Grok</span>
            <span class="col-badge" class:badge-ok={providerStatus.grok_configured} class:badge-missing={!providerStatus.grok_configured}>
              {configuredLabel(providerStatus.grok_configured)}
            </span>
          </div>
          <div class="status-row">
            <span class="col-name">Soniox</span>
            <span class="col-badge" class:badge-ok={providerStatus.soniox_configured} class:badge-missing={!providerStatus.soniox_configured}>
              {configuredLabel(providerStatus.soniox_configured)}
            </span>
          </div>
          <div class="status-row">
            <span class="col-name">{t('settings.field_local_url')}</span>
            <code class="col-url">{providerStatus.ollama_url || t('debug.provider_not_set')}</code>
          </div>
        </div>
      </div>

      <!-- Engine & task assignments -->
      <div class="status-group">
        <h4 class="group-title">{t('debug.providers_assignments')}</h4>
        <div class="status-table">
          <div class="status-row">
            <span class="col-name">{t('debug.providers_active_engine')}</span>
            <code class="col-url">{providerStatus.active_engine}</code>
          </div>
          <div class="status-row">
            <span class="col-name">{t('debug.task_grammar')}</span>
            <code class="col-url">{providerStatus.grammar_provider} / {providerStatus.grammar_model}</code>
          </div>
          <div class="status-row">
            <span class="col-name">{t('debug.task_translate')}</span>
            <code class="col-url">{providerStatus.translate_provider} / {providerStatus.translate_model}</code>
          </div>
          <div class="status-row">
            <span class="col-name">{t('debug.task_improve')}</span>
            <code class="col-url">{providerStatus.improve_provider} / {providerStatus.improve_model}</code>
          </div>
          <div class="status-row">
            <span class="col-name">{t('debug.task_screen_question')}</span>
            <code class="col-url">{providerStatus.screen_question_provider} / {providerStatus.screen_question_model}</code>
          </div>
        </div>
      </div>
    {/if}
  </section>
</div>

<style>
  .page-debug {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-width: 100%;
  }

  .provider-status {
    flex-shrink: 0;
    padding: var(--space-md);
    border-top: 1px solid var(--color-border);
  }

  .section-title {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 var(--space-sm) 0;
  }

  .status-loading,
  .status-error {
    font-size: var(--font-size-sm);
    color: var(--color-text-muted);
  }

  .status-error { color: var(--color-error); }

  .status-group {
    margin-bottom: var(--space-md);
  }

  .group-title {
    font-size: var(--font-size-xs);
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin: 0 0 var(--space-xs) 0;
  }

  .status-table {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .status-row {
    display: flex;
    align-items: baseline;
    gap: var(--space-sm);
    font-size: var(--font-size-sm);
  }

  .col-name {
    flex-shrink: 0;
    font-weight: 500;
    min-width: 140px;
    color: var(--color-text-primary);
  }

  .col-badge {
    font-size: var(--font-size-xs);
    padding: 1px var(--space-xs);
    border-radius: var(--radius-sm);
    background: var(--color-surface-elevated);
    color: var(--color-text-muted);
  }

  .badge-ok {
    background: var(--color-success-bg, var(--color-surface-elevated));
    color: var(--color-success, var(--color-text-secondary));
  }

  .badge-missing {
    background: var(--color-warning-bg, var(--color-surface-elevated));
    color: var(--color-warning, var(--color-text-muted));
  }

  .col-url {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    word-break: break-all;
    min-width: 0;
  }
</style>
