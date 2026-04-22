/**
 * Shortcut Types
 *
 * Type definitions for keyboard shortcut handling.
 */

/**
 * Modifier key state from keyboard event
 */
export interface ModifierState {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
}

/**
 * Parsed shortcut details
 */
export interface ShortcutDetails {
  modifiers: ModifierState;
  key: string;
  code: string;
}

/**
 * Platform type for display formatting
 */
export type Platform = 'windows' | 'macos' | 'linux';
