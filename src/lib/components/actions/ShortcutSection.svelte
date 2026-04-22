<script lang="ts">
  /**
   * ShortcutSection - Per-action "Keyboard Shortcut" section.
   *
   * Shared by /actions/{improve,grammar,translate,screen-question} to display the
   * current binding for the action and open the canonical ShortcutEditorModal on
   * edit. Writes flow through shortcuts.svelte.ts::updateShortcut — the SAME path
   * /shortcuts uses, so edits appear everywhere that reads `shortcutsState`.
   *
   * No new editor, no new state module. Pure wrapper.
   */
  import { onMount } from 'svelte';
  import {
    shortcutsState,
    loadDefaultShortcuts,
    updateShortcut,
    getDefaultShortcut,
    setShortcuts,
  } from '$lib/state/shortcuts.svelte';
  import { getRegisteredShortcuts } from '$lib/api/tauri';
  import ShortcutEditorModal from '$lib/components/ui/ShortcutEditorModal.svelte';
  import { SettingsSection } from '$lib/components/settings';
  import { Kbd, Button } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';

  interface Props {
    /** Action key matching shortcuts config (improve|grammar|translate|screen_question). */
    actionKey: string;
    /** i18n namespace providing section_shortcut / shortcut_label / button_edit_shortcut keys. */
    translationNamespace: string;
  }

  let { actionKey, translationNamespace }: Props = $props();

  let isEditorOpen = $state(false);
  let editingDefault = $state('');

  onMount(async () => {
    // Ensure defaults are loaded (idempotent).
    await loadDefaultShortcuts();
    // Ensure shortcutsState is populated — normally filled by ShortcutDispatcher
    // on app init, but refresh defensively here so per-action pages work even
    // when the dispatcher hasn't run yet (e.g. deep-link, tests).
    if (shortcutsState.length === 0) {
      try {
        setShortcuts(await getRegisteredShortcuts());
      } catch {
        // Silent — editor button stays disabled when shortcut unavailable.
      }
    }
  });

  const currentShortcut = $derived(
    shortcutsState.find((s) => s.action === actionKey) ?? null,
  );

  function handleEdit(): void {
    if (!currentShortcut) return;
    editingDefault = getDefaultShortcut(currentShortcut.action);
    isEditorOpen = true;
  }

  async function handleSave(action: string, newShortcut: string): Promise<void> {
    await updateShortcut(action, newShortcut);
  }

  function handleEditorClose(): void {
    isEditorOpen = false;
    editingDefault = '';
  }
</script>

<SettingsSection title={t(`${translationNamespace}.section_shortcut`)}>
  <div class="shortcut-row">
    <span class="label">{t(`${translationNamespace}.shortcut_label`)}</span>
    {#if currentShortcut}
      <Kbd keys={currentShortcut.shortcut} />
    {/if}
    <Button
      variant="secondary"
      onclick={handleEdit}
      disabled={!currentShortcut}
    >
      {t(`${translationNamespace}.button_edit_shortcut`)}
    </Button>
  </div>
</SettingsSection>

<ShortcutEditorModal
  isOpen={isEditorOpen}
  shortcutInfo={currentShortcut}
  defaultShortcut={editingDefault}
  onSave={handleSave}
  onClose={handleEditorClose}
/>

<style>
  .shortcut-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    flex-wrap: wrap;
  }
  .label {
    color: var(--color-text-muted);
    font-size: 0.9rem;
  }
</style>
