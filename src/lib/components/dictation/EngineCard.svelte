<script lang="ts">
  /**
   * EngineCard - Individual transcription engine card
   *
   * Displays engine name, status badge, description, privacy info,
   * model size (if applicable), and an action button. When the engine
   * requires a model download, shows inline download UI backed by the
   * shared `useModelDownload()` controller.
   */
  import { onMount, onDestroy } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { useModelDownload } from '$lib/features/dictation/model-download.svelte';
  import type { EngineInfo } from '$lib/types';
  import { Button } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';

  interface Props {
    engine: EngineInfo;
    showDownload?: boolean;
    onActivate: (id: string) => void;
    onDownloadComplete?: () => void;
    onDownloadCancel?: () => void;
  }

  let { engine, showDownload = false, onActivate, onDownloadComplete, onDownloadCancel }: Props = $props();

  const statusLabel = $derived(t(`engine.status_${engine.status}`));
  const statusClass = $derived(`status-${engine.status}`);

  const dl = useModelDownload();
  let cleanup: UnlistenFn | null = null;
  let wasReady = false;

  onMount(async () => {
    if (!engine.capabilities.requires_model_download) return;
    cleanup = await dl.attach();
  });

  onDestroy(() => {
    cleanup?.();
  });

  // Fire onDownloadComplete once when the model transitions to "ready".
  $effect(() => {
    if (dl.status.state === 'ready' && !wasReady) {
      wasReady = true;
      onDownloadComplete?.();
    }
    if (dl.status.state !== 'ready') {
      wasReady = false;
    }
  });

  async function cancelDownload(): Promise<void> {
    await dl.cancel();
    onDownloadCancel?.();
  }

  const isDownloadVisible = $derived(
    showDownload || dl.isDownloading || dl.status.state === 'downloading'
  );
</script>

<div class="engine-card" class:active={engine.status === 'active'}>
  <div class="engine-header">
    <h3 class="engine-name">{t(engine.display_name)}</h3>
    <span class="status-badge {statusClass}">{statusLabel}</span>
  </div>
  <p class="engine-desc">{t(engine.description)}</p>
  <p class="engine-privacy">{t(engine.privacy_summary)}</p>
  {#if engine.model_size_mb}
    <p class="engine-size">{t('engine.model_size', { size: String(engine.model_size_mb) })}</p>
  {/if}

  <!-- Inline download UI -->
  {#if isDownloadVisible && engine.capabilities.requires_model_download}
    <div class="download-inline">
      {#if dl.isDownloading || dl.status.state === 'downloading'}
        <div class="progress-bar"><div class="progress-fill" style="width: {dl.progressPercent}%"></div></div>
        <div class="progress-row">
          <span class="progress-text">{t('engine.download_progress', { percent: String(dl.progressPercent) })}</span>
          <Button variant="ghost" onclick={cancelDownload}>{t('engine.download_cancel')}</Button>
        </div>
      {:else if dl.status.state === 'ready'}
        <p class="download-done">{t('engine.download_complete')}</p>
      {:else}
        <p class="download-hint">{t('engine.download_size_info')}</p>
        <Button variant="primary" onclick={() => dl.start()}>{t('engine.download_model')}</Button>
      {/if}
      {#if dl.error}
        <div class="download-error">
          <p class="error-text">{t('engine.download_error', { error: dl.error })}</p>
          <Button variant="secondary" onclick={() => dl.start()}>{t('engine.download_retry')}</Button>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Action button (hidden during download) -->
  {#if !isDownloadVisible}
    <div class="engine-actions">
      {#if engine.status === 'active'}
        <span class="active-label">{t('engine.currently_active')}</span>
      {:else if engine.status === 'coming_soon'}
        <span class="muted-label">{t('engine.coming_soon_hint')}</span>
      {:else if engine.status === 'not_available'}
        <span class="muted-label">{t('engine.not_available_hint')}</span>
      {:else if engine.status === 'not_configured'}
        <span class="muted-label">{t('engine.not_configured_hint')}</span>
        <a class="settings-link" href="/settings/providers">{t('engine.configure_link')}</a>
      {:else if engine.status === 'not_downloaded'}
        <Button variant="primary" onclick={() => onActivate(engine.id)}>
          {t('engine.download_and_activate')}
        </Button>
      {:else}
        <Button variant="secondary" onclick={() => onActivate(engine.id)}>
          {t('engine.make_active')}
        </Button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .engine-card {
    padding: var(--spacing-md);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-md);
    background: var(--color-surface);
    transition: border-color 0.15s ease;
  }

  .engine-card.active {
    border-color: var(--color-primary);
    background: var(--color-primary-light);
  }

  .engine-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .engine-name { margin: 0; font-size: 0.95rem; font-weight: 600; }

  .status-badge {
    font-size: 0.75rem; font-weight: 500;
    padding: 2px var(--spacing-sm);
    border-radius: var(--border-radius-sm);
    white-space: nowrap;
  }

  .status-active { background: var(--color-success-light); color: var(--color-success); }
  .status-available { background: var(--color-kbd-bg); color: var(--color-text-muted); }
  .status-not_downloaded { background: var(--color-warning-light); color: var(--color-warning); }
  .status-downloading { background: var(--color-primary-light); color: var(--color-primary); }
  .status-not_available, .status-coming_soon { background: var(--color-kbd-bg); color: var(--color-text-hint); }
  .status-not_configured { background: var(--color-warning-light); color: var(--color-warning); }

  .engine-desc, .engine-privacy, .engine-size {
    margin: 0 0 var(--spacing-xs); font-size: 0.85rem; color: var(--color-text-muted);
  }
  .engine-privacy { font-style: italic; font-size: 0.8rem; }
  .engine-size { font-size: 0.8rem; color: var(--color-text-hint); }

  .engine-actions {
    margin-top: var(--spacing-sm);
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-wrap: wrap;
  }
  .active-label { font-size: 0.85rem; font-weight: 500; color: var(--color-success); }
  .muted-label { font-size: 0.85rem; color: var(--color-text-hint); font-style: italic; }
  .settings-link { font-size: 0.85rem; color: var(--color-primary); text-decoration: none; }
  .settings-link:hover { text-decoration: underline; }

  /* Inline download */
  .download-inline {
    margin-top: var(--spacing-sm);
    padding-top: var(--spacing-sm);
    border-top: 1px solid var(--color-kbd-border);
  }

  .download-hint { margin: 0 0 var(--spacing-sm); font-size: 0.8rem; color: var(--color-text-muted); }

  .progress-bar {
    height: 6px; background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm); overflow: hidden;
  }
  .progress-fill {
    height: 100%; background: var(--color-primary);
    border-radius: var(--border-radius-sm); transition: width 0.3s ease;
  }

  .progress-row {
    display: flex; align-items: center; justify-content: space-between;
    margin-top: var(--spacing-xs);
  }
  .progress-text { font-size: 0.8rem; color: var(--color-text-muted); }

  .download-done { margin: 0; font-size: 0.85rem; color: var(--color-success); font-weight: 500; }

  .download-error {
    margin-top: var(--spacing-sm); padding: var(--spacing-sm);
    background: var(--color-danger-light); border-radius: var(--border-radius-sm);
  }
  .error-text { margin: 0 0 var(--spacing-xs); font-size: 0.8rem; color: var(--color-danger); }
</style>
