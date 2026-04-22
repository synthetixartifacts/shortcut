/**
 * Text Transform Base Controller
 *
 * Factory for creating text transformation handlers.
 * Used by grammar, translation, and similar features.
 */

import { log, logError } from '$lib/utils/logger';
import { getSelectionWithFormat, pasteFormatted } from '$lib/api/tauri';
import { withIndicator } from '$lib/features/indicator';
import type { ActivityType } from '$lib/features/indicator';

interface TextTransformOptions {
  name: string;
  successMessage: string;
  transform: (text: string) => Promise<string>;
  /** Optional: show floating indicator during operation */
  activityType?: ActivityType;
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
