/**
 * Improve Controller
 *
 * Uses the configured LLM provider to improve selected text.
 * The prompt template is managed in config.improve.prompt (user-editable).
 */

import { createTextTransformHandler } from '$lib/features/text-transform/base-controller';
import { transformText } from '$lib/api/tauri';

export const handleImproveText = createTextTransformHandler({
  name: 'Improve',
  successMessage: 'Text improved!',
  transform: (text) => transformText('improve', text),
  activityType: 'improve',
});
