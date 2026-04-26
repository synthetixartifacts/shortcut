<script lang="ts">
  /**
   * TextTransformHistoryItem - Single text-transform history entry card.
   *
   * Click-to-copy with visual feedback (mirrors HistoryItem.svelte).
   * Renders only the result text, the timestamp, and an action badge
   * (Grammar / Translate / Improve). NO source text, NO provider/model
   * metadata — those are out of scope for this feature (D2, D3).
   */
  import type { TextTransformHistoryEntry, TransformAction } from '$lib/types';
  import { Button, Icon } from '$lib/components/ui/primitives';
  import { formatTimestamp } from '$lib/utils/format';
  import { t } from '$lib/i18n';

  interface Props {
    entry: TextTransformHistoryEntry;
    onCopy: (text: string) => void;
    onDelete?: (id: string) => void;
  }

  let { entry, onCopy, onDelete }: Props = $props();
  let justCopied = $state(false);

  function handleClick(): void {
    onCopy(entry.result);
    justCopied = true;
    setTimeout(() => {
      justCopied = false;
    }, 1500);
  }

  function handleDeleteClick(): void {
    onDelete?.(entry.id);
  }

  /** Locale key for the action badge label. */
  function actionLabelKey(action: TransformAction): string {
    return `text_transform_history.action_${action}`;
  }
</script>

<div
  class="history-item"
  class:copied={justCopied}
  onclick={handleClick}
  onkeydown={(e) => e.key === 'Enter' && handleClick()}
  role="button"
  tabindex="0"
  title={t('text_transform_history.click_to_copy')}
>
  <div class="item-content">
    <p class="item-text">{entry.result}</p>
    <div class="item-meta">
      <span class="timestamp">{formatTimestamp(entry.timestamp)}</span>
      <span class="action-badge action-{entry.action}">
        {t(actionLabelKey(entry.action))}
      </span>
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

  .timestamp {
    opacity: 0.8;
  }

  .action-badge {
    padding: 1px 6px;
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    font-size: 0.65rem;
    font-weight: 500;
    color: var(--color-text-muted);
  }

  .action-badge.action-grammar {
    background: var(--color-primary-light);
    color: var(--color-primary);
  }

  .action-badge.action-translate {
    background: var(--color-success-light);
    color: var(--color-success);
  }

  .action-badge.action-improve {
    background: var(--color-kbd-bg);
    color: var(--color-text);
  }

  .copied-badge {
    color: var(--color-success);
    font-weight: 500;
  }

  .item-actions {
    flex-shrink: 0;
  }
</style>
