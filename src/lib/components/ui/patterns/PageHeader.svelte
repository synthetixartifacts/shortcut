<script lang="ts">
  /**
   * PageHeader - Consistent page header with optional back link and actions
   *
   * Use on all pages for consistent header styling.
   */
  import type { Snippet } from 'svelte';
  import { t } from '$lib/i18n';

  interface Props {
    title: string;
    subtitle?: string;
    backHref?: string;
    backLabel?: string;
    actions?: Snippet;
  }

  let {
    title,
    subtitle,
    backHref,
    backLabel = t('common.back'),
    actions,
  }: Props = $props();
</script>

<header class="page-header">
  {#if backHref}
    <a href={backHref} class="back-link">&larr; {backLabel}</a>
  {/if}

  <div class="header-content">
    <div class="header-text">
      <h1>{title}</h1>
      {#if subtitle}
        <p class="subtitle">{subtitle}</p>
      {/if}
    </div>

    {#if actions}
      <div class="header-actions">
        {@render actions()}
      </div>
    {/if}
  </div>
</header>

<style>
  .page-header {
    margin-bottom: var(--page-header-gap, var(--spacing-xl));
  }

  .back-link {
    display: inline-block;
    margin-bottom: var(--spacing-sm);
    color: var(--color-primary);
    text-decoration: none;
    font-size: 0.85rem;
  }

  .back-link:hover {
    text-decoration: underline;
  }

  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--spacing-md);
  }

  .header-text h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  .subtitle {
    margin: var(--spacing-xs) 0 0;
    color: var(--color-text-muted);
  }

  .header-actions {
    flex-shrink: 0;
  }
</style>
