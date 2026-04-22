<script lang="ts">
  /**
   * ProviderStatusCard Component
   *
   * Shows per-provider readiness as a compact badge in the dashboard header.
   */
  import { t } from '$lib/i18n';

  type ProviderSummary = 'checking' | 'ready' | 'partial' | 'none';

  interface Props {
    summary: ProviderSummary;
  }

  let { summary }: Props = $props();
</script>

{#if summary === 'checking'}
  <span class="provider-badge provider-badge--checking">{t('status.checking')}</span>
{:else if summary === 'ready'}
  <span class="provider-badge provider-badge--ready">{t('dashboard.providers_ready')}</span>
{:else if summary === 'none'}
  <a href="/settings/providers" class="provider-badge provider-badge--none">{t('dashboard.configure_providers')}</a>
{:else}
  <a href="/settings/providers" class="provider-badge provider-badge--partial">{t('dashboard.providers_partial')}</a>
{/if}

<style>
  .provider-badge {
    display: flex;
    align-items: center;
    padding: 4px var(--spacing-sm);
    font-size: 0.8rem;
    font-weight: 500;
    border-radius: var(--border-radius-md);
    border: 1px solid var(--color-kbd-border);
    white-space: nowrap;
    text-decoration: none;
    color: var(--color-text);
    background: var(--color-card-bg);
  }

  .provider-badge--checking {
    color: var(--color-text-muted);
  }

  .provider-badge--ready {
    border-color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 8%, var(--color-card-bg));
    color: var(--color-success);
  }

  .provider-badge--none {
    border-color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 8%, var(--color-card-bg));
    color: var(--color-danger);
  }

  .provider-badge--partial {
    border-color: var(--color-warning);
    background: color-mix(in srgb, var(--color-warning) 8%, var(--color-card-bg));
    color: var(--color-warning);
  }

  .provider-badge--none:hover,
  .provider-badge--partial:hover {
    filter: brightness(1.05);
  }
</style>
