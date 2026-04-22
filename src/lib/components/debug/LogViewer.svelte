<script lang="ts">
  /**
   * Log Viewer
   *
   * Displays log entries with auto-scroll support, level badges, and timestamps.
   */
  import { Icon } from '$lib/components/ui/primitives';
  import type { LogLevel, LogEntry } from '$lib/state/debug.svelte';
  import { t } from '$lib/i18n';

  interface Props {
    logs: LogEntry[];
    autoScroll: boolean;
    totalCount: number;
  }

  const { logs, autoScroll, totalCount }: Props = $props();

  let logContainer: HTMLDivElement | undefined;
  let previousLogCount = 0;
  let userScrolledUp = $state(false);

  // Pause auto-scroll while the user is reading higher up the buffer.
  // The page polls for new logs every 500ms, so without this guard the
  // effect below would yank the viewport back to the bottom on every tick.
  function handleScroll(): void {
    if (!logContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = logContainer;
    userScrolledUp = scrollHeight - scrollTop - clientHeight > 20;
  }

  // Auto-scroll to bottom only when new logs are actually appended
  // and the user has not scrolled away from the bottom.
  $effect(() => {
    const currentLength = logs.length;
    const grew = currentLength > previousLogCount;
    previousLogCount = currentLength;

    if (grew && autoScroll && !userScrolledUp && logContainer) {
      requestAnimationFrame(() => {
        if (logContainer) {
          logContainer.scrollTop = logContainer.scrollHeight;
        }
      });
    }
  });

  function formatTime(date: Date): string {
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3,
    });
  }

  function getLevelClass(level: LogLevel): string {
    switch (level) {
      case 'error': return 'log-error';
      case 'warn': return 'log-warn';
      default: return 'log-info';
    }
  }
</script>

<div class="log-container" bind:this={logContainer} onscroll={handleScroll}>
  {#if logs.length === 0}
    <div class="empty-state">
      <Icon name="terminal" size={32} />
      <p>{t('debug.empty_title')}</p>
      <p class="hint">{t('debug.empty_hint')}</p>
    </div>
  {:else}
    {#each logs as entry (entry.id)}
      <div class="log-entry {getLevelClass(entry.level)}">
        <span class="log-time">{formatTime(entry.timestamp)}</span>
        <span class="log-level">{entry.level.toUpperCase()}</span>
        <span class="log-message">{entry.message}</span>
      </div>
    {/each}
  {/if}
</div>

<div class="debug-footer">
  <span class="log-count">{logs.length} / {totalCount} {t('debug.entry_count')}</span>
  <span class="hint">{t('debug.footer_hint')}</span>
</div>

<style>
  .log-container {
    flex: 1;
    min-height: 300px;
    max-height: calc(100vh - 320px);
    overflow-y: auto;
    background: var(--color-debug-bg);
    border-radius: var(--border-radius-md);
    padding: var(--spacing-sm);
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Consolas', monospace;
    font-size: 0.8rem;
    line-height: 1.5;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--color-text-muted);
    text-align: center;
  }

  .empty-state p {
    margin: var(--spacing-xs) 0 0 0;
  }

  .hint {
    font-size: 0.75rem;
    color: var(--color-text-hint);
  }

  .log-entry {
    display: flex;
    gap: var(--spacing-sm);
    padding: 2px 4px;
    border-radius: 2px;
    color: var(--color-debug-text);
  }

  .log-entry:hover {
    background: var(--color-debug-stripe);
  }

  .log-time {
    color: var(--color-debug-hint);
    flex-shrink: 0;
  }

  .log-level {
    width: 45px;
    flex-shrink: 0;
    font-weight: 600;
  }

  .log-message {
    word-break: break-word;
    white-space: pre-wrap;
  }

  .log-info .log-level {
    color: var(--color-primary);
  }

  .log-warn .log-level {
    color: var(--color-log-warn);
  }

  .log-error .log-level {
    color: var(--color-log-error);
  }

  .log-error {
    background: var(--color-debug-error-bg);
  }

  .debug-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm) 0;
    margin-top: var(--spacing-sm);
  }

  .log-count {
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }
</style>
