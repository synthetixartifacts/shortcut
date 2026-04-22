/**
 * Activity State Management
 *
 * Centralized state for tracking current activity.
 * Broadcasts changes to indicator window via Tauri events.
 */

import { emit } from '@tauri-apps/api/event';
import { log, logError } from '$lib/utils/logger';
import type { ActivityType, ActivityState, ActivityInfo } from '$lib/features/indicator';
import { showIndicator, hideIndicator } from '$lib/api/tauri';
import { ANIMATION } from '$lib/features/indicator';

/**
 * Activity state - reactive with $state rune
 */
export const activityState = $state<ActivityInfo>({
  type: 'processing',
  state: 'idle',
  message: undefined,
});

// Auto-hide timeout
let hideTimeout: ReturnType<typeof setTimeout> | null = null;

/**
 * Clear any pending hide timeout
 */
function clearHideTimeout(): void {
  if (hideTimeout) {
    clearTimeout(hideTimeout);
    hideTimeout = null;
  }
}

/**
 * Schedule auto-hide after delay
 */
function scheduleHide(delay: number): void {
  clearHideTimeout();
  hideTimeout = setTimeout(() => {
    endActivity();
  }, delay);
}

/**
 * Broadcast state to indicator window
 */
async function broadcastState(): Promise<void> {
  try {
    await emit('indicator-update', {
      type: activityState.type,
      state: activityState.state,
      message: activityState.message,
    });
  } catch (e) {
    // Indicator window might not be open
    console.debug('[Activity] Failed to broadcast:', e);
  }
}

/**
 * Start an activity - shows indicator
 */
export async function startActivity(
  type: ActivityType,
  message?: string
): Promise<void> {
  log(`[Activity] Starting: ${type}`);

  clearHideTimeout();

  activityState.type = type;
  activityState.state = 'active';
  activityState.message = message;

  // Show indicator window
  try {
    await showIndicator();
  } catch (e) {
    logError('[Activity] Failed to show indicator', e);
  }

  await broadcastState();
}

/**
 * Update activity state (success/error)
 */
export async function updateActivity(
  state: ActivityState,
  message?: string
): Promise<void> {
  log(`[Activity] Updating to: ${state}`);

  activityState.state = state;
  if (message !== undefined) {
    activityState.message = message;
  }

  await broadcastState();

  // Schedule auto-hide for terminal states
  if (state === 'success') {
    scheduleHide(ANIMATION.SUCCESS_DISPLAY_TIME);
  } else if (state === 'error') {
    scheduleHide(ANIMATION.ERROR_DISPLAY_TIME);
  }
}

/**
 * End activity - hides indicator
 */
export async function endActivity(): Promise<void> {
  log('[Activity] Ending');

  clearHideTimeout();

  activityState.state = 'idle';
  activityState.message = undefined;

  // Hide indicator window
  try {
    await hideIndicator();
  } catch (e) {
    logError('[Activity] Failed to hide indicator', e);
  }
}

