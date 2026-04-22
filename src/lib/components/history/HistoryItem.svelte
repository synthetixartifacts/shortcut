<script lang="ts">
  /**
   * HistoryItem - Single history entry card
   *
   * Clickable card that copies text on click with visual feedback.
   */
  import type { HistoryEntry } from '$lib/types';
  import { Button, Icon } from '$lib/components/ui/primitives';
  import { formatTimestamp, formatDuration } from '$lib/utils/format';
  import { t } from '$lib/i18n';

  interface Props {
    entry: HistoryEntry;
    onCopy: (text: string) => void;
    onDelete?: (id: string) => void;
  }

  let { entry, onCopy, onDelete }: Props = $props();
  let justCopied = $state(false);

  function handleClick(): void {
    onCopy(entry.text);
    justCopied = true;
    setTimeout(() => { justCopied = false; }, 1500);
  }

  function handleDeleteClick(): void {
    onDelete?.(entry.id);
  }
</script>

<div
  class="history-item"
  class:copied={justCopied}
  onclick={handleClick}
  onkeydown={(e) => e.key === 'Enter' && handleClick()}
  role="button"
  tabindex="0"
  title={t('history.click_to_copy')}
>
  <div class="item-content">
    <p class="item-text">{entry.text}</p>
    <div class="item-meta">
      <span class="timestamp">{formatTimestamp(entry.timestamp)}</span>
      <span class="duration">{formatDuration(entry.duration_ms)}</span>
      {#if entry.language}
        <span class="language">{entry.language.toUpperCase()}</span>
      {/if}
      {#if entry.engine}
        <span class="engine-badge" class:local={entry.engine.startsWith('local')}>
          {entry.engine?.startsWith('local') ? t('engine.badge_local') : t('engine.badge_cloud')}
        </span>
      {/if}
      {#if justCopied}
        <span class="copied-badge">{t('common.copied')}</span>
      {/if}
    </div>
  </div>
  {#if onDelete}
    <div class="item-actions">
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div role="presentation" onclick={(e) => e.stopPropagation()}>
        <Button variant="ghost" onclick={handleDeleteClick} title={t('common.delete')}>
          <Icon name="close" size={14} />
        </Button>
      </div>
    </div>
  {/if}
</div>

<style>
  .history-item {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--color-card-bg);
    border-radius: var(--border-radius-md);
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .history-item:hover {
    background: var(--color-primary-light);
    border-color: var(--color-primary-border);
  }

  .history-item:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .history-item.copied {
    background: var(--color-success-light);
    border-color: var(--color-success);
  }

  .item-content {
    flex: 1;
    min-width: 0;
  }

  .item-text {
    margin: 0 0 var(--spacing-xs) 0;
    color: var(--color-text);
    word-wrap: break-word;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .item-meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-sm);
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .timestamp, .duration {
    opacity: 0.8;
  }

  .language {
    padding: 1px 6px;
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    font-size: 0.65rem;
    font-weight: 500;
  }

  .engine-badge {
    padding: 1px 6px;
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    font-size: 0.65rem;
    font-weight: 500;
    color: var(--color-text-muted);
  }

  .engine-badge.local {
    background: var(--color-success-light);
    color: var(--color-success);
  }

  .copied-badge {
    color: var(--color-success);
    font-weight: 500;
  }

  .item-actions {
    flex-shrink: 0;
  }
</style>
