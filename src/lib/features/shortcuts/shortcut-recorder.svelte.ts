/**
 * Shortcut Recorder Logic
 *
 * Reactive state machine for recording keyboard shortcuts.
 * Extracted from ShortcutRecorder.svelte for reusability.
 */

import type { ModifierState, Platform } from './types';
import { keyEventToShortcut, isModifierKey, getModifierState } from './shortcut-parser';
import { shortcutToDisplay, buildLivePreview, getCurrentPlatform } from './shortcut-display';
import { isValidShortcut } from './shortcut-validation';

type RecorderMode = 'display' | 'recording';

interface ValidationResult {
  type: 'success' | 'error';
  text: string;
}

export function createShortcutRecorder() {
  // State
  let mode = $state<RecorderMode>('display');
  let recordedShortcut = $state<string | null>(null);
  let currentModifiers = $state<ModifierState>({
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
  });
  let currentKey = $state<string | null>(null);
  let platform = $state<Platform>('windows');
  let isActivelyRecording = $state(false);

  // Initialize platform
  getCurrentPlatform().then((p) => (platform = p));

  // Derived state
  const isRecording = $derived(mode === 'recording');

  const isValid = $derived(recordedShortcut !== null && isValidShortcut(recordedShortcut));

  const livePreview = $derived.by(() => {
    if (mode !== 'recording') return '';
    if (isActivelyRecording) {
      return buildLivePreview(currentModifiers, currentKey, platform);
    }
    if (recordedShortcut) {
      return shortcutToDisplay(recordedShortcut, platform);
    }
    return '';
  });

  const modifierCount = $derived.by(() => {
    if (!recordedShortcut) return 0;
    return recordedShortcut.split('+').length - 1;
  });

  const validationMessage = $derived.by((): ValidationResult | null => {
    if (!recordedShortcut) return null;

    if (isValid) {
      const keyCount = modifierCount + 1;
      return { type: 'success', text: `Valid ${keyCount}-key combination` };
    }

    const modCount = modifierCount;
    if (modCount < 1) {
      const modKey = platform === 'macos' ? 'Cmd' : 'Win';
      return {
        type: 'error',
        text: `Need at least 1 modifier (Ctrl, Alt, Shift, or ${modKey})`,
      };
    }

    if (modCount > 3) {
      return { type: 'error', text: 'Too many modifiers (max 3)' };
    }

    return { type: 'error', text: 'Invalid combination' };
  });

  // Actions
  function startRecording(): void {
    mode = 'recording';
    recordedShortcut = null;
    currentKey = null;
    currentModifiers = { ctrl: false, alt: false, shift: false, meta: false };
    isActivelyRecording = false;
  }

  function clearRecording(): void {
    recordedShortcut = null;
    currentKey = null;
    currentModifiers = { ctrl: false, alt: false, shift: false, meta: false };
    isActivelyRecording = false;
  }

  function cancelRecording(): void {
    mode = 'display';
    clearRecording();
  }

  function finishRecording(): void {
    mode = 'display';
    recordedShortcut = null;
    currentKey = null;
    currentModifiers = { ctrl: false, alt: false, shift: false, meta: false };
    isActivelyRecording = false;
  }

  function handleKeyDown(event: KeyboardEvent): void {
    if (mode !== 'recording') return;

    event.preventDefault();
    event.stopPropagation();

    isActivelyRecording = true;
    currentModifiers = getModifierState(event);

    if (!isModifierKey(event.code) && !isModifierKey(event.key)) {
      const shortcut = keyEventToShortcut(event);
      if (shortcut) {
        recordedShortcut = shortcut;
        currentKey = event.code.startsWith('Key')
          ? event.code.slice(3)
          : event.code.startsWith('Digit')
            ? event.code.slice(5)
            : event.code;
      }
    }
  }

  function handleKeyUp(event: KeyboardEvent): void {
    if (mode !== 'recording') return;

    event.preventDefault();
    event.stopPropagation();

    currentModifiers = getModifierState(event);

    if (
      !currentModifiers.ctrl &&
      !currentModifiers.alt &&
      !currentModifiers.shift &&
      !currentModifiers.meta
    ) {
      isActivelyRecording = false;
      currentKey = null;
    }
  }

  function handleBlur(): void {
    // Don't cancel, just stop active recording
    isActivelyRecording = false;
  }

  return {
    // State (getters for reactivity)
    get mode() {
      return mode;
    },
    get isRecording() {
      return isRecording;
    },
    get recordedShortcut() {
      return recordedShortcut;
    },
    get isValid() {
      return isValid;
    },
    get isActivelyRecording() {
      return isActivelyRecording;
    },
    get livePreview() {
      return livePreview;
    },
    get validationMessage() {
      return validationMessage;
    },
    get platform() {
      return platform;
    },

    // Actions
    startRecording,
    clearRecording,
    cancelRecording,
    finishRecording,
    handleKeyDown,
    handleKeyUp,
    handleBlur,
  };
}

export type ShortcutRecorderState = ReturnType<typeof createShortcutRecorder>;
