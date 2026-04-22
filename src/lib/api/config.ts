import { invokeWithErrorHandling } from './core';
import type {
  AppConfig,
  DictationConfigBackend,
  HotkeyConfig,
  AppSettingsConfig,
  ImproveConfig,
  GrammarConfig,
  TranslateConfig,
  ScreenQuestionConfig,
} from '$lib/types';

export async function getConfig(): Promise<AppConfig> {
  return invokeWithErrorHandling<AppConfig>('get_config');
}

export async function updateDictationConfig(dictation: DictationConfigBackend): Promise<void> {
  await invokeWithErrorHandling<void>('update_dictation_config', { dictation });
}

export async function updateHotkeysConfig(hotkeys: HotkeyConfig): Promise<void> {
  await invokeWithErrorHandling<void>('update_hotkeys_config', { hotkeys });
}

export async function updateAppSettingsConfig(appSettings: AppSettingsConfig): Promise<void> {
  await invokeWithErrorHandling<void>('update_app_settings_config', { appSettings });
}

export async function updateImproveConfig(improve: ImproveConfig): Promise<void> {
  await invokeWithErrorHandling<void>('update_improve_config', { improve });
}

export async function getDefaultImproveConfig(): Promise<ImproveConfig> {
  return invokeWithErrorHandling<ImproveConfig>('get_default_improve_config');
}

export async function updateGrammarConfig(grammar: GrammarConfig): Promise<void> {
  await invokeWithErrorHandling<void>('update_grammar_config', { grammar });
}

export async function getDefaultGrammarConfig(): Promise<GrammarConfig> {
  return invokeWithErrorHandling<GrammarConfig>('get_default_grammar_config');
}

export async function updateTranslateConfig(translate: TranslateConfig): Promise<void> {
  await invokeWithErrorHandling<void>('update_translate_config', { translate });
}

export async function getDefaultTranslateConfig(): Promise<TranslateConfig> {
  return invokeWithErrorHandling<TranslateConfig>('get_default_translate_config');
}

export async function updateScreenQuestionConfig(
  screenQuestion: ScreenQuestionConfig,
): Promise<void> {
  await invokeWithErrorHandling<void>('update_screen_question_config', {
    screenQuestion,
  });
}

export async function getDefaultScreenQuestionConfig(): Promise<ScreenQuestionConfig> {
  return invokeWithErrorHandling<ScreenQuestionConfig>(
    'get_default_screen_question_config',
  );
}
