/**
 * Text Transform History state.
 *
 * Mirrors `history.svelte.ts` but adds an explicit `loaded` flag (D7) and an
 * `actionFilter` field. The `loaded` flag flips to `true` after the first
 * `loadTextTransformHistory` resolution (success or error), so
 * `refreshTextTransformHistory` can correctly reload page 1 even when the
 * initial load returned zero entries — fixing the broken
 * `entries.length > 0 || total > 0` guard pattern in the dictation module.
 *
 * Uses Svelte 5 runes (`$state`) for reactive state management.
 */

import type { TextTransformHistoryEntry, TransformActionFilter } from '$lib/types';
import {
  getTextTransformHistory,
  deleteTextTransformHistoryEntry,
  clearTextTransformHistory,
} from '$lib/api/tauri';
import { withAsyncState } from '$lib/utils/async-state';

/**
 * Text transform history state - reactive with $state rune
 */
export const textTransformHistoryState = $state({
  /** Current page entries */
  entries: [] as TextTransformHistoryEntry[],
  /** Total number of entries (after filters applied) */
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
  /** Current search query (case-insensitive substring match on `result`) */
  searchQuery: '',
  /** Current action filter; `'all'` means unfiltered */
  actionFilter: 'all' as TransformActionFilter,
  /** D7: flips to `true` after the first load resolves (success or error) */
  loaded: false,
});

/**
 * Load text-transform history entries for a specific page.
 * Sets `loaded = true` after `withAsyncState` resolves regardless of outcome
 * so `refreshTextTransformHistory` can fire correctly even after an empty load.
 */
export async function loadTextTransformHistory(page: number = 1): Promise<void> {
  await withAsyncState(textTransformHistoryState, async () => {
    const query = textTransformHistoryState.searchQuery || undefined;
    const action =
      textTransformHistoryState.actionFilter === 'all'
        ? null
        : textTransformHistoryState.actionFilter;
    const result = await getTextTransformHistory(
      page,
      textTransformHistoryState.pageSize,
      query,
      action
    );
    textTransformHistoryState.entries = result.entries;
    textTransformHistoryState.total = result.total;
    textTransformHistoryState.currentPage = result.page;
    textTransformHistoryState.totalPages = result.total_pages;
  });
  textTransformHistoryState.loaded = true;
}

/**
 * Search text-transform history by query string. Resets to page 1.
 */
export async function searchTextTransformHistory(query: string): Promise<void> {
  textTransformHistoryState.searchQuery = query;
  await loadTextTransformHistory(1);
}

/**
 * Set the action filter (`'all' | 'grammar' | 'translate' | 'improve'`).
 * Resets to page 1.
 */
export async function setTextTransformActionFilter(
  filter: TransformActionFilter
): Promise<void> {
  textTransformHistoryState.actionFilter = filter;
  await loadTextTransformHistory(1);
}

/**
 * Delete a single text-transform history entry. Reloads the current page.
 */
export async function deleteTextTransformEntry(id: string): Promise<void> {
  await deleteTextTransformHistoryEntry(id);
  await loadTextTransformHistory(textTransformHistoryState.currentPage);
}

/**
 * Clear all text-transform history entries. Returns to page 1.
 */
export async function clearAllTextTransformHistory(): Promise<void> {
  await clearTextTransformHistory();
  await loadTextTransformHistory(1);
}

/**
 * Refresh text-transform history — reload page 1 to surface newest entries.
 * D7: gated on the `loaded` flag (NOT on `entries.length > 0 || total > 0`)
 * so refresh fires correctly even after an empty initial load.
 */
export async function refreshTextTransformHistory(): Promise<void> {
  if (textTransformHistoryState.loaded) {
    await loadTextTransformHistory(1);
  }
}
