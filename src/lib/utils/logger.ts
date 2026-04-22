/**
 * Logging utility for ShortCut
 * Logs to browser console, in-memory debug buffer, and (optionally) Rust backend.
 *
 * Backend IPC is gated on `appSettingsState.debugEnabled` so production builds
 * with debug mode off don't flood the backend log file. The in-memory buffer
 * always fills so the Debug page works locally regardless.
 *
 * Circular-import safety: `app-settings.svelte.ts` imports `log` from this
 * module but only invokes it inside function bodies. We import the
 * `appSettingsState` object lazily — both modules can load without executing
 * each other's body-scope code.
 */

import { frontendLog } from '$lib/api/tauri';
import { addLogEntry } from '$lib/state/debug.svelte';
import { appSettingsState } from '$lib/state/app-settings.svelte';

async function maybeForwardToBackend(message: string): Promise<void> {
  if (!appSettingsState.debugEnabled) return;
  await frontendLog(message);
}

/**
 * Log a message to console, debug buffer, and (if debug enabled) backend
 */
export async function log(message: string): Promise<void> {
  console.log(message);
  addLogEntry('info', message);
  await maybeForwardToBackend(message);
}

/**
 * Log an error to console, debug buffer, and (if debug enabled) backend
 */
export async function logError(message: string, error?: unknown): Promise<void> {
  const errorMessage = error instanceof Error ? error.message : String(error ?? '');
  const fullMessage = errorMessage ? `${message}: ${errorMessage}` : message;
  console.error(fullMessage);
  addLogEntry('error', fullMessage);
  await maybeForwardToBackend(`[ERROR] ${fullMessage}`);
}

/**
 * Log a warning to console, debug buffer, and (if debug enabled) backend
 */
export async function logWarn(message: string): Promise<void> {
  console.warn(message);
  addLogEntry('warn', message);
  await maybeForwardToBackend(`[WARN] ${message}`);
}
