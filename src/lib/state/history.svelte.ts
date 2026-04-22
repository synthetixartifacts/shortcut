/**
 * History State Management
 *
 * Manages dictation history with pagination support.
 * Uses Svelte 5 runes for reactive state management.
 */

import type { HistoryEntry } from '$lib/types';
import * as api from '$lib/api/tauri';
import { withAsyncState } from '$lib/utils/async-state';

/**
 * History state - reactive with $state rune
 */
export const historyState = $state({
  /** Current page entries */
  entries: [] as HistoryEntry[],
  /** Total number of entries */
  total: 0,
  /** Current page number (1-indexed) */
  currentPage: 1,
  /** Items per page */
  pageSize: 20,
  /** Total number of pages */
  totalPages: 0,
  /** Loading state */
  isLoading: false,
  /** Error message if any */
  error: null as string | null,
  /** Current search query */
  searchQuery: '',
});

/**
 * Load history entries for a specific page
 */
export async function loadHistory(page: number = 1): Promise<void> {
  await withAsyncState(historyState, async () => {
    const query = historyState.searchQuery || undefined;
    const result = await api.getHistory(page, historyState.pageSize, query);
    historyState.entries = result.entries;
    historyState.total = result.total;
    historyState.currentPage = result.page;
    historyState.totalPages = result.total_pages;
  });
}

/**
 * Search history by query string. Resets to page 1.
 */
export async function searchHistory(query: string): Promise<void> {
  historyState.searchQuery = query;
  await loadHistory(1);
}

/**
 * Delete a single history entry
 */
export async function deleteEntry(id: string): Promise<void> {
  await api.deleteHistoryEntry(id);
  await loadHistory(historyState.currentPage);
}

/**
 * Clear all history entries
 */
export async function clearAllHistory(): Promise<void> {
  await api.clearHistory();
  await loadHistory(1);
}

/**
 * Refresh history - reload page 1 to show newest entries
 * Called after adding new entries (e.g., from dictation)
 */
export async function refreshHistory(): Promise<void> {
  // Only refresh if we have loaded history at least once
  // (entries array exists means history page was visited)
  if (historyState.entries.length > 0 || historyState.total > 0) {
    await loadHistory(1);
  }
}
