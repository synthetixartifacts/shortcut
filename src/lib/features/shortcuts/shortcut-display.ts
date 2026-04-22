/**
 * Shortcut Display
 *
 * Platform-specific display formatting for keyboard shortcuts.
 */

import type { ModifierState, Platform } from './types';
import { parseShortcut } from './shortcut-parser';

/**
 * Get current platform
 *
 * Uses navigator.userAgent heuristic (matches the pattern used by
 * engine.svelte.ts). Avoids the @tauri-apps/plugin-os dep since this
 * is the only non-test consumer and the heuristic is accurate enough
 * for shortcut-display formatting.
 */
export async function getCurrentPlatform(): Promise<Platform> {
  if (typeof navigator !== 'undefined') {
    const ua = navigator.userAgent.toLowerCase();
    if (ua.includes('mac')) return 'macos';
    if (ua.includes('win')) return 'windows';
  }
  return 'linux';
}

/**
 * Get platform-specific display labels for modifiers
 */
function getModifierLabels(plat: Platform): Record<string, string> {
  switch (plat) {
    case 'macos':
      return {
        meta: 'Cmd',
        ctrl: 'Ctrl',
        alt: 'Option',
        shift: 'Shift',
      };
    case 'windows':
      return {
        meta: 'Win',
        ctrl: 'Ctrl',
        alt: 'Alt',
        shift: 'Shift',
      };
    default:
      return {
        meta: 'Super',
        ctrl: 'Ctrl',
        alt: 'Alt',
        shift: 'Shift',
      };
  }
}

/**
 * Convert a shortcut string to platform-specific display format
 */
export function shortcutToDisplay(shortcut: string, plat: Platform): string {
  const parsed = parseShortcut(shortcut);
  if (!parsed) return shortcut;

  const labels = getModifierLabels(plat);
  const parts: string[] = [];

  // Order: Meta, Ctrl, Alt, Shift
  if (parsed.modifiers.meta) parts.push(labels.meta);
  if (parsed.modifiers.ctrl) parts.push(labels.ctrl);
  if (parsed.modifiers.alt) parts.push(labels.alt);
  if (parsed.modifiers.shift) parts.push(labels.shift);

  // Add the key
  parts.push(parsed.key);

  return parts.join('+');
}

/**
 * Get modifier state for live display during recording
 */
function modifiersToDisplayParts(modifiers: ModifierState, plat: Platform): string[] {
  const labels = getModifierLabels(plat);
  const parts: string[] = [];

  if (modifiers.meta) parts.push(labels.meta);
  if (modifiers.ctrl) parts.push(labels.ctrl);
  if (modifiers.alt) parts.push(labels.alt);
  if (modifiers.shift) parts.push(labels.shift);

  return parts;
}

/**
 * Build live preview string while recording
 */
export function buildLivePreview(
  modifiers: ModifierState,
  key: string | null,
  plat: Platform
): string {
  const parts = modifiersToDisplayParts(modifiers, plat);

  if (key) {
    parts.push(key);
  } else if (parts.length > 0) {
    parts.push('...');
  }

  return parts.join('+') || 'Press keys...';
}
