/**
 * Action Menu Constants
 *
 * Configuration values for the radial pie menu.
 * Colors match indicator activity colors for visual consistency.
 */

import type { MenuItem } from './types';

/** Menu items for Phase 1 (flat, 4 items) */
export const MENU_ITEMS: MenuItem[] = [
	{
		id: 'dictation',
		label: 'action_menu.item_dictation',
		icon: '\u{1F3A4}',
		color: '#ef4444',
		action: 'dictation_start'
	},
	{
		id: 'grammar',
		label: 'action_menu.item_grammar',
		icon: '\u{1F4DD}',
		color: '#3b82f6',
		action: 'grammar'
	},
	{
		id: 'translate',
		label: 'action_menu.item_translate',
		icon: '\u{1F310}',
		color: '#8b5cf6',
		action: 'translate'
	},
	{
		id: 'improve',
		label: 'action_menu.item_improve',
		icon: '\u{2728}',
		color: '#10b981',
		action: 'improve'
	},
	{
		id: 'screen_question',
		label: 'action_menu.item_screen_question',
		icon: '\u{1F4F7}',
		color: '#f59e0b',
		action: 'screen_question'
	}
];

/** Menu dimensions (px) */
export const MENU_SIZE = 280;
export const OUTER_RADIUS = 130;
export const INNER_RADIUS = 35;

/** Auto-dismiss timeout (ms) */
export const AUTO_DISMISS_MS = 5000;
