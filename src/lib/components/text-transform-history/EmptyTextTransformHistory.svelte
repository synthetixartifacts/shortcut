<script lang="ts">
  /**
   * EmptyTextTransformHistory - Empty state for text-transform history list
   *
   * Renders different copy depending on whether the empty state is the
   * result of a search/filter producing zero matches, or a genuinely empty
   * history (no entries persisted yet).
   */
  import { t } from '$lib/i18n';

  interface Props {
    /** When true, render the "no results match" copy instead of the genuinely-empty copy. */
    filtered?: boolean;
  }

  let { filtered = false }: Props = $props();
</script>

<div class="empty-state">
  {#if filtered}
    <p class="empty-title">{t('text_transform_history.search_no_results_title')}</p>
    <p class="empty-description">{t('text_transform_history.search_no_results')}</p>
  {:else}
    <p class="empty-title">{t('text_transform_history.empty_title')}</p>
    <p class="empty-description">{t('text_transform_history.empty_description')}</p>
  {/if}
</div>

<style>
  .empty-state {
    text-align: center;
    padding: var(--spacing-xl) var(--spacing-lg);
    color: var(--color-text-muted);
  }

  .empty-title {
    margin: 0 0 var(--spacing-xs) 0;
    font-size: 1rem;
    color: var(--color-text);
  }

  .empty-description {
    margin: 0;
    font-size: 0.875rem;
  }
</style>
