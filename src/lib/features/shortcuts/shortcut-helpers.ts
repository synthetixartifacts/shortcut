/**
 * Shortcut Display Helpers
 *
 * Shared utilities for resolving shortcut display strings.
 * Used by Dashboard and Actions pages.
 */

import { shortcutsState } from '$lib/state/shortcuts.svelte';
import { shortcutToDisplay } from './shortcut-display';
import type { Platform } from './types';

/**
 * Get the display string for a shortcut action on the given platform.
 */
export function getShortcutDisplay(action: string, platform: Platform): string {
  const info = shortcutsState.find(s => s.action === action);
  return info ? shortcutToDisplay(info.shortcut, platform) : '';
}
