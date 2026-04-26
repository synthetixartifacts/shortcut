<script lang="ts">
  /**
   * ActionFilter - Pill-style filter for text-transform history entries.
   *
   * Renders four buttons (All / Grammar / Translate / Improve). The active
   * button receives `class="active"` and `aria-pressed="true"` for assistive
   * technology. Selecting a filter calls `onChange(filter)` — the page is
   * responsible for resetting pagination to page 1.
   */
  import type { TransformActionFilter } from '$lib/types';
  import { t } from '$lib/i18n';

  interface Props {
    value: TransformActionFilter;
    onChange: (filter: TransformActionFilter) => void;
  }

  let { value, onChange }: Props = $props();

  const filters: { id: TransformActionFilter; labelKey: string }[] = [
    { id: 'all', labelKey: 'text_transform_history.filter_all' },
    { id: 'grammar', labelKey: 'text_transform_history.filter_grammar' },
    { id: 'translate', labelKey: 'text_transform_history.filter_translate' },
    { id: 'improve', labelKey: 'text_transform_history.filter_improve' },
  ];
</script>

<div class="action-filter" role="group" aria-label={t('text_transform_history.filter_aria')}>
  {#each filters as filter (filter.id)}
    <button
      type="button"
      class="filter-pill"
      class:active={value === filter.id}
      aria-pressed={value === filter.id}
      onclick={() => onChange(filter.id)}
    >
      {t(filter.labelKey)}
    </button>
  {/each}
</div>

<style>
  .action-filter {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
  }

  .filter-pill {
    padding: var(--spacing-xs) var(--spacing-md);
    font-size: 0.85rem;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    border: 1px solid var(--color-kbd-border);
    background: var(--color-kbd-bg);
    color: var(--color-text);
    transition: all 0.15s ease;
  }

  .filter-pill:hover {
    background: var(--color-kbd-border);
  }

  .filter-pill.active {
    background: var(--color-primary-light);
    border-color: var(--color-primary);
    color: var(--color-primary);
    font-weight: 500;
  }
</style>
