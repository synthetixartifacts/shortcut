<script lang="ts">
  /**
   * ShortcutsList Component
   *
   * Displays registered keyboard shortcuts in a list format.
   * Supports click-to-edit when editable is true.
   */

  import { onMount } from 'svelte';
  import type { ShortcutInfo } from '$lib/types';
  import { shortcutToDisplay, getCurrentPlatform, type Platform } from '$lib/features/shortcuts';
  import { Icon, Kbd } from './primitives';
  import { SaveIndicator } from './patterns';
  import type { SaveStatus } from '$lib/utils/save-status.svelte';
  import { t } from '$lib/i18n';

  interface Props {
    shortcuts: ShortcutInfo[];
    onEdit?: (shortcut: ShortcutInfo) => void;
    editable?: boolean;
    saveStatusByAction?: Record<string, SaveStatus>;
  }

  let { shortcuts, onEdit, editable = false, saveStatusByAction }: Props = $props();

  const ACTION_ICONS: Record<string, string> = {
    dictation: '🎤',
    grammar: '📝',
    translate: '🌐',
    improve: '✨',
    open_menu: '🎯',
    screen_question: '📷',
  };

  function getActionIcon(action: string): string {
    return ACTION_ICONS[action] ?? '⌨️';
  }

  function getActionHint(action: string): string {
    return action === 'dictation' ? t('shortcuts.hint_hold') : t('shortcuts.hint_press');
  }

  let platform = $state<Platform>('windows');

  onMount(async () => {
    platform = await getCurrentPlatform();
  });

  function handleClick(shortcut: ShortcutInfo): void {
    if (editable && onEdit) {
      onEdit(shortcut);
    }
  }

  function handleKeyDown(event: KeyboardEvent, shortcut: ShortcutInfo): void {
    if (editable && onEdit && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      onEdit(shortcut);
    }
  }
</script>

<div class="shortcuts-section">
  <h2 class="label-caps">{t('shortcuts.list_heading')}</h2>
  <div class="shortcuts-list">
    {#each shortcuts as shortcut (shortcut.action)}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div
        class="shortcut-item"
        class:editable
        onclick={() => handleClick(shortcut)}
        onkeydown={(e) => handleKeyDown(e, shortcut)}
        role={editable ? 'button' : undefined}
        tabindex={editable ? 0 : undefined}
      >
        <span class="action-icon">{getActionIcon(shortcut.action)}</span>
        <Kbd keys={shortcutToDisplay(shortcut.shortcut, platform)} class="shortcut-kbd" />
        <span class="action-hint">{getActionHint(shortcut.action)}</span>
        <span class="shortcut-desc">{shortcut.description}</span>
        {#if editable}
          <span class="edit-icon" aria-hidden="true">
            <Icon name="edit" size={14} />
          </span>
        {/if}
        {#if saveStatusByAction?.[shortcut.action]}
          <span class="row-save-indicator">
            <SaveIndicator status={saveStatusByAction[shortcut.action]} />
          </span>
        {/if}
      </div>
    {/each}
  </div>
  {#if editable}
    <p class="edit-hint">{t('shortcuts.list_edit_hint')}</p>
  {/if}
</div>

<style>
  .shortcuts-section {
    margin-bottom: var(--spacing-lg);
  }

  .shortcuts-section h2 {
    margin: 0 0 var(--spacing-md);
  }

  .shortcuts-list {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--spacing-sm);
  }

  @media (max-width: 1100px) {
    .shortcuts-list {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 700px) {
    .shortcuts-list {
      grid-template-columns: minmax(0, 1fr);
    }
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-card-bg);
    border-radius: var(--border-radius-md);
    transition: all 0.15s ease;
  }

  .shortcut-item.editable {
    cursor: pointer;
  }

  .shortcut-item.editable:hover {
    background: var(--color-preview-bg);
  }

  .shortcut-item.editable:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .action-icon {
    font-size: 1.25rem;
    line-height: 1;
    flex-shrink: 0;
  }

  .shortcut-item :global(.shortcut-kbd) {
    min-width: 120px;
  }

  .action-hint {
    font-size: 0.65rem;
    color: var(--color-text-hint);
    font-style: italic;
    flex-shrink: 0;
  }

  .shortcut-desc {
    font-size: 0.9rem;
    flex: 1;
  }

  .edit-icon {
    opacity: 0;
    color: var(--color-text-muted);
    transition: opacity 0.15s ease;
  }

  .row-save-indicator {
    display: inline-flex;
    align-items: center;
    justify-content: flex-end;
    flex-shrink: 0;
    min-width: 5rem;
  }

  .shortcut-item.editable:hover .edit-icon,
  .shortcut-item.editable:focus .edit-icon {
    opacity: 1;
  }

  .edit-hint {
    margin: var(--spacing-sm) 0 0;
    font-size: 0.75rem;
    color: var(--color-text-hint);
    text-align: center;
  }
</style>
