/**
 * Engine State Management
 *
 * Manages transcription engine selection, capabilities, and model status.
 * Uses Svelte 5 runes for reactive state management.
 */

import type { EngineId, EngineInfo, EngineCapabilities, EngineStatus, ModelStatus } from '$lib/types';
import { getActiveEngine, setActiveEngine, getModelStatus, getConfig, getProvidersConfig, updateTranscriptionConfig } from '$lib/api/tauri';
import { log } from '$lib/utils/logger';
import { withAsyncState } from '$lib/utils/async-state';
import { appState } from '$lib/state/app.svelte';

/** Default capabilities (Soniox-like) used as fallback */
const DEFAULT_CAPABILITIES: EngineCapabilities = {
	supports_custom_terms: true,
	supports_background_text: true,
	supports_translation: false,
	requires_network: true,
	requires_token: true,
	requires_model_download: false,
	audio_leaves_device: true,
};

/** Platform detection */
function getCurrentPlatform(): string {
	const ua = navigator.userAgent.toLowerCase();
	if (ua.includes('win')) return 'windows';
	if (ua.includes('mac')) return 'macos';
	return 'linux';
}

/** Static engine definitions (capabilities are fixed at compile time) */
function getEngineDefinitions(): Omit<EngineInfo, 'status'>[] {
	return [
		{
			id: 'soniox',
			display_name: 'engine.soniox_name',
			description: 'engine.soniox_desc',
			privacy_summary: 'engine.soniox_privacy',
			platforms: ['windows', 'macos', 'linux'],
			capabilities: {
				supports_custom_terms: true,
				supports_background_text: true,
				supports_translation: false,
				requires_network: true,
				requires_token: true,
				requires_model_download: false,
				audio_leaves_device: true,
			},
		},
		{
			id: 'local-windows',
			display_name: 'engine.local_windows_name',
			description: 'engine.local_windows_desc',
			privacy_summary: 'engine.local_windows_privacy',
			platforms: ['windows'],
			capabilities: {
				supports_custom_terms: false,
				supports_background_text: false,
				supports_translation: false,
				requires_network: false,
				requires_token: false,
				requires_model_download: true,
				audio_leaves_device: false,
			},
			model_size_mb: 700,
		},
		{
			id: 'local-macos',
			display_name: 'engine.local_macos_name',
			description: 'engine.local_macos_desc',
			privacy_summary: 'engine.local_macos_privacy',
			platforms: ['macos'],
			capabilities: {
				supports_custom_terms: false,
				supports_background_text: false,
				supports_translation: false,
				requires_network: false,
				requires_token: false,
				requires_model_download: true,
				audio_leaves_device: false,
			},
			model_size_mb: 700,
		},
	];
}

/** Determine status for an engine based on context */
function resolveEngineStatus(
	def: Omit<EngineInfo, 'status'>,
	activeEngine: string,
	platform: string,
	credentialReady: boolean,
): EngineStatus {
	if (def.id === activeEngine) return 'active';
	if (!def.platforms.includes(platform)) {
		return def.id === 'local-macos' ? 'coming_soon' : 'not_available';
	}
	if (def.capabilities.requires_token && !credentialReady) return 'not_configured';
	return 'available';
}

/** Lookup whether an engine's required credential is configured */
function isEngineCredentialReady(
	engineId: string,
	credentials: { soniox_api_key: string },
): boolean {
	if (engineId === 'soniox') return !!credentials.soniox_api_key;
	return true;
}

/** Engine state */
export const engineState = $state<{
	activeEngine: string;
	pendingEngine: string | null;
	engines: EngineInfo[];
	modelStatus: ModelStatus | null;
	isLoading: boolean;
	isSwitching: boolean;
	error: string | null;
	platform: string;
	firstRunCompleted: boolean;
	showSlownessWarning: boolean;
	lastRtf: number;
	slownessDismissed: boolean;
}>({
	activeEngine: 'soniox',
	pendingEngine: null,
	engines: [],
	modelStatus: null,
	isLoading: false,
	isSwitching: false,
	error: null,
	platform: 'windows',
	firstRunCompleted: false,
	showSlownessWarning: false,
	lastRtf: 0,
	slownessDismissed: false,
});

