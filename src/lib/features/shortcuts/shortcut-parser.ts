/**
 * Shortcut Parser
 *
 * Converts between keyboard events and shortcut strings.
 */

import type { ModifierState, ShortcutDetails } from './types';

/**
 * List of modifier key codes
 */
const MODIFIER_KEYS = new Set([
  'Control',
  'ControlLeft',
  'ControlRight',
  'Alt',
  'AltLeft',
  'AltRight',
  'Shift',
  'ShiftLeft',
  'ShiftRight',
  'Meta',
  'MetaLeft',
  'MetaRight',
  'OS',
  'OSLeft',
  'OSRight',
]);

/**
 * Check if a key is a modifier key
 */
export function isModifierKey(key: string): boolean {
  return MODIFIER_KEYS.has(key);
}

/**
 * Extract modifier state from a keyboard event
 */
export function getModifierState(event: KeyboardEvent): ModifierState {
  return {
    ctrl: event.ctrlKey,
    alt: event.altKey,
    shift: event.shiftKey,
    meta: event.metaKey,
  };
}

/**
 * Convert keyboard event key code to standard key name
 */
function normalizeKeyCode(code: string): string {
  // Remove "Key" prefix for letters
  if (code.startsWith('Key')) {
    return code.slice(3);
  }

  // Remove "Digit" prefix for numbers
  if (code.startsWith('Digit')) {
    return code.slice(5);
  }

  // Map special keys
  const keyMap: Record<string, string> = {
    Space: 'Space',
    Enter: 'Enter',
    Tab: 'Tab',
    Backspace: 'Backspace',
    Delete: 'Delete',
    Escape: 'Escape',
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
    Home: 'Home',
    End: 'End',
    PageUp: 'PageUp',
    PageDown: 'PageDown',
    Insert: 'Insert',
  };

  // Handle function keys
  if (code.startsWith('F') && /^F\d+$/.test(code)) {
    return code;
  }

  return keyMap[code] || code;
}

/**
 * Convert a keyboard event to a Tauri shortcut string
 * Returns null if the event doesn't form a valid shortcut
 */
export function keyEventToShortcut(event: KeyboardEvent): string | null {
  const modifiers = getModifierState(event);

  // Ignore if only modifier keys are pressed
  if (isModifierKey(event.code) || isModifierKey(event.key)) {
    return null;
  }

  // Must have at least one modifier
  if (!modifiers.ctrl && !modifiers.alt && !modifiers.shift && !modifiers.meta) {
    return null;
  }

  const key = normalizeKeyCode(event.code);
  if (!key || key === 'Unknown') {
    return null;
  }

  // Build shortcut string
  const parts: string[] = [];

  // Order: Super/Meta, Ctrl, Alt, Shift (consistent ordering)
  if (modifiers.meta) parts.push('Super');
  if (modifiers.ctrl) parts.push('Ctrl');
  if (modifiers.alt) parts.push('Alt');
  if (modifiers.shift) parts.push('Shift');

  parts.push(key);

  return parts.join('+');
}

/**
 * Parse a shortcut string into its components
 */
export function parseShortcut(shortcut: string): ShortcutDetails | null {
  if (!shortcut) return null;

  const parts = shortcut.split('+').map((s) => s.trim());
  if (parts.length === 0) return null;

  const modifiers: ModifierState = {
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
  };

  let key = '';
  let code = '';

  for (const part of parts) {
    const partLower = part.toLowerCase();
    switch (partLower) {
      case 'control':
      case 'ctrl':
        modifiers.ctrl = true;
        break;
      case 'alt':
      case 'option':
        modifiers.alt = true;
        break;
      case 'shift':
        modifiers.shift = true;
        break;
      case 'super':
      case 'meta':
      case 'command':
      case 'cmd':
      case 'win':
        modifiers.meta = true;
        break;
      default:
        key = part;
        code = part;
        break;
    }
  }

  if (!key) return null;

  return { modifiers, key, code };
}
