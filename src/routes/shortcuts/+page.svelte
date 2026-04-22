<script lang="ts">
  /**
   * Shortcuts Page - Full keyboard shortcuts management
   */
  import { onMount } from 'svelte';

  // State
  import {
    shortcutsState,
    shortcutsSaveStatus,
    loadDefaultShortcuts,
    updateShortcut,
    getDefaultShortcut,
    resetAllShortcuts,
  } from '$lib/state/shortcuts.svelte';

  // Components
  import ShortcutEditorModal from '$lib/components/ui/ShortcutEditorModal.svelte';
  import { PageHeader, ShortcutsList, Button } from '$lib/components/ui';

  // Types
  import type { ShortcutInfo } from '$lib/types';
  import { t } from '$lib/i18n';

  // Editor state
  let isEditorOpen = $state(false);
  let editingShortcut = $state<ShortcutInfo | null>(null);
  let editingDefaultShortcut = $state('');
  let isResetting = $state(false);

  onMount(async () => {
    await loadDefaultShortcuts();
  });

  function handleShortcutEdit(shortcut: ShortcutInfo): void {
    editingShortcut = shortcut;
    editingDefaultShortcut = getDefaultShortcut(shortcut.action);
    isEditorOpen = true;
  }

  async function handleShortcutSave(action: string, newShortcut: string): Promise<void> {
    await updateShortcut(action, newShortcut);
    if (editingShortcut && editingShortcut.action === action) {
      editingShortcut = { ...editingShortcut, shortcut: newShortcut };
    }
  }

  function handleEditorClose(): void {
    isEditorOpen = false;
    editingShortcut = null;
    editingDefaultShortcut = '';
  }

  async function handleResetAll(): Promise<void> {
    if (!confirm(t('shortcuts.confirm_reset_all'))) return;
    isResetting = true;
    try {
      await resetAllShortcuts();
    } finally {
      isResetting = false;
    }
  }
</script>

<div class="page-shortcuts">
  <PageHeader
    title={t('shortcuts.title')}
    subtitle={t('shortcuts.subtitle')}
    backHref="/settings"
    backLabel={t('settings_hub.back_label')}
  >
    {#snippet actions()}
      <Button variant="secondary" onclick={handleResetAll} disabled={isResetting}>
        {isResetting ? t('shortcuts.button_resetting') : t('shortcuts.button_reset_all')}
      </Button>
    {/snippet}
  </PageHeader>

  <ShortcutsList
    shortcuts={shortcutsState}
    onEdit={handleShortcutEdit}
    editable={true}
    saveStatusByAction={shortcutsSaveStatus}
  />
</div>

<ShortcutEditorModal
  isOpen={isEditorOpen}
  shortcutInfo={editingShortcut}
  defaultShortcut={editingDefaultShortcut}
  onSave={handleShortcutSave}
  onClose={handleEditorClose}
/>

<style>
  .page-shortcuts {
    max-width: var(--page-max-width);
  }
</style>
