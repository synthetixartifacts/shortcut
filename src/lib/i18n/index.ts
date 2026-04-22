/**
 * Lightweight i18n system for ShortCut
 *
 * Usage:
 *   import { t } from '$lib/i18n';
 *   t('nav.dashboard')  // → "Dashboard"
 *
 * Translations are flat JSON objects with dot-notation keys.
 * Locale is read reactively from appSettingsState.language.
 */

import en from './locales/en.json';
import fr from './locales/fr.json';
import es from './locales/es.json';
import { appSettingsState } from '$lib/state/app-settings.svelte';

/** All loaded translation maps, keyed by locale code */
const translations: Record<string, Record<string, string>> = { en, fr, es };

/**
 * Translate a key using the current locale.
 * Falls back to English, then returns the key itself if not found.
 *
 * Supports simple interpolation: t('key', { count: 5 })
 * In the JSON: "key": "Found {count} items"
 */
export function t(key: string, params?: Record<string, string | number>): string {
  const locale = appSettingsState.language;
  let text = translations[locale]?.[key] ?? translations.en?.[key] ?? key;

  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(`{${k}}`, String(v));
    }
  }

  return text;
}
