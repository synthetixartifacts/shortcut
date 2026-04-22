/**
 * Dictation Configuration State
 *
 * Manages dictation settings with persistence via backend config.
 */

import type { DictationConfig, AudioSettings } from '$lib/features/dictation/types';
import { DEFAULT_DICTATION_CONFIG } from '$lib/features/dictation/types';
import type { AppConfig, DictationConfigBackend } from '$lib/types';
import { getConfig, updateDictationConfig } from '$lib/api/tauri';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';
import { createSaveStatus, type SaveStatus } from '$lib/utils/save-status.svelte';

/**
 * Per-field save-status keys exposed by `dictationConfigState.saveStatus`.
 *
 * Most entries map 1:1 to a single `DictationConfig` field. `audio_settings`
 * is per-section (noise/echo/gain toggled together); `custom_terms` is
 * per-section because `addCustomTerm` / `removeCustomTerm` mutate the list
 * in bulk.
 */
export type DictationFieldKey =
  | 'microphone'
  | 'audio_settings'
  | 'custom_terms'
  | 'topic'
  | 'names'
  | 'background_text'
  | 'language_hints'
  | 'language_identification'
  | 'translation_mode'
  | 'translation_target_language'
  | 'translation_language_a'
  | 'translation_language_b'
  | 'translation_terms'
  | 'enable_endpoint_detection'
  | 'enable_speaker_diarization';

const DICTATION_FIELD_KEYS: readonly DictationFieldKey[] = [
  'microphone',
  'audio_settings',
  'custom_terms',
  'topic',
  'names',
  'background_text',
  'language_hints',
  'language_identification',
  'translation_mode',
  'translation_target_language',
  'translation_language_a',
  'translation_language_b',
  'translation_terms',
  'enable_endpoint_detection',
  'enable_speaker_diarization',
];

function buildSaveStatusRecord(): Record<DictationFieldKey, SaveStatus> {
  const record = {} as Record<DictationFieldKey, SaveStatus>;
  for (const key of DICTATION_FIELD_KEYS) record[key] = createSaveStatus();
  return record;
}

/** State for dictation configuration */
export const dictationConfigState = $state<{
  config: DictationConfig;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  saveStatus: Record<DictationFieldKey, SaveStatus>;
}>({
  config: { ...DEFAULT_DICTATION_CONFIG },
  isLoading: false,
  isSaving: false,
  error: null,
  saveStatus: buildSaveStatusRecord(),
});

/**
 * Convert backend config (snake_case) to frontend config (camelCase)
 */
function convertFromBackend(backend: AppConfig['dictation']): Partial<DictationConfig> {
  if (!backend) return {};

  return {
    selectedMicrophoneId: backend.selected_microphone_id,
    audioSettings: backend.audio_settings ? {
      noiseSuppression: backend.audio_settings.noise_suppression,
      echoCancellation: backend.audio_settings.echo_cancellation,
      autoGainControl: backend.audio_settings.auto_gain_control,
    } : undefined,
    topic: backend.topic,
    names: backend.names,
    backgroundText: backend.background_text,
    customTerms: backend.custom_terms,
    languageHints: backend.language_hints,
    enableLanguageIdentification: backend.enable_language_identification,
    translationMode: backend.translation_mode as DictationConfig['translationMode'],
    translationTargetLanguage: backend.translation_target_language,
    translationLanguageA: backend.translation_language_a,
    translationLanguageB: backend.translation_language_b,
    translationTerms: backend.translation_terms,
    enableEndpointDetection: backend.enable_endpoint_detection,
    enableSpeakerDiarization: backend.enable_speaker_diarization,
  };
}

/**
 * Load dictation config from backend
 */
export async function loadDictationConfig(): Promise<void> {
  await withAsyncState(dictationConfigState, async () => {
    const appConfig = await getConfig();
    const backendConfig = convertFromBackend(appConfig.dictation);
    const merged = {
      ...DEFAULT_DICTATION_CONFIG,
      ...backendConfig,
      audioSettings: {
        ...DEFAULT_DICTATION_CONFIG.audioSettings,
        ...(backendConfig.audioSettings || {}),
      },
    };

    // Seed default terms if user has none saved
    if (!backendConfig.customTerms || backendConfig.customTerms.length === 0) {
      merged.customTerms = [...DEFAULT_DICTATION_CONFIG.customTerms];
    }

    dictationConfigState.config = merged;
    await log('[DictationConfig] Loaded');
  }, { errorFallback: 'Failed to load dictation config' });
}

/**
 * Convert frontend config (camelCase) to backend config (snake_case)
 */
function convertToBackend(config: DictationConfig): DictationConfigBackend {
  return {
    selected_microphone_id: config.selectedMicrophoneId,
    audio_settings: {
      noise_suppression: config.audioSettings.noiseSuppression,
      echo_cancellation: config.audioSettings.echoCancellation,
      auto_gain_control: config.audioSettings.autoGainControl,
    },
    topic: config.topic,
    names: config.names,
    background_text: config.backgroundText,
    custom_terms: config.customTerms,
    language_hints: config.languageHints,
    enable_language_identification: config.enableLanguageIdentification,
    translation_mode: config.translationMode,
    translation_target_language: config.translationTargetLanguage,
    translation_language_a: config.translationLanguageA,
    translation_language_b: config.translationLanguageB,
    translation_terms: config.translationTerms,
    enable_endpoint_detection: config.enableEndpointDetection,
    enable_speaker_diarization: config.enableSpeakerDiarization,
  };
}

/**
 * Save dictation config to backend.
 *
 * `fieldKey` is optional for backwards compatibility: when provided, the
 * corresponding `saveStatus[fieldKey]` entry flips through
 * `saving → saved` (or `error`); when omitted, no indicator flips. Phase 4
 * migrates page/component call sites to pass explicit field keys.
 */
export async function saveDictationConfig(
  updates: Partial<DictationConfig>,
  fieldKey?: DictationFieldKey,
): Promise<void> {
  await withAsyncState(dictationConfigState, async () => {
    const newDictationConfig: DictationConfig = {
      ...dictationConfigState.config,
      ...updates,
    };

    await updateDictationConfig(convertToBackend(newDictationConfig));

    dictationConfigState.config = newDictationConfig;
    await log('[DictationConfig] Saved');
  }, {
    loadingKey: 'isSaving',
    rethrow: true,
    errorFallback: 'Failed to save dictation config',
    onSaving: fieldKey ? () => dictationConfigState.saveStatus[fieldKey].markSaving() : undefined,
    onSaved: fieldKey ? () => dictationConfigState.saveStatus[fieldKey].markSaved() : undefined,
    onError: fieldKey ? (m) => dictationConfigState.saveStatus[fieldKey].markError(m) : undefined,
  });
}

/**
 * Add a custom term (per-section save — `custom_terms` indicator flips).
 */
export async function addCustomTerm(term: string): Promise<void> {
  const trimmed = term.trim();
  if (!trimmed || dictationConfigState.config.customTerms.includes(trimmed)) {
    return;
  }

  await saveDictationConfig(
    { customTerms: [...dictationConfigState.config.customTerms, trimmed] },
    'custom_terms',
  );
}

/**
 * Remove a custom term (per-section save — `custom_terms` indicator flips).
 */
export async function removeCustomTerm(term: string): Promise<void> {
  await saveDictationConfig(
    { customTerms: dictationConfigState.config.customTerms.filter((t) => t !== term) },
    'custom_terms',
  );
}


