/**
 * Action Menu Controller
 *
 * Handles action selection from the pie menu.
 * Emits event to main window and hides the menu window.
 */

import { emit } from '@tauri-apps/api/event';
import { hideActionMenu } from '$lib/api/tauri';
import { log, logError } from '$lib/utils/logger';
import type { ShortcutAction } from '$lib/types';

/**
 * Handle action selection from the pie menu.
 * Emits event to main window and hides the menu.
 * Empty string means dismissal without selection.
 */
export async function selectAction(action: ShortcutAction | ''): Promise<void> {
	// If action is empty, just hide (dismiss without executing)
	if (action) {
		await log(`[ActionMenu] Action selected: ${action}`);
		await emit('menu-action-selected', { action });
	} else {
		await log('[ActionMenu] Menu dismissed');
	}

	try {
		await hideActionMenu();
	} catch (e) {
		await logError('[ActionMenu] Failed to hide menu', e);
	}
}
