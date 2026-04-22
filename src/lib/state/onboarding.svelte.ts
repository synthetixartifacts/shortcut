/**
 * Onboarding state — form values + step orchestration for the first-run wizard.
 *
 * Extracted from `src/routes/onboarding/+page.svelte` in PHASE 3B so the route
 * becomes a thin composition shell and the three step components
 * (OnboardingStepLlm / StepStt / StepComplete) each get a focused surface.
 */

import {
  getProvidersConfig,
  getModelStatus,
  updateProvidersConfig,
  setActiveEngine,
  updateTranscriptionConfig,
} from '$lib/api/tauri';
import type { ProvidersConfig } from '$lib/types';
import { extractErrorMessage } from '$lib/utils/error';
import { DEFAULT_LOCAL_CHAT_URL, normalizeProvidersConfig } from '$lib/features/providers';
import { getCurrentPlatform, type Platform } from '$lib/features/shortcuts';
import { t } from '$lib/i18n';

export type OnboardingStep = 'llm' | 'stt' | 'complete';
export type SttEngineChoice = 'soniox' | 'local-windows';

/** All mutable state backing the three onboarding steps. */
export const onboardingState = $state({
  step: 'llm' as OnboardingStep,
  // LLM step credential form
  openaiKey: '',
  anthropicKey: '',
  geminiKey: '',
  grokKey: '',
  localUrl: DEFAULT_LOCAL_CHAT_URL,
  // STT step
  sonioxKey: '',
  selectedSttEngine: 'soniox' as SttEngineChoice,
  localModelReady: false,
  // Save/error flags per step
  isSaving: false,
  isSttSaving: false,
  saveError: null as string | null,
  sttError: null as string | null,
  // Resolved platform (async). Defaults to 'linux' until probe completes so
  // neither macOS-only nor Windows-only UI flashes for Linux users.
  platform: 'linux' as Platform,
});

export async function resolvePlatform(): Promise<void> {
  onboardingState.platform = await getCurrentPlatform();
  if (onboardingState.platform === 'windows') {
    await refreshLocalModelReady();
  }
}

export async function refreshLocalModelReady(): Promise<void> {
  try {
    const status = await getModelStatus();
    onboardingState.localModelReady = status.state === 'ready';
  } catch {
    onboardingState.localModelReady = false;
  }
}

/**
 * Save the filled-in LLM credentials (only fields the user touched).
 *
 * Reopening onboarding with pre-existing credentials and leaving fields blank
 * must NOT wipe the previously saved keys.
 */
export async function saveLlmAndContinue(): Promise<void> {
  onboardingState.isSaving = true;
  onboardingState.saveError = null;
  try {
    const config: ProvidersConfig = normalizeProvidersConfig(await getProvidersConfig());
    const openai = onboardingState.openaiKey.trim();
    if (openai) config.credentials.openai_api_key = openai;
    const anthropic = onboardingState.anthropicKey.trim();
    if (anthropic) config.credentials.anthropic_api_key = anthropic;
    const gemini = onboardingState.geminiKey.trim();
    if (gemini) config.credentials.gemini_api_key = gemini;
    const grok = onboardingState.grokKey.trim();
    if (grok) config.credentials.grok_api_key = grok;
    // Local: skip when the field equals the hardcoded default so we don't
    // overwrite a user-customized base URL with the default.
    const local = onboardingState.localUrl.trim();
    if (local && local !== DEFAULT_LOCAL_CHAT_URL) {
      config.credentials.local.base_url = local;
    }
    // D9: no probing on Onboarding. We persist whatever the user typed and
    // move on; discovery / error surfacing happens only on Settings → Providers.
    await updateProvidersConfig(normalizeProvidersConfig(config));
    onboardingState.step = 'stt';
  } catch (err) {
    onboardingState.saveError = extractErrorMessage(err);
  } finally {
    onboardingState.isSaving = false;
  }
}

export function selectSttEngine(engine: SttEngineChoice): void {
  onboardingState.selectedSttEngine = engine;
  onboardingState.sttError = null;
  if (engine === 'local-windows') {
    void refreshLocalModelReady();
  }
}

/**
 * Commit the user's STT choice, saving any Soniox key and setting the active
 * engine. Advances to the Complete step on success.
 */
export async function continueStt(): Promise<void> {
  onboardingState.isSttSaving = true;
  onboardingState.sttError = null;

  try {
    if (onboardingState.selectedSttEngine === 'soniox') {
      const trimmedKey = onboardingState.sonioxKey.trim();
      if (!trimmedKey) {
        onboardingState.sttError = t('onboarding.stt_key_required');
        return;
      }

      const config: ProvidersConfig = normalizeProvidersConfig(await getProvidersConfig());
      config.credentials.soniox_api_key = trimmedKey;
      await updateProvidersConfig(normalizeProvidersConfig(config));
      await setActiveEngine('soniox');
      await markComplete('soniox');
      onboardingState.step = 'complete';
      return;
    }

    const status = await getModelStatus();
    onboardingState.localModelReady = status.state === 'ready';
    if (!onboardingState.localModelReady) {
      onboardingState.sttError = t('onboarding.download_required');
      return;
    }

    await setActiveEngine('local-windows');
    await markComplete('local-windows');
    onboardingState.step = 'complete';
  } catch (err) {
    onboardingState.sttError = extractErrorMessage(err);
  } finally {
    onboardingState.isSttSaving = false;
  }
}

async function markComplete(engine: string): Promise<void> {
  await updateTranscriptionConfig({
    active_engine: engine,
    first_run_completed: true,
    slowness_dismissed: false,
  });
}

/** Skip all steps, using Soniox as the default engine (user can reconfigure). */
export async function skipAll(): Promise<void> {
  await updateTranscriptionConfig({
    active_engine: 'soniox',
    first_run_completed: true,
    slowness_dismissed: false,
  });
}

export function setStep(step: OnboardingStep): void {
  onboardingState.step = step;
}
