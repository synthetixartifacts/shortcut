<script lang="ts">
  /**
   * ShortcutEditorModal Component
   *
   * Modal wrapper for editing a specific shortcut action.
   * Combines Modal and ShortcutRecorder components.
   */

  import Modal from './Modal.svelte';
  import ShortcutRecorder from './ShortcutRecorder.svelte';
  import type { ShortcutInfo } from '$lib/types';
  import { formatActionName } from '$lib/utils/format';
  import { t } from '$lib/i18n';

  function getActionHint(action: string): string {
    return action === 'dictation' ? t('shortcuts.hint_hold') : t('shortcuts.hint_press');
  }

  interface Props {
    isOpen: boolean;
    shortcutInfo: ShortcutInfo | null;
    defaultShortcut: string;
    onSave: (action: string, newShortcut: string) => Promise<void>;
    onClose: () => void;
  }

  let {
    isOpen,
    shortcutInfo,
    defaultShortcut,
    onSave,
    onClose,
  }: Props = $props();

  let isSaving = $state(false);
  let error = $state<string | null>(null);

  // Derive title from action
  const title = $derived(
    shortcutInfo
      ? t('shortcuts.editor_title', { action: formatActionName(shortcutInfo.action) })
      : t('shortcuts.editor_title_aria')
  );

  async function handleSave(newShortcut: string): Promise<void> {
    if (!shortcutInfo) return;

    isSaving = true;
    error = null;

    try {
      await onSave(shortcutInfo.action, newShortcut);
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to save shortcut';
    } finally {
      isSaving = false;
    }
  }

  async function handleReset(): Promise<void> {
    if (!shortcutInfo) return;

    isSaving = true;
    error = null;

    try {
      await onSave(shortcutInfo.action, defaultShortcut);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to reset shortcut';
    } finally {
      isSaving = false;
    }
  }

  function handleClose(): void {
    error = null;
    onClose();
  }
</script>

<Modal
  {isOpen}
  {title}
  onClose={handleClose}
  closeOnBackdropClick={!isSaving}
  closeOnEscape={!isSaving}
>
  {#if shortcutInfo}
    <div class="editor-content">
      <p class="description">{shortcutInfo.description}</p>
      <p class="action-hint-badge">
        <span class="hint-label">{getActionHint(shortcutInfo.action)}</span>
      </p>

      <ShortcutRecorder
        value={shortcutInfo.shortcut}
        defaultValue={defaultShortcut}
        onSave={handleSave}
        onReset={handleReset}
        disabled={isSaving}
      />

      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      {#if isSaving}
        <div class="saving-indicator">{t('shortcuts.editor_saving')}</div>
      {/if}
    </div>
  {/if}
</Modal>

<style>
  .editor-content {
    min-width: 280px;
  }

  .description {
    margin: 0 0 var(--spacing-xs);
    color: var(--color-text-muted);
    font-size: 0.9rem;
  }

  .action-hint-badge {
    margin: 0 0 var(--spacing-lg);
  }

  .hint-label {
    display: inline-block;
    font-size: 0.7rem;
    font-style: italic;
    color: var(--color-text-hint);
    background: var(--color-kbd-bg);
    padding: 2px 8px;
    border-radius: var(--border-radius-sm);
  }

  .error-message {
    margin-top: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-card-recording-bg);
    border: 1px solid var(--color-danger);
    border-radius: var(--border-radius-sm);
    color: var(--color-danger);
    font-size: 0.85rem;
  }

  .saving-indicator {
    margin-top: var(--spacing-md);
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }
</style>
