/**
 * Activity Indicator Types
 *
 * Type definitions for the floating indicator window.
 */

/** Activity types that trigger the indicator */
export type ActivityType =
  | 'dictation' // Voice recording
  | 'grammar' // Grammar fixing
  | 'translate' // Translation
  | 'improve' // Text improvement (MAI)
  | 'processing'; // Generic processing

/** Activity state */
export type ActivityState =
  | 'idle' // No activity
  | 'preparing' // Setting up (e.g., acquiring microphone)
  | 'active' // Activity in progress
  | 'success' // Completed successfully
  | 'error'; // Failed

/** Activity info for display */
export interface ActivityInfo {
  type: ActivityType;
  state: ActivityState;
  message?: string;
}

/** Color scheme for activity types */
export interface ActivityColors {
  primary: string;
  secondary: string;
  background: string;
}
