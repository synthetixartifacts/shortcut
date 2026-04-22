/**
 * Shortcut Validation
 *
 * Validates keyboard shortcut strings.
 */

import type { ModifierState } from './types';
import { parseShortcut } from './shortcut-parser';

/**
 * Count active modifiers
 */
function countModifiers(modifiers: ModifierState): number {
  return (
    (modifiers.ctrl ? 1 : 0) +
    (modifiers.alt ? 1 : 0) +
    (modifiers.shift ? 1 : 0) +
    (modifiers.meta ? 1 : 0)
  );
}

/**
 * Validate a shortcut string
 * Requirements:
 * - 1-3 modifiers (min 1, max 3)
 * - Exactly one non-modifier key
 * - Total: 2-4 keys
 */
export function isValidShortcut(shortcut: string): boolean {
  const parsed = parseShortcut(shortcut);
  if (!parsed) return false;

  // Must have 1-3 modifiers
  const modifierCount = countModifiers(parsed.modifiers);
  if (modifierCount < 1 || modifierCount > 3) return false;

  // Must have a non-modifier key
  if (!parsed.key) return false;

  return true;
}
