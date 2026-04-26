/**
 * Text Transform Base Controller
 *
 * Factory for creating text transformation handlers.
 * Used by grammar, translation, and similar features.
 *
 * History integration (D6 — single hook):
 * After `pasteFormatted` succeeds, the result is recorded to text-transform
 * history via `addTextTransformHistoryEntry` and the page state is refreshed.
 * The save block is wrapped in its own try/catch so any failure is logged and
 * swallowed — D5 mandates that history persistence MUST NOT block paste.
 * Both direct shortcuts and the action wheel reach this factory through the
 * dispatcher, so a single hook covers all entry points (AC5).
 */

import { log, logError } from '$lib/utils/logger';
import {
  getSelectionWithFormat,
  pasteFormatted,
  addTextTransformHistoryEntry,
} from '$lib/api/tauri';
import { refreshTextTransformHistory } from '$lib/state/text-transform-history.svelte';
import { withIndicator } from '$lib/features/indicator';
import type { ActivityType } from '$lib/features/indicator';
import type { TransformAction } from '$lib/types';

interface TextTransformOptions {
  name: string;
  successMessage: string;
  transform: (text: string) => Promise<string>;
  /** Optional: show floating indicator during operation */
  activityType?: ActivityType;
}

/**
 * Map the controller's display `name` to the lowercase history action key.
 * Returns `null` for unknown names so non-transform contexts skip recording
 * silently — we don't want to pollute the history with non-action entries.
 */
function nameToAction(name: string): TransformAction | null {
  switch (name) {
    case 'Grammar':
      return 'grammar';
    case 'Translation':
      return 'translate';
    case 'Improve':
      return 'improve';
    default:
      return null;
  }
}

/**
 * Create a text transform handler
 */
export function createTextTransformHandler(options: TextTransformOptions) {
  const { name, successMessage, transform, activityType } = options;

  return async function handleTransform(): Promise<void> {
    try {
      await log(`[${name}] Starting...`);

      // Get selected text with format detection (HTML→Markdown if available)
      const selection = await getSelectionWithFormat();

      // Silently skip if no text selected
      if (!selection.text || selection.text.trim() === '') {
        await log(`[${name}] No text selected, skipping`);
        return;
      }

      // Core transformation logic
      async function doTransform(): Promise<void> {
        await log(`[${name}] Selected text (${selection.format}): "${selection.text.substring(0, 50)}..."`);

        const result = await transform(selection.text);
        await log(`[${name}] Result: "${result.substring(0, 50)}..."`);

        if (!result || result.trim() === '') {
          await log(`[${name}] Empty result from API`);
          throw new Error('Received empty response from provider');
        }

        await pasteFormatted(result, selection.format);

        // D6: single integration point for text-transform history. Runs AFTER
        // pasteFormatted so a paste failure short-circuits via the outer catch
        // and we never record an unpasted result. D5: history failure is fully
        // swallowed — the user's transform-and-paste workflow stays unaffected.
        const action = nameToAction(name);
        if (action && result.trim() !== '') {
          try {
            await addTextTransformHistoryEntry(action, result);
            // Best-effort UI refresh; D7's `loaded` flag gates the actual reload.
            void refreshTextTransformHistory();
          } catch (historyError) {
            await logError(
              `[${name}] Failed to record history (paste already done)`,
              historyError
            );
          }
        }

        await log(`[${name}] Completed`);
      }

      if (activityType) {
        // Wrap with indicator
        await withIndicator(activityType, doTransform, {
          successMessage: successMessage,
        });
      } else {
        // No indicator
        await doTransform();
      }
    } catch (e) {
      await logError(`[${name}] Failed`, e);
    }
  };
}
