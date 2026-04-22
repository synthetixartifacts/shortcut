/**
 * Action Menu Types
 *
 * Type definitions for the radial pie menu.
 */

import type { ShortcutAction } from '$lib/types';

/** A single menu item */
export interface MenuItem {
	id: string;
	label: string;
	icon: string;
	color: string;
	action: ShortcutAction;
}

