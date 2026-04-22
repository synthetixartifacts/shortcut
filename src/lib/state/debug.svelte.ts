/**
 * Debug State Management
 *
 * In-memory log buffer for debugging issues in production.
 * Uses a non-reactive internal array to avoid render loops.
 * The Debug page explicitly refreshes to see new logs.
 */

export type LogLevel = 'info' | 'warn' | 'error';

export interface LogEntry {
  id: string;
  timestamp: Date;
  level: LogLevel;
  message: string;
}

/** Maximum number of log entries to keep in memory */
const MAX_LOGS = 500;

/** Counter for generating unique IDs */
let logIdCounter = 0;

/** Internal non-reactive log storage to avoid render loops */
const internalLogs: LogEntry[] = [];

/** Whether logging is enabled */
let loggingEnabled = true;

/**
 * Debug state - reactive snapshot for UI display
 * Only updates when refreshDebugLogs() is called
 */
export const debugState = $state<{
  logs: LogEntry[];
  isEnabled: boolean;
}>({
  logs: [],
  isEnabled: true,
});

/**
 * Add a log entry to the internal buffer (non-reactive)
 * Does NOT trigger re-renders
 */
export function addLogEntry(level: LogLevel, message: string): void {
  if (!loggingEnabled) return;

  const entry: LogEntry = {
    id: `log-${Date.now()}-${logIdCounter++}`,
    timestamp: new Date(),
    level,
    message,
  };

  // Add to end, remove from beginning if over limit (FIFO)
  internalLogs.push(entry);
  if (internalLogs.length > MAX_LOGS) {
    internalLogs.shift();
  }
}

/**
 * Refresh the reactive debug state from internal logs
 * Call this when opening the Debug page or to update the UI
 */
export function refreshDebugLogs(): void {
  debugState.logs = [...internalLogs];
}

/**
 * Clear all log entries
 */
export function clearLogs(): void {
  internalLogs.length = 0;
  debugState.logs = [];
}

/**
 * Get logs as a formatted string (for copying to clipboard)
 */
export function getLogsAsText(logs?: LogEntry[]): string {
  const entries = logs ?? internalLogs;
  return entries
    .map(log => {
      const time = log.timestamp.toISOString();
      const level = log.level.toUpperCase().padEnd(5);
      return `[${time}] ${level} ${log.message}`;
    })
    .join('\n');
}

