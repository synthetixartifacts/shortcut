/**
 * Save Status primitive for unified per-field save feedback.
 *
 * `createSaveStatus()` returns a reactive `$state` proxy with four transition
 * methods. Each instance owns a single `setTimeout` handle cleared before any
 * new mutation — no leaks, no stale transitions. `markError` does not schedule
 * a revert; the error sticks until the next `markSaving` / `markSaved` call.
 */

/** Debounce window for auto-save flows (ms). */
export const SAVE_DEBOUNCE_MS = 500;

/** How long the "saved" confirmation lingers before reverting to idle (ms). */
const SAVED_LINGER_MS = 2000;

/** Discrete status values the indicator transitions between. */
type SaveStatusValue = 'idle' | 'saving' | 'saved' | 'error';

/** Create a fresh reactive save-status slot. Return value is a `$state` proxy. */
export function createSaveStatus() {
  let timer: ReturnType<typeof setTimeout> | null = null;
  const clear = (): void => {
    if (timer) { clearTimeout(timer); timer = null; }
  };

  const state = $state({
    status: 'idle' as SaveStatusValue,
    message: null as string | null,
    markSaving(): void {
      clear();
      state.status = 'saving';
      state.message = null;
    },
    markSaved(): void {
      clear();
      state.status = 'saved';
      state.message = null;
      timer = setTimeout(() => {
        timer = null;
        if (state.status === 'saved') state.status = 'idle';
      }, SAVED_LINGER_MS);
    },
    markError(msg: string): void {
      clear();
      state.status = 'error';
      state.message = msg;
    },
    reset(): void {
      clear();
      state.status = 'idle';
      state.message = null;
    },
  });

  return state;
}

/** The reactive object returned by `createSaveStatus()`. */
export type SaveStatus = ReturnType<typeof createSaveStatus>;
