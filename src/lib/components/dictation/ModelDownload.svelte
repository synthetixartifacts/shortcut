<script lang="ts">
  /**
   * ModelDownload - Download progress UI for the local STT model
   *
   * Presentational wrapper around the shared `useModelDownload()` controller,
   * which owns the Tauri listeners + download state machine.
   */
  import { onMount, onDestroy } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { useModelDownload } from '$lib/features/dictation/model-download.svelte';
  import { Button } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';

  interface Props {
    onComplete?: () => void;
    onCancel?: () => void;
  }

  let { onComplete, onCancel }: Props = $props();

  const dl = useModelDownload();
  let cleanup: UnlistenFn | null = null;
  let wasReady = false;

  onMount(async () => {
    cleanup = await dl.attach();
  });

  onDestroy(() => {
    cleanup?.();
  });

  // Fire onComplete once when the model transitions to "ready".
  $effect(() => {
    if (dl.status.state === 'ready' && !wasReady) {
      wasReady = true;
      onComplete?.();
    }
    if (dl.status.state !== 'ready') {
      wasReady = false;
    }
  });

  async function handleCancel(): Promise<void> {
    await dl.cancel();
    onCancel?.();
  }
</script>

<div class="model-download">
  <h3 class="download-title">{t('engine.download_title')}</h3>
  <p class="download-info">{t('engine.download_size_info')}</p>

  {#if dl.isDownloading || dl.status.state === 'downloading'}
    <div class="progress-section">
      <div class="progress-bar">
        <div class="progress-fill" style="width: {dl.progressPercent}%"></div>
      </div>
      <p class="progress-text">{t('engine.download_progress', { percent: String(dl.progressPercent) })}</p>
      <Button variant="ghost" onclick={handleCancel}>{t('engine.download_cancel')}</Button>
    </div>
  {:else if dl.status.state === 'ready'}
    <p class="download-complete">{t('engine.download_complete')}</p>
  {:else}
    <Button variant="primary" onclick={() => dl.start()}>{t('engine.download_model')}</Button>
  {/if}

  {#if dl.error}
    <div class="download-error">
      <p class="error-text">{t('engine.download_error', { error: dl.error })}</p>
      <Button variant="secondary" onclick={() => dl.start()}>{t('engine.download_retry')}</Button>
    </div>
  {/if}
</div>

<style>
  .model-download {
    padding: var(--spacing-md);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-md);
    background: var(--color-surface);
  }

  .download-title {
    margin: 0 0 var(--spacing-xs);
    font-size: 0.95rem;
    font-weight: 600;
  }

  .download-info {
    margin: 0 0 var(--spacing-md);
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .progress-bar {
    height: 8px;
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary);
    border-radius: var(--border-radius-sm);
    transition: width 0.3s ease;
  }

  .progress-text {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .download-complete {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-success);
    font-weight: 500;
  }

  .download-error {
    margin-top: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: var(--color-danger-light);
    border-radius: var(--border-radius-sm);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .error-text {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-danger);
  }
</style>
