/**
 * TypeScript type definitions for ShortCut application
 */

/** Shortcut information from Rust backend */
export interface ShortcutInfo {
  action: string;
  shortcut: string;
  description: string;
}

/** Hotkey configuration */
export interface HotkeyConfig {
  dictation: string;
  grammar: string;
  translate: string;
  improve: string;
  open_menu: string;
  screen_question: string;
}

/** User profile configuration */
export interface UserConfig {
  name: string;
  preferred_language: string;
}

/** Dictation audio settings */
export interface DictationAudioSettings {
  noise_suppression: boolean;
  echo_cancellation: boolean;
  auto_gain_control: boolean;
}

/** Translation term mapping */
export interface DictationTranslationTerm {
  source: string;
  target: string;
}

/** Dictation configuration (from Rust - snake_case) */
export interface DictationConfigBackend {
  selected_microphone_id: string | null;
  audio_settings: DictationAudioSettings;
  topic: string;
  names: string[];
  background_text: string;
  custom_terms: string[];
  language_hints: string[];
  enable_language_identification: boolean;
  translation_mode: string;
  translation_target_language: string;
  translation_language_a: string;
  translation_language_b: string;
  translation_terms: DictationTranslationTerm[];
  enable_endpoint_detection: boolean;
  enable_speaker_diarization: boolean;
}

/** Improve (MAI) configuration */
export interface ImproveConfig {
  prompt: string;
  system_prompt: string;
}

/** App-level settings (theme, language, debug visibility) */
export interface AppSettingsConfig {
  theme: string;
  language: string;
  debug_enabled: boolean;
}

/** Transcription engine configuration (from Rust - snake_case) */
export interface TranscriptionConfig {
  active_engine: string;
  first_run_completed: boolean;
  slowness_dismissed: boolean;
}

/** Application configuration from Rust backend */
export interface AppConfig {
  hotkeys: HotkeyConfig;
  user: UserConfig;
  dictation?: DictationConfigBackend;
  app_settings?: AppSettingsConfig;
  improve?: ImproveConfig;
  transcription?: TranscriptionConfig;
  /** Provider credentials and task assignments (Phase 2+) */
  providers?: ProvidersConfig;
  /** Grammar prompt configuration (Phase 2+) */
  grammar?: GrammarConfig;
  /** Translation prompt configuration (Phase 2+) */
  translate?: TranslateConfig;
  /** Screen-question system prompt configuration */
  screen_question?: ScreenQuestionConfig;
}

/** Shortcut action types */
export type ShortcutAction =
  | 'dictation_start'
  | 'dictation_stop'
  | 'dictation'
  | 'grammar'
  | 'translate'
  | 'improve'
  | 'open_menu'
  | 'screen_question';

/** Transcription result from backend (engine-agnostic) */
export interface TranscriptionResult {
  text: string;
  duration_ms: number;
  language?: string;
  engine?: string;
}

// =============================================================================
// Provider abstraction types
// =============================================================================

/** Provider credentials (new config shape) */
export interface ProviderCredentials {
  openai_api_key: string;
  anthropic_api_key: string;
  gemini_api_key: string;
  grok_api_key: string;
  soniox_api_key: string;
  /** Local chat completion URL (default: http://localhost:11434/api/chat) */
  ollama_base_url: string;
  /** Legacy hidden field kept for config compatibility; ignored by routing */
  openai_base_url: string;
  /** Legacy hidden field kept for config compatibility; ignored by routing */
  soniox_base_url: string;
}

export interface TaskAssignment {
  provider_id: string;
  model: string;
  /**
   * Per-model vision capability as reported by provider discovery.
   * `undefined`/null means "unknown" — backend falls back to provider-level
   * capability. `true`/`false` reflects the discovered per-model flag.
   */
  supports_vision?: boolean | null;
}

export interface TaskAssignments {
  grammar: TaskAssignment;
  translate: TaskAssignment;
  improve: TaskAssignment;
  screen_question: TaskAssignment;
}

export interface ProvidersConfig {
  credentials: ProviderCredentials;
  task_assignments: TaskAssignments;
}

export interface GrammarConfig {
  prompt: string;
  system_prompt: string;
}

export interface TranslateConfig {
  prompt: string;
  system_prompt: string;
}

export interface ScreenQuestionConfig {
  system_prompt: string;
}

export interface ProviderModelInfo {
  id: string;
  label: string;
  supports_vision: boolean;
}

/**
 * Per-provider configuration status report (from Rust: get_provider_status)
 */
export interface ProviderStatusReport {
  openai_configured: boolean;
  anthropic_configured: boolean;
  gemini_configured: boolean;
  grok_configured: boolean;
  soniox_configured: boolean;
  ollama_url: string;
  active_engine: string;
  grammar_provider: string;
  grammar_model: string;
  translate_provider: string;
  translate_model: string;
  improve_provider: string;
  improve_model: string;
  screen_question_provider: string;
  screen_question_model: string;
}

// =============================================================================

/** A single history entry representing one dictation transcription */
export interface HistoryEntry {
  id: string;
  text: string;
  timestamp: number;
  duration_ms: number;
  language: string | null;
  engine?: string;
}

/** Paginated history response */
export interface HistoryPage {
  entries: HistoryEntry[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

// ============================================
// Transcription Engine Types
// ============================================

/** Transcription engine identifier */
export type EngineId = 'soniox' | 'local-windows' | 'local-macos';

/** Engine availability status */
export type EngineStatus =
  | 'active'
  | 'available'
  | 'not_downloaded'
  | 'downloading'
  | 'not_available'
  | 'not_configured'
  | 'coming_soon';

/** Engine capability flags */
export interface EngineCapabilities {
  supports_custom_terms: boolean;
  supports_background_text: boolean;
  supports_translation: boolean;
  requires_network: boolean;
  requires_token: boolean;
  requires_model_download: boolean;
  audio_leaves_device: boolean;
}

/** Engine descriptor (static info + runtime status) */
export interface EngineInfo {
  id: EngineId;
  display_name: string;
  description: string;
  privacy_summary: string;
  platforms: string[];
  capabilities: EngineCapabilities;
  status: EngineStatus;
  model_size_mb?: number;
}

/** Model download/lifecycle status from Rust */
export interface ModelStatus {
  state: 'not_downloaded' | 'downloading' | 'ready' | 'corrupt' | 'unavailable';
  progress?: number;
  size_bytes?: number;
  path?: string;
  error?: string;
}