/** Get capabilities of the active engine */
export function getActiveEngineCapabilities(): EngineCapabilities {
	const engine = engineState.engines.find((e) => e.id === engineState.activeEngine);
	return engine?.capabilities ?? DEFAULT_CAPABILITIES;
}

/** Load engine state from backend */
export async function loadEngineState(): Promise<void> {
	await withAsyncState(engineState, async () => {
		const platform = getCurrentPlatform();
		engineState.platform = platform;

		const [activeEngine, appConfig, providers] = await Promise.all([
			getActiveEngine(),
			getConfig(),
			getProvidersConfig(),
		]);
		engineState.activeEngine = activeEngine;
		engineState.firstRunCompleted = appConfig.transcription?.first_run_completed ?? false;
		engineState.slownessDismissed = appConfig.transcription?.slowness_dismissed ?? false;

		// Build engine list with status
		const definitions = getEngineDefinitions();
		const engines: EngineInfo[] = definitions.map((def) => ({
			...def,
			status: resolveEngineStatus(
				def,
				activeEngine,
				platform,
				isEngineCredentialReady(def.id, providers.credentials),
			),
		}));
		engineState.engines = engines;

		// Load model status (returns "unavailable" if local-stt feature is not enabled)
		try {
			const modelStatus = await getModelStatus();
			engineState.modelStatus = modelStatus;

			// Update local engine status based on model state
			if (modelStatus.state !== 'unavailable') {
				const localEngine = engines.find((e) => e.id === `local-${platform}`);
				if (localEngine && localEngine.status !== 'active' && modelStatus.state === 'not_downloaded') {
					localEngine.status = 'not_downloaded';
				}
			}
		} catch {
			engineState.modelStatus = null;
		}

		await log('[Engine] Loaded engine state');
	}, { errorFallback: 'Failed to load engine state' });
}

/** Switch to a different engine.
 *  Returns true if the switch completed, false if a model download is needed first.
 */
export async function switchEngine(engineId: string): Promise<boolean> {
	// Block engine switch during active recording
	if (appState.isRecording) {
		engineState.error = 'Cannot switch engines while recording. Finish your current dictation first.';
		return false;
	}

	// Block activation when the engine requires a credential that isn't set
	const def = engineState.engines.find((e) => e.id === engineId);
	if (def?.status === 'not_configured') {
		engineState.error = 'This engine requires an API key. Configure it in Settings → AI Providers.';
		return false;
	}

	// Check if the engine requires a model download first
	if (engineId.startsWith('local-')) {
		try {
			const status = await getModelStatus();
			if (status.state !== 'ready') {
				// Model not downloaded — caller should show download UI
				engineState.pendingEngine = engineId;
				return false;
			}
		} catch {
			// local-stt feature not available in this build
			engineState.error = 'Local transcription is not available in this build.';
			return false;
		}
	}

	await withAsyncState(engineState, async () => {
		await setActiveEngine(engineId);
		engineState.activeEngine = engineId;
		engineState.pendingEngine = null;
		await loadEngineState();
		await log(`[Engine] Switched to ${engineId}`);
	}, { loadingKey: 'isSwitching', errorFallback: 'Failed to switch engine' });
	return true;
}

/** Complete a pending engine switch after model download finishes */
export async function completePendingSwitch(): Promise<void> {
	const pending = engineState.pendingEngine;
	if (!pending) return;

	await withAsyncState(engineState, async () => {
		await setActiveEngine(pending);
		engineState.activeEngine = pending;
		engineState.pendingEngine = null;
		await loadEngineState();
		await log(`[Engine] Pending switch completed to ${pending}`);
	}, { loadingKey: 'isSwitching', errorFallback: 'Failed to switch engine' });
}

/** Show slowness warning (called from dictation controller) */
export function showSlownessWarning(rtf: number): void {
	engineState.showSlownessWarning = true;
	engineState.lastRtf = rtf;
}

/** Dismiss slowness warning */
export async function dismissSlownessWarning(dontAskAgain: boolean): Promise<void> {
	engineState.showSlownessWarning = false;
	if (dontAskAgain) {
		engineState.slownessDismissed = true;
		await updateTranscriptionConfig({
			active_engine: engineState.activeEngine,
			first_run_completed: engineState.firstRunCompleted,
			slowness_dismissed: true,
		});
	}
}
