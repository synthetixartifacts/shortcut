import { invokeWithErrorHandling } from './core';
import type { HistoryEntry, HistoryPage } from '$lib/types';

export async function getHistory(
  page: number = 1,
  pageSize: number = 20,
  query?: string
): Promise<HistoryPage> {
  return invokeWithErrorHandling<HistoryPage>('get_history', {
    page,
    pageSize,
    query: query || null,
  });
}

export async function addHistoryEntry(
  text: string,
  durationMs: number,
  language: string | null,
  engine?: string
): Promise<HistoryEntry> {
  return invokeWithErrorHandling<HistoryEntry>('add_history_entry', {
    text,
    durationMs,
    language,
    engine: engine || null,
  });
}

export async function deleteHistoryEntry(id: string): Promise<void> {
  await invokeWithErrorHandling<void>('delete_history_entry', { id });
}

export async function clearHistory(): Promise<void> {
  await invokeWithErrorHandling<void>('clear_history');
}
