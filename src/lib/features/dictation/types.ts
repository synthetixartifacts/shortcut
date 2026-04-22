/**
 * Dictation Customization Types
 *
 * Types for configuring speech recognition, context, and translation.
 */

/** Translation mode */
export type TranslationMode = 'off' | 'one_way';

/** Audio processing options */
export interface AudioSettings {
  noiseSuppression: boolean;
  echoCancellation: boolean;
  autoGainControl: boolean;
}

/** Translation term mapping */
export interface TranslationTerm {
  source: string;
  target: string;
}

/** Full dictation configuration */
export interface DictationConfig {
  // Audio
  selectedMicrophoneId: string | null;
  audioSettings: AudioSettings;

  // Context
  topic: string;
  names: string[];
  backgroundText: string;
  customTerms: string[];

  // Languages
  languageHints: string[];
  enableLanguageIdentification: boolean;

  // Translation
  translationMode: TranslationMode;
  translationTargetLanguage: string;
  translationLanguageA: string;
  translationLanguageB: string;
  translationTerms: TranslationTerm[];

  // Processing
  enableEndpointDetection: boolean;
  enableSpeakerDiarization: boolean;
}

/** Default configuration */
export const DEFAULT_DICTATION_CONFIG: DictationConfig = {
  selectedMicrophoneId: null,
  audioSettings: {
    noiseSuppression: true,
    echoCancellation: true,
    autoGainControl: true,
  },
  topic: '',
  names: [],
  backgroundText: '',
  customTerms: [],
  languageHints: [],
  enableLanguageIdentification: false,
  translationMode: 'off',
  translationTargetLanguage: 'en',
  translationLanguageA: 'en',
  translationLanguageB: 'es',
  translationTerms: [],
  enableEndpointDetection: true,
  enableSpeakerDiarization: false,
};
