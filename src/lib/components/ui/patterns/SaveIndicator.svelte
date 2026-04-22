<script lang="ts">
  /**
   * SaveIndicator — inline per-field save feedback.
   * States: idle (reserved slot) | saving (spinner) | saved (check) | error (msg).
   * `aria-live="polite"` on the always-present root announces transitions.
   */
  import { t } from '$lib/i18n';
  import { Icon } from '$lib/components/ui/primitives';
  import type { SaveStatus } from '$lib/utils/save-status.svelte';

  interface Props { status: SaveStatus; }
  let { status }: Props = $props();
</script>

<span class="save-indicator state-{status.status}" aria-live="polite">
  {#if status.status === 'idle'}
    <span class="placeholder" aria-hidden="true"></span>
  {:else if status.status === 'saving'}
    <span class="content">
      <span class="spinner" aria-hidden="true"></span>
      <span class="text">{t('save_status.saving')}</span>
    </span>
  {:else if status.status === 'saved'}
    <span class="content">
      <Icon name="check" size={14} />
      <span class="text">{t('save_status.saved')}</span>
    </span>
  {:else if status.status === 'error'}
    <span class="content" title={status.message ?? ''}>
      <Icon name="warning" size={14} />
      <span class="text">{status.message ?? t('save_status.error')}</span>
    </span>
  {/if}
</span>

<style>
  .save-indicator {
    display: inline-flex;
    align-items: center;
    min-height: 1.1rem;
    font-size: 0.75rem;
    line-height: 1.1;
  }
  .placeholder {
    display: inline-block; visibility: hidden; width: 1px; height: 1.1rem;
  }
  .content {
    display: inline-flex; align-items: center; gap: 0.35rem;
    transition: opacity 0.15s ease;
  }
  .text {
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 22ch;
  }
  .state-saving { color: var(--color-text-muted); }
  .state-saved  { color: var(--color-success); }
  .state-error  { color: var(--color-danger); }
  .spinner {
    display: inline-block;
    width: 12px; height: 12px;
    border: 2px solid var(--color-text-muted);
    border-top-color: var(--color-primary);
    border-radius: 50%;
  }
  @media (prefers-reduced-motion: no-preference) {
    .spinner { animation: save-indicator-spin 0.9s linear infinite; }
  }
  @keyframes save-indicator-spin { to { transform: rotate(360deg); } }
</style>
