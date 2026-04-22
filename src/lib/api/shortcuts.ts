import { invokeWithErrorHandling } from './core';
import type { ShortcutInfo, HotkeyConfig } from '$lib/types';

export async function getRegisteredShortcuts(): Promise<ShortcutInfo[]> {
  return invokeWithErrorHandling<ShortcutInfo[]>('get_registered_shortcuts');
}

export async function updateShortcuts(hotkeys: HotkeyConfig): Promise<void> {
  await invokeWithErrorHandling<void>('update_shortcuts', { hotkeys });
}

export async function getDefaultShortcuts(): Promise<HotkeyConfig> {
  return invokeWithErrorHandling<HotkeyConfig>('get_default_shortcuts');
}
