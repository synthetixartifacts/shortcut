import { invokeWithErrorHandling } from './core';

export async function showIndicator(): Promise<void> {
  await invokeWithErrorHandling<void>('show_indicator');
}

export async function hideIndicator(): Promise<void> {
  await invokeWithErrorHandling<void>('hide_indicator');
}

/**
 * Force-reset the indicator window (destroys and recreates).
 * Used when indicator stops appearing after display changes.
 */
export async function resetIndicator(): Promise<void> {
  await invokeWithErrorHandling<void>('reset_indicator');
}

export async function toggleActionMenu(): Promise<void> {
  await invokeWithErrorHandling<void>('toggle_action_menu');
}

export async function hideActionMenu(): Promise<void> {
  await invokeWithErrorHandling<void>('hide_action_menu');
}

export async function screenQuestion(): Promise<void> {
  await invokeWithErrorHandling<void>('screen_question');
}

export async function hideScreenQuestion(): Promise<void> {
  await invokeWithErrorHandling<void>('hide_screen_question');
}
