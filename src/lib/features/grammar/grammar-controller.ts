/**
 * Grammar Controller
 */

import { createTextTransformHandler } from '$lib/features/text-transform/base-controller';
import { transformText } from '$lib/api/tauri';

export const handleGrammarFix = createTextTransformHandler({
  name: 'Grammar',
  successMessage: 'Grammar fixed!',
  transform: (text) => transformText('grammar', text),
  activityType: 'grammar',
});
