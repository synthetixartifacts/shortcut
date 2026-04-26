<script lang="ts">
  /**
   * Recent Entries
   *
   * Displays the last few history entries with copy-on-click. Accepts a
   * combined feed of dictation and text-transform results, distinguished by
   * `kind` for the per-entry badge.
   */
  import { formatTimestamp } from '$lib/utils/format';
  import { t } from '$lib/i18n';

  export type RecentEntryKind = 'dictation' | 'grammar' | 'translate' | 'improve';

  export interface RecentEntry {
    id: string;
    kind: RecentEntryKind;
    text: string;
    timestamp: number;
  }

  interface Props {
    entries: RecentEntry[];
    copiedId: string | null;
    onCopy: (id: string, text: string) => void;
  }

  const { entries, copiedId, onCopy }: Props = $props();

  function badgeLabel(kind: RecentEntryKind): string {
    return kind === 'dictation'
      ? t('dashboard.recent_kind_dictation')
      : t(`text_transform_history.action_${kind}`);
  }
</script>

<section class="recent">
  <div class="section-header">
    <h2 class="section-title">{t('dashboard.recent_heading')}</h2>
    <a href="/history" class="view-all">{t('dashboard.recent_view_all')}</a>
  </div>
  <div class="recent-list">
    {#each entries as entry (entry.id)}
      <button
        class="recent-item"
        class:copied={copiedId === entry.id}
        onclick={() => onCopy(entry.id, entry.text)}
        title={t('dashboard.recent_click_to_copy')}
      >
        <p class="recent-text">{entry.text}</p>
        <div class="recent-meta">
          <span class="timestamp">{formatTimestamp(entry.timestamp)}</span>
          <span class="kind-badge kind-{entry.kind}">{badgeLabel(entry.kind)}</span>
          {#if copiedId === entry.id}
            <span class="copied-badge">{t('common.copied')}</span>
          {/if}
        </div>
      </button>
    {/each}
  </div>
</section>

<style>
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-md);
  }

  .section-header :global(.section-title) {
    margin: 0;
  }

  .view-all {
    font-size: 0.8rem;
    color: var(--color-primary);
    text-decoration: none;
  }

  .view-all:hover {
    text-decoration: underline;
  }

  .recent-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .recent-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: var(--spacing-md);
    background: var(--color-card-bg);
    border: 1px solid transparent;
    border-radius: var(--border-radius-md);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .recent-item:hover {
    background: var(--color-primary-light);
    border-color: var(--color-primary-border);
  }

  .recent-item:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .recent-item.copied {
    background: var(--color-success-light);
    border-color: var(--color-success);
  }

  .recent-text {
    margin: 0 0 var(--spacing-xs) 0;
    font-size: 0.9rem;
    color: var(--color-text);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .recent-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .kind-badge {
    padding: 1px 6px;
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    font-size: 0.65rem;
    font-weight: 500;
    color: var(--color-text-muted);
  }

  .kind-badge.kind-dictation {
    background: var(--color-kbd-bg);
    color: var(--color-text);
  }

  .kind-badge.kind-grammar {
    background: var(--color-primary-light);
    color: var(--color-primary);
  }

  .kind-badge.kind-translate {
    background: var(--color-success-light);
    color: var(--color-success);
  }

  .kind-badge.kind-improve {
    background: var(--color-kbd-bg);
    color: var(--color-text);
  }

  .copied-badge {
    color: var(--color-success);
    font-weight: 500;
  }
</style>
