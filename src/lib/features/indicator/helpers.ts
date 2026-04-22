/**
 * Indicator Helper Functions
 *
 * Convenience functions for common activity patterns.
 * Provides DRY integration with the indicator system.
 */

import { startActivity, updateActivity, endActivity } from '$lib/state/activity.svelte';
import type { ActivityType } from './types';

/**
 * Execute an async operation with indicator
 *
 * Shows indicator during operation, handles success/error automatically.
 * The indicator auto-hides after showing success/error state.
 *
 * @param type - The activity type (dictation, grammar, translate, processing)
 * @param operation - Async function to execute
 * @param options - Optional success/error messages
 * @returns The result of the operation
 * @throws Re-throws any error from the operation
 *
 * @example
 * await withIndicator('grammar', async () => {
 *   const text = await getSelectedText();
 *   const fixed = await transformText('grammar', text);
 *   await pasteText(fixed);
 * });
 *
 * @example
 * const result = await withIndicator('translate', async () => {
 *   return await transformText('translate', text);
 * }, {
 *   successMessage: 'Translated!',
 *   errorMessage: 'Translation failed'
 * });
 */
export async function withIndicator<T>(
  type: ActivityType,
  operation: () => Promise<T>,
  options?: {
    successMessage?: string;
    errorMessage?: string;
  }
): Promise<T> {
  await startActivity(type);

  try {
    const result = await operation();
    await updateActivity('success', options?.successMessage);
    return result;
  } catch (e) {
    const message =
      options?.errorMessage || (e instanceof Error ? e.message : 'Failed');
    await updateActivity('error', message);
    throw e;
  }
}

/**
 * Start indicator for manual control
 *
 * Use when you need fine-grained control over the indicator lifecycle.
 * You are responsible for calling endActivity() or updateActivity().
 *
 * @param type - The activity type
 * @param message - Optional status message
 *
 * @example
 * await startIndicator('dictation');
 * // ... do recording ...
 * await endIndicator();
 */
export async function startIndicator(
  type: ActivityType,
  message?: string
): Promise<void> {
  await startActivity(type, message);
}

/**
 * Mark current activity as successful
 *
 * Shows success state and auto-hides after delay.
 *
 * @param message - Optional success message
 */
export async function indicatorSuccess(message?: string): Promise<void> {
  await updateActivity('success', message);
}

/**
 * Mark current activity as failed
 *
 * Shows error state and auto-hides after delay.
 *
 * @param message - Optional error message
 */
export async function indicatorError(message?: string): Promise<void> {
  await updateActivity('error', message);
}
