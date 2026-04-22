import { invoke } from '@tauri-apps/api/core';

export async function invokeWithErrorHandling<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    console.error(`[Tauri] ${cmd} failed:`, message);
    throw new Error(`${cmd}: ${message}`);
  }
}

/**
 * Escape hatch for commands whose name is resolved dynamically at runtime
 * (e.g. overlay-chat feeds different `sendCommand` names per consumer).
 *
 * Prefer the per-domain wrappers in `api/*.ts` — use this only when the
 * command name genuinely isn't known at import time. Error handling mirrors
 * `invokeWithErrorHandling` so dynamic callers get the same reporting.
 */
export async function invokeDynamic<T>(
  command: string,
  args: Record<string, unknown>
): Promise<T> {
  return invokeWithErrorHandling<T>(command, args);
}

export async function frontendLog(message: string): Promise<void> {
  try {
    await invoke('frontend_log', { message });
  } catch {
    // Ignore if command not available
  }
}
