/**
 * Provider Readiness State
 *
 * Tracks which providers are configured (have API keys).
 * Does NOT make network calls — checks local config only.
 */

import { getProvidersConfig } from '$lib/api/tauri';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';

/** Per-provider readiness (exported type for type-safe consumers) */
export type ProviderReadiness = {
  /** At least one cloud LLM provider (OpenAI, Anthropic, Gemini, Grok) or Local is configured */
  any_llm_configured: boolean;
  /** Soniox key OR local engine is active */
  stt_configured: boolean;
  /** Initial check has completed */
  providers_checked: boolean;
  openai_ready: boolean;
  anthropic_ready: boolean;
  gemini_ready: boolean;
  grok_ready: boolean;
  /** Local LLM: ready when base URL is set (no key required) */
  local_ready: boolean;
  soniox_ready: boolean;
  /** Set externally by layout after reading engine state */
  local_engine_active: boolean;
  isChecking: boolean;
  error: string | null;
}

export const providerReadiness = $state<ProviderReadiness>({
  any_llm_configured: false,
  stt_configured: false,
  providers_checked: false,
  openai_ready: false,
  anthropic_ready: false,
  gemini_ready: false,
  grok_ready: false,
  local_ready: false,
  soniox_ready: false,
  local_engine_active: false,
  isChecking: true,
  error: null,
});

/**
 * Check which providers are configured (have API keys).
 * Reads local config only — no network calls.
 *
 * `providers_checked` is set in the outer `finally` (once, at the end)
 * regardless of whether the config read succeeded, so the layout never
 * remains stuck behind the startup loading screen.
 */
export async function checkProviderReadiness(): Promise<void> {
  try {
    await withAsyncState(providerReadiness, async () => {
      const config = await getProvidersConfig();
      const creds = config.credentials;

      providerReadiness.openai_ready = !!creds.openai_api_key;
      providerReadiness.anthropic_ready = !!creds.anthropic_api_key;
      providerReadiness.gemini_ready = !!creds.gemini_api_key;
      providerReadiness.grok_ready = !!creds.grok_api_key;
      // Local uses a URL, not a key — configured when URL is non-empty.
      // CANONICAL SOURCE OF TRUTH for "Local configured" (MASTER_PLAN D4). The
      // dropdown-configured check in `provider-catalog.ts::isProviderConfigured`
      // mirrors this exact rule so the Dashboard badge and the per-task
      // dropdowns never disagree. Discovery success (`models.local.length > 0`)
      // is informational only — never gate visibility on it.
      providerReadiness.local_ready = !!creds.local?.base_url;
      providerReadiness.soniox_ready = !!creds.soniox_api_key;

      providerReadiness.any_llm_configured =
        providerReadiness.openai_ready ||
        providerReadiness.anthropic_ready ||
        providerReadiness.gemini_ready ||
        providerReadiness.grok_ready ||
        providerReadiness.local_ready;

      // STT: Soniox cloud key OR local engine active (set externally by layout)
      providerReadiness.stt_configured =
        providerReadiness.soniox_ready || providerReadiness.local_engine_active;

      await log('[Providers] Readiness checked');
    }, { loadingKey: 'isChecking', errorFallback: 'Failed to check provider readiness' });
  } finally {
    providerReadiness.providers_checked = true;
  }
}
