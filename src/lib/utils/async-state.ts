/**
 * Async State Helper
 *
 * Wraps async state operations with loading/error tracking,
 * eliminating repetitive try/catch/finally boilerplate.
 */

import { extractErrorMessage } from './error';

/**
 * Wrap an async state operation with loading/error tracking.
 *
 * @param state - Object with `error` and a loading flag property
 * @param operation - The async work to perform
 * @param options - Optional config:
 *   - `loadingKey`: name of the boolean flag on `state` to toggle (default `'isLoading'`).
 *   - `rethrow`: re-throw the caught error after recording it (default `false`).
 *   - `errorFallback`: message used when the thrown error has no extractable text.
 *   - `onSaving`: called once `state[loadingKey]` is set to `true` and `state.error` is cleared.
 *   - `onSaved`: called only after the operation resolves successfully, before the return.
 *   - `onError`: called with the resolved error message after `state.error` is set.
 */
export async function withAsyncState<T>(
  state: { error: string | null } & Record<string, unknown>,
  operation: () => Promise<T>,
  options?: {
    loadingKey?: string;
    rethrow?: boolean;
    errorFallback?: string;
    onSaving?: () => void;
    onSaved?: () => void;
    onError?: (message: string) => void;
  }
): Promise<T | undefined> {
  const loadingKey = options?.loadingKey ?? 'isLoading';
  state[loadingKey] = true;
  state.error = null;
  options?.onSaving?.();
  try {
    const result = await operation();
    options?.onSaved?.();
    return result;
  } catch (e) {
    state.error = extractErrorMessage(e, options?.errorFallback);
    options?.onError?.(state.error);
    if (options?.rethrow) throw e;
    return undefined;
  } finally {
    state[loadingKey] = false;
  }
}
