/**
 * Translation Controller
 */

import { createTextTransformHandler } from '$lib/features/text-transform/base-controller';
import { transformText } from '$lib/api/tauri';

export const handleTranslation = createTextTransformHandler({
  name: 'Translation',
  successMessage: 'Translated!',
  transform: (text) => transformText('translate', text),
  activityType: 'translate',
});
