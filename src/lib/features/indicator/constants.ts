/**
 * Indicator Constants
 *
 * Configuration values for the floating indicator window.
 */

import type { ActivityType, ActivityColors } from './types';

/** Animation timing (ms) */
export const ANIMATION = {
  DOT_PULSE_DURATION: 1500,
  DOT_STAGGER_DELAY: 200,
  FADE_IN_DURATION: 200,
  FADE_OUT_DURATION: 150,
  SUCCESS_DISPLAY_TIME: 1000,
  ERROR_DISPLAY_TIME: 2000,
} as const;

/** Color schemes per activity type */
export const ACTIVITY_COLORS: Record<ActivityType, ActivityColors> = {
  dictation: {
    primary: '#ef4444', // Red for recording
    secondary: '#fca5a5',
    background: 'rgba(127, 29, 29, 0.9)',
  },
  grammar: {
    primary: '#3b82f6', // Blue
    secondary: '#93c5fd',
    background: 'rgba(30, 58, 138, 0.9)',
  },
  translate: {
    primary: '#8b5cf6', // Purple
    secondary: '#c4b5fd',
    background: 'rgba(76, 29, 149, 0.9)',
  },
  improve: {
    primary: '#10b981', // Teal
    secondary: '#6ee7b7',
    background: 'rgba(5, 46, 22, 0.9)',
  },
  processing: {
    primary: '#6b7280', // Gray
    secondary: '#9ca3af',
    background: 'rgba(31, 41, 55, 0.9)',
  },
};

/** Labels for activities */
export const ACTIVITY_LABELS: Record<ActivityType, string> = {
  dictation: 'Recording',
  grammar: 'Fixing',
  translate: 'Translating',
  improve: 'Improving',
  processing: 'Processing',
};
