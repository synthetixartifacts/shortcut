/**
 * Extract a human-readable message from an unknown error.
 */
export function extractErrorMessage(e: unknown, fallback = 'An unexpected error occurred'): string {
  if (e instanceof Error) return e.message;
  if (typeof e === 'string') return e;
  return fallback;
}

/**
 * Truncate a message to a maximum length, appending '...' if truncated.
 */
export function truncateMessage(message: string, maxLength: number): string {
  if (message.length <= maxLength) return message;
  return message.slice(0, maxLength - 3) + '...';
}

/**
 * Heuristic: does the given error look like a network/transport failure?
 *
 * Matches common wording from both the Tauri HTTP layer (`reqwest::Error`)
 * and browser fetch failures. Used by dictation to suggest a local fallback
 * when the cloud STT path fails for connectivity reasons.
 *
 * Extracted from `dictation-controller.ts` so other features (translation,
 * improve, screen-question) can share the same detection.
 */
export function isNetworkError(error: unknown): boolean {
  const msg = error instanceof Error ? error.message : String(error);
  const lower = msg.toLowerCase();
  return (
    lower.includes('network') ||
    lower.includes('timeout') ||
    lower.includes('failed to fetch') ||
    lower.includes('request failed') ||
    lower.includes('could not reach')
  );
}
