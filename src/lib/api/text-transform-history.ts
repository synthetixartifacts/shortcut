/**
 * Tauri wrapper for the text-transform history backend (Phase 1).
 * See `src-tauri/src/text_transform_history.rs`.
 *
 * Exposes the four backend commands:
 *   - get_text_transform_history     (paginated read with optional query/action filter)
 *   - add_text_transform_history_entry
 *   - delete_text_transform_history_entry
 *   - clear_text_transform_history
 *
 * Tauri's serde converts camelCase JS keys to snake_case Rust args (matches
 * the `history.ts` wrapper convention). `null` is forwarded for absent
 * `query`/`action` filters; the backend treats `None` and the literal `"all"`
 * action as no-op (Phase 1 handoff).
 */

import { invokeWithErrorHandling } from './core';
import type {
  TextTransformHistoryEntry,
  TextTransformHistoryPage,
  TransformAction,
} from '$lib/types';

export async function getTextTransformHistory(
  page: number = 1,
  pageSize: number = 20,
  query?: string,
  action?: TransformAction | null
): Promise<TextTransformHistoryPage> {
  return invokeWithErrorHandling<TextTransformHistoryPage>('get_text_transform_history', {
    page,
    pageSize,
    query: query || null,
    action: action || null,
  });
}

export async function addTextTransformHistoryEntry(
  action: TransformAction,
  result: string
): Promise<TextTransformHistoryEntry> {
  return invokeWithErrorHandling<TextTransformHistoryEntry>('add_text_transform_history_entry', {
    action,
    result,
  });
}

export async function deleteTextTransformHistoryEntry(id: string): Promise<void> {
  await invokeWithErrorHandling<void>('delete_text_transform_history_entry', { id });
}

export async function clearTextTransformHistory(): Promise<void> {
  await invokeWithErrorHandling<void>('clear_text_transform_history');
}
