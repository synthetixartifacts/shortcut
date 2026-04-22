/**
 * Shortcut Event Dispatcher
 *
 * Listens to Tauri shortcut events and dispatches to appropriate handlers.
 * Follows the Open/Closed principle - easy to add new shortcuts without
 * modifying existing code.
 *
 * Includes debouncing to prevent double-fire issues from keyboards,
 * accessibility software, or OS-level event duplication.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { log, logError, logWarn } from '$lib/utils/logger';
import { TRAILING_SPEECH_DELAY_MS } from './constants';
import { getRegisteredShortcuts, toggleActionMenu, hideActionMenu, hideScreenQuestion, screenQuestion } from '$lib/api/tauri';
import { setShortcuts } from '$lib/state/shortcuts.svelte';
import { getDictationController } from '$lib/features/dictation';
import { handleGrammarFix } from '$lib/features/grammar';
import { handleTranslation } from '$lib/features/translation';
import { handleImproveText } from '$lib/features/improve';
import type { ShortcutAction } from '$lib/types';

/**
 * Debounce configuration
 */
const DEBOUNCE_MS = 100; // Minimum time between same action events

/**
 * Shortcut handler function type
 */
type ShortcutHandler = () => Promise<void>;

/**
 * Registry of shortcut action handlers
 *
 * To add a new shortcut:
 * 1. Add the action type to ShortcutAction in types/index.ts
 * 2. Create a handler function in the appropriate feature module
 * 3. Add an entry to this registry
 */
const shortcutHandlers: Record<ShortcutAction, ShortcutHandler> = {
  dictation_start: async () => {
    const controller = getDictationController();
    if (!controller.isRecording) {
      await controller.startRecording();
    }
  },

  dictation_stop: async () => {
    const controller = getDictationController();
    if (controller.isRecording) {
      // Wait before stopping to capture trailing speech
      await new Promise((resolve) => setTimeout(resolve, TRAILING_SPEECH_DELAY_MS));
      // Check again in case recording was cancelled during the delay
      if (controller.isRecording) {
        await controller.stopRecording();
      }
    }
  },

  dictation: async () => {
    // Legacy toggle mode
    const controller = getDictationController();
    await controller.toggleRecording();
  },

  grammar: handleGrammarFix,

  translate: handleTranslation,

  improve: handleImproveText,

  open_menu: async () => {
    await toggleActionMenu();
  },

  screen_question: async () => {
    await screenQuestion();
  },
};

/**
 * Shortcut Dispatcher
 *
 * Manages the lifecycle of shortcut event listening and dispatching.
 */
class ShortcutDispatcher {
  private unlistenFn: UnlistenFn | null = null;
  private unlistenMenuFn: UnlistenFn | null = null;
  private initialized = false;
  private lastEventTimes: Map<ShortcutAction, number> = new Map();

  /**
   * Initialize the shortcut dispatcher
   *
   * Loads registered shortcuts and starts listening for events.
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    await log('[ShortcutDispatcher] Initializing...');

    // Load registered shortcuts from backend
    try {
      const shortcuts = await getRegisteredShortcuts();
      setShortcuts(shortcuts);
      await log(`[ShortcutDispatcher] Loaded ${shortcuts.length} shortcuts`);
    } catch (e) {
      logError('[ShortcutDispatcher] Failed to load shortcuts', e);
    }

    // Listen for shortcut events from Rust
    this.unlistenFn = await listen<ShortcutAction>(
      'shortcut-triggered',
      async (event) => {
        const action = event.payload;
        await this.dispatch(action);
      }
    );

    // Listen for action selections from the action menu window
    this.unlistenMenuFn = await listen<{ action: ShortcutAction }>(
      'menu-action-selected',
      async (event) => {
        const action = event.payload.action;
        await log(`[ShortcutDispatcher] Menu action selected: ${action}`);
        // Small delay to let the menu window hide before indicator shows
        await new Promise(resolve => setTimeout(resolve, 50));
        await this.dispatch(action);
      }
    );

    this.initialized = true;
    await log('[ShortcutDispatcher] Initialized successfully');
  }

  /**
   * Check if an action should be debounced (fired too recently)
   */
  private shouldDebounce(action: ShortcutAction): boolean {
    const now = Date.now();
    const lastTime = this.lastEventTimes.get(action) || 0;
    const timeSinceLast = now - lastTime;

    if (timeSinceLast < DEBOUNCE_MS) {
      return true;
    }

    this.lastEventTimes.set(action, now);
    return false;
  }

  /**
   * Dispatch a shortcut action to its handler
   */
  async dispatch(action: ShortcutAction): Promise<void> {
    // Debounce: prevent duplicate events firing too quickly
    if (this.shouldDebounce(action)) {
      log(`[ShortcutDispatcher] Debounced duplicate: ${action}`);
      return;
    }

    // If a direct shortcut fires while menu/overlay is visible, hide them first
    // hideActionMenu/hideScreenQuestion are no-ops if the window is already hidden
    if (action !== 'open_menu' && action !== 'screen_question') {
      try {
        await hideActionMenu();
      } catch {
        // Menu might not be visible, ignore
      }
      try {
        await hideScreenQuestion();
      } catch {
        // Screen question might not be visible, ignore
      }
    }

    log(`[ShortcutDispatcher] Dispatching: ${action}`);

    const handler = shortcutHandlers[action];
    if (handler) {
      try {
        await handler();
      } catch (e) {
        logError(`[ShortcutDispatcher] Handler error for ${action}`, e);
      }
    } else {
      logWarn(`[ShortcutDispatcher] No handler for action: ${action}`);
    }
  }

  /**
   * Clean up event listeners
   */
  cleanup(): void {
    if (this.unlistenFn) {
      this.unlistenFn();
      this.unlistenFn = null;
    }
    this.unlistenMenuFn?.();
    this.unlistenMenuFn = null;
    this.initialized = false;
    this.lastEventTimes.clear();
    log('[ShortcutDispatcher] Cleaned up');
  }
}

/**
 * Singleton instance
 */
let shortcutDispatcherInstance: ShortcutDispatcher | null = null;

/**
 * Get or create the shortcut dispatcher singleton
 */
export function getShortcutDispatcher(): ShortcutDispatcher {
  if (!shortcutDispatcherInstance) {
    shortcutDispatcherInstance = new ShortcutDispatcher();
  }
  return shortcutDispatcherInstance;
}
