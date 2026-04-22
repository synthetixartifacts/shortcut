/**
 * Global Application State
 *
 * Uses Svelte 5 runes for reactive state management.
 * This is the single source of truth for app-wide state.
 */

/**
 * Core application state - reactive with $state rune.
 *
 * Kept minimal — only fields actually read by UI. Status messages, last
 * transcription, and live-buffer previews are driven by the activity indicator
 * + controllers directly and do not live here.
 */
export const appState = $state({
  /** Whether dictation is currently recording */
  isRecording: false,
});

/**
 * Reset recording-related state
 */
export function resetRecordingState(): void {
  appState.isRecording = false;
}

/**
 * Set recording state
 */
export function setRecording(isRecording: boolean): void {
  appState.isRecording = isRecording;
}
