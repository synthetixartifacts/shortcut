<script lang="ts">
  /**
   * Log Filter Controls
   *
   * Filter bar with level toggles, auto-scroll, copy, and clear buttons.
   */
  import { Button } from '$lib/components/ui';
  import { Icon } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';

  interface Props {
    showInfo: boolean;
    showWarn: boolean;
    showError: boolean;
    autoScroll: boolean;
    copySuccess: boolean;
    infoCount: number;
    warnCount: number;
    errorCount: number;
    onToggleInfo: (checked: boolean) => void;
    onToggleWarn: (checked: boolean) => void;
    onToggleError: (checked: boolean) => void;
    onToggleAutoScroll: (checked: boolean) => void;
    onCopy: () => void;
    onClear: () => void;
  }

  const {
    showInfo,
    showWarn,
    showError,
    autoScroll,
    copySuccess,
    infoCount,
    warnCount,
    errorCount,
    onToggleInfo,
    onToggleWarn,
    onToggleError,
    onToggleAutoScroll,
    onCopy,
    onClear,
  }: Props = $props();
</script>

<div class="debug-controls">
  <div class="filter-group">
    <span class="filter-label">{t('debug.filter_label')}</span>
    <label class="filter-toggle">
      <input type="checkbox" checked={showInfo} onchange={(e) => onToggleInfo((e.target as HTMLInputElement).checked)} />
      <span class="filter-badge info">{t('debug.filter_info')} ({infoCount})</span>
    </label>
    <label class="filter-toggle">
      <input type="checkbox" checked={showWarn} onchange={(e) => onToggleWarn((e.target as HTMLInputElement).checked)} />
      <span class="filter-badge warn">{t('debug.filter_warn')} ({warnCount})</span>
    </label>
    <label class="filter-toggle">
      <input type="checkbox" checked={showError} onchange={(e) => onToggleError((e.target as HTMLInputElement).checked)} />
      <span class="filter-badge error">{t('debug.filter_error')} ({errorCount})</span>
    </label>
  </div>

  <div class="action-group">
    <label class="auto-scroll-toggle">
      <input type="checkbox" checked={autoScroll} onchange={(e) => onToggleAutoScroll((e.target as HTMLInputElement).checked)} />
      <span>{t('debug.auto_scroll')}</span>
    </label>
    <Button onclick={onCopy} variant="secondary">
      <Icon name="copy" size={14} />
      {copySuccess ? t('debug.button_copied') : t('debug.button_copy')}
    </Button>
    <Button onclick={onClear} variant="secondary">
      <Icon name="trash" size={14} />
      {t('debug.button_clear')}
    </Button>
  </div>
</div>

<style>
  .debug-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-md);
    margin-bottom: var(--spacing-md);
    flex-wrap: wrap;
    gap: var(--spacing-sm);
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .filter-label {
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .filter-toggle {
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .filter-toggle input {
    display: none;
  }

  .filter-badge {
    padding: 2px 8px;
    border-radius: var(--border-radius-sm);
    font-size: 0.75rem;
    font-weight: 500;
    opacity: 0.5;
    transition: opacity 0.15s;
  }

  .filter-toggle input:checked + .filter-badge {
    opacity: 1;
  }

  .filter-badge.info {
    background: var(--color-primary-light);
    color: var(--color-primary);
  }

  .filter-badge.warn {
    background: var(--color-badge-warn-bg);
    color: var(--color-badge-warn-text);
  }

  .filter-badge.error {
    background: var(--color-danger-light);
    color: var(--color-danger);
  }

  .action-group {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .auto-scroll-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.85rem;
    color: var(--color-text-muted);
    cursor: pointer;
  }
</style>
