/**
 * Tauri API barrel — re-exports all backend command wrappers grouped by domain.
 *
 * Split across:
 *   core.ts       — invokeWithErrorHandling helper + frontendLog
 *   clipboard.ts  — paste + selection
 *   shortcuts.ts  — global hotkey registration
 *   config.ts     — app config read/write
 *   providers.ts  — transform_text + providers config + status
 *   windows.ts    — indicator / action menu / screen question windows
 *   dictation.ts  — transcribe + engine + model management
 *   history.ts    — history CRUD
 */
export { frontendLog, invokeDynamic } from './core';
export { pasteText, getSelectionWithFormat, pasteFormatted } from './clipboard';
export { getRegisteredShortcuts, updateShortcuts, getDefaultShortcuts } from './shortcuts';
export {
  getConfig,
  updateDictationConfig,
  updateHotkeysConfig,
  updateAppSettingsConfig,
  updateImproveConfig,
  getDefaultImproveConfig,
  updateGrammarConfig,
  getDefaultGrammarConfig,
  updateTranslateConfig,
  getDefaultTranslateConfig,
  updateScreenQuestionConfig,
  getDefaultScreenQuestionConfig,
} from './config';
export {
  transformText,
  getProviderModels,
  getProviderStatus,
  getProvidersConfig,
  updateProvidersConfig,
} from './providers';
export {
  showIndicator,
  hideIndicator,
  resetIndicator,
  toggleActionMenu,
  hideActionMenu,
  screenQuestion,
  hideScreenQuestion,
} from './windows';
export {
  transcribeAudio,
  getActiveEngine,
  setActiveEngine,
  updateTranscriptionConfig,
  getModelStatus,
  downloadModel,
  deleteModel,
  cancelModelDownload,
} from './dictation';
export {
  getHistory,
  addHistoryEntry,
  deleteHistoryEntry,
  clearHistory,
} from './history';
