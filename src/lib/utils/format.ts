/**
 * Format Utilities
 */

/**
 * Format action name for display
 * Example: 'grammar_fix' -> 'Grammar Fix'
 */
export function formatActionName(action: string): string {
  return action
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

/**
 * Format a Unix timestamp for display.
 * Shows time only for today, date + time otherwise.
 */
export function formatTimestamp(ts: number): string {
  const date = new Date(ts);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();

  if (isToday) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  return date.toLocaleDateString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

/**
 * Format a duration in milliseconds to a human-readable string.
 * Example: 5000 -> '5s', 65000 -> '1m 5s'
 */
export function formatDuration(ms: number): string {
  const seconds = Math.round(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}m ${secs}s`;
}
