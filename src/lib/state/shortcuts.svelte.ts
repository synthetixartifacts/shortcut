/**
 * Shortcuts State
 *
 * Manages keyboard shortcut registry, user preferences, and updates.
 */

import type { ShortcutInfo, HotkeyConfig } from '$lib/types';
import {
  getRegisteredShortcuts,
  updateShortcuts as updateShortcutsApi,
  getDefaultShortcuts as getDefaultShortcutsApi,
  updateHotkeysConfig,
} from '$lib/api/tauri';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

/** Canonical action-key union for per-row shortcut save-status indicators. */
type ShortcutActionKey =
  | 'dictation'
  | 'grammar'
  | 'translate'
  | 'improve'
  | 'open_menu'
  | 'screen_question';

const SHORTCUT_ACTION_KEYS: readonly ShortcutActionKey[] = [
  'dictation',
  'grammar',
  'translate',
  'improve',
  'open_menu',
  'screen_question',
];

function buildShortcutsSaveStatus(): Record<ShortcutActionKey, SaveStatus> {
  const record = {} as Record<ShortcutActionKey, SaveStatus>;
  for (const key of SHORTCUT_ACTION_KEYS) record[key] = createSaveStatus();
  return record;
}

/**
 * Per-row save-status record for shortcuts. Keys mirror the action names
 * used throughout the app; each row's indicator flips via `updateShortcut`.
 */
export const shortcutsSaveStatus = $state<Record<ShortcutActionKey, SaveStatus>>(
  buildShortcutsSaveStatus(),
);

function isShortcutActionKey(action: string): action is ShortcutActionKey {
  return (SHORTCUT_ACTION_KEYS as readonly string[]).includes(action);
}

/**
 * Registered shortcuts - reactive with $state rune
 */
export const shortcutsState = $state<ShortcutInfo[]>([]);

/**
 * Default shortcuts for reference
 */
const defaultShortcuts = $state<HotkeyConfig>({
  dictation: 'Ctrl+Shift+D',
  grammar: 'Ctrl+Shift+G',
  translate: 'Ctrl+Shift+T',
  improve: 'Ctrl+Shift+I',
  open_menu: 'Ctrl+Shift+J',
  screen_question: 'Ctrl+Shift+S',
});

/**
 * Loading/error state for shortcut operations
 */
const shortcutsMetaState = $state<{
  isUpdating: boolean;
  error: string | null;
}>({
  isUpdating: false,
  error: null,
});

/**
 * Set shortcuts from backend
 */
export function setShortcuts(shortcuts: ShortcutInfo[]): void {
  shortcutsState.length = 0;
  shortcutsState.push(...shortcuts);
}

/**
 * Load default shortcuts from backend
 */
export async function loadDefaultShortcuts(): Promise<void> {
  try {
    const defaults = await getDefaultShortcutsApi();
    defaultShortcuts.dictation = defaults.dictation;
    defaultShortcuts.grammar = defaults.grammar;
    defaultShortcuts.translate = defaults.translate;
    defaultShortcuts.improve = defaults.improve;
    defaultShortcuts.open_menu = defaults.open_menu;
    defaultShortcuts.screen_question = defaults.screen_question;
    await log('[ShortcutsState] Loaded default shortcuts');
  } catch (e) {
    console.error('[ShortcutsState] Failed to load default shortcuts:', e);
  }
}

/**
 * Refresh shortcuts from backend
 */
async function refreshShortcuts(): Promise<void> {
  try {
    const shortcuts = await getRegisteredShortcuts();
    setShortcuts(shortcuts);
    await log('[ShortcutsState] Refreshed shortcuts from backend');
  } catch (e) {
    console.error('[ShortcutsState] Failed to refresh shortcuts:', e);
  }
}

/**
 * Get current hotkeys from registered shortcuts state
 */
function getCurrentHotkeys(): HotkeyConfig {
  const hotkeys: HotkeyConfig = {
    dictation: defaultShortcuts.dictation,
    grammar: defaultShortcuts.grammar,
    translate: defaultShortcuts.translate,
    improve: defaultShortcuts.improve,
    open_menu: defaultShortcuts.open_menu,
    screen_question: defaultShortcuts.screen_question,
  };
  for (const s of shortcutsState) {
    if (s.action in hotkeys) {
      (hotkeys as unknown as Record<string, string>)[s.action] = s.shortcut;
    }
  }
  return hotkeys;
}

/**
 * Get the default shortcut for a specific action
 */
export function getDefaultShortcut(action: string): string {
  switch (action) {
    case 'dictation':
      return defaultShortcuts.dictation;
    case 'grammar':
      return defaultShortcuts.grammar;
    case 'translate':
      return defaultShortcuts.translate;
    case 'improve':
      return defaultShortcuts.improve;
    case 'open_menu':
      return defaultShortcuts.open_menu;
    case 'screen_question':
      return defaultShortcuts.screen_question;
    default:
      return '';
  }
}

/**
 * Update a single shortcut.
 *
 * When `action` matches a known `ShortcutActionKey`, the corresponding
 * `shortcutsSaveStatus[action]` entry flips through `saving → saved`
 * (or `error`). Unknown actions are still persisted, but no indicator flips.
 */
export async function updateShortcut(action: string, newShortcut: string): Promise<void> {
  const key: ShortcutActionKey | null = isShortcutActionKey(action) ? action : null;
  await withAsyncState(shortcutsMetaState, async () => {
    // Build from current registered shortcuts
    const current = getCurrentHotkeys();
    const newHotkeys: HotkeyConfig = {
      ...current,
      [action]: newShortcut,
    };

    await updateShortcutsApi(newHotkeys);
    await updateHotkeysConfig(newHotkeys);

    await refreshShortcuts();
    await log(`[ShortcutsState] Updated ${action} shortcut to ${newShortcut}`);
  }, {
    loadingKey: 'isUpdating',
    rethrow: true,
    errorFallback: 'Failed to update shortcut',
    onSaving: key ? () => shortcutsSaveStatus[key].markSaving() : undefined,
    onSaved: key ? () => shortcutsSaveStatus[key].markSaved() : undefined,
    onError: key ? (m) => shortcutsSaveStatus[key].markError(m) : undefined,
  });
}

/**
 * Reset all shortcuts to defaults.
 *
 * Every row's `saveStatus` flips through `saving → saved` together so the
 * list visibly confirms the bulk reset after the single backend write.
 */
export async function resetAllShortcuts(): Promise<void> {
  await withAsyncState(shortcutsMetaState, async () => {
    const newHotkeys: HotkeyConfig = {
      dictation: defaultShortcuts.dictation,
      grammar: defaultShortcuts.grammar,
      translate: defaultShortcuts.translate,
      improve: defaultShortcuts.improve,
      open_menu: defaultShortcuts.open_menu,
      screen_question: defaultShortcuts.screen_question,
    };

    await updateShortcutsApi(newHotkeys);
    await updateHotkeysConfig(newHotkeys);
    await refreshShortcuts();

    await log('[ShortcutsState] Reset all shortcuts to defaults');
  }, {
    loadingKey: 'isUpdating',
    rethrow: true,
    errorFallback: 'Failed to reset shortcuts',
    onSaving: () => { for (const k of SHORTCUT_ACTION_KEYS) shortcutsSaveStatus[k].markSaving(); },
    onSaved: () => { for (const k of SHORTCUT_ACTION_KEYS) shortcutsSaveStatus[k].markSaved(); },
    onError: (m) => { for (const k of SHORTCUT_ACTION_KEYS) shortcutsSaveStatus[k].markError(m); },
  });
}

