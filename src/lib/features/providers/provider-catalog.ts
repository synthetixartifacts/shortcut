import { t } from '$lib/i18n';
import type { LocalCredentials, ProviderCredentials, ProviderModelInfo, ProvidersConfig, TaskAssignment, TaskAssignments } from '$lib/types';

export type TaskKey = keyof TaskAssignments;
export type ProviderId = 'openai' | 'anthropic' | 'gemini' | 'grok' | 'local';

export interface ProviderOption {
	value: ProviderId;
	label: string;
}

export interface ModelOption {
	value: string;
	label: string;
}

/**
 * Default base URL for the Local LLM server. Bare origin — the Rust
 * normalizer (`normalize_local_base_url`) and the provider adapter append
 * the right path for whichever protocol resolves (Ollama `/api/chat`,
 * OpenAI-compat `/v1/chat/completions`). Keeping this suffix-free avoids
 * confusing auto-detection, which probes both `/api/tags` and `/v1/models`
 * off the origin.
 */
export const DEFAULT_LOCAL_CHAT_URL = 'http://localhost:11434';
export const DEFAULT_SONIOX_URL = 'https://api.soniox.com';

/**
 * Sentinel value used in the model <Select> for Local providers to reveal a
 * free-text input. Chosen to be unambiguous (no real model id starts with
 * double underscore) and scoped to Local only — cloud provider dropdowns never
 * surface this option (Phase 4 / D5).
 */
export const CUSTOM_MODEL_SENTINEL = '__custom__';

const PROVIDER_ORDER: ProviderId[] = ['openai', 'anthropic', 'gemini', 'grok', 'local'];
const VISION_PROVIDER_ORDER: ProviderId[] = ['openai', 'anthropic', 'gemini', 'grok', 'local'];

const DEFAULT_TASK_MODELS: Record<ProviderId, Record<TaskKey, string>> = {
	openai: {
		grammar: 'gpt-5.4-mini',
		translate: 'gpt-5.4-mini',
		improve: 'gpt-5.4',
		screen_question: 'gpt-5.4',
	},
	anthropic: {
		grammar: 'claude-3-5-haiku-20241022',
		translate: 'claude-3-5-haiku-20241022',
		improve: 'claude-sonnet-4-20250514',
		screen_question: 'claude-sonnet-4-20250514',
	},
	gemini: {
		grammar: 'gemini-2.5-flash',
		translate: 'gemini-2.5-flash',
		improve: 'gemini-2.5-pro',
		screen_question: 'gemini-2.5-flash',
	},
	grok: {
		grammar: 'grok-4',
		translate: 'grok-4',
		improve: 'grok-4',
		screen_question: 'grok-4',
	},
	// Local has no hardcoded default model — the user's server (Ollama, LM Studio,
	// etc.) may run any model id. Empty string triggers the "Custom" flow in the
	// settings UI, where the user either picks from the discovered list or types
	// a custom id. Keys are preserved so callers that index by TaskKey don't
	// break. See `normalizeTaskAssignment` which preserves empty strings for Local.
	local: {
		grammar: '',
		translate: '',
		improve: '',
		screen_question: '',
	},
};

function defaultTaskAssignments(): TaskAssignments {
	return {
		grammar: { provider_id: 'openai', model: DEFAULT_TASK_MODELS.openai.grammar, supports_vision: null },
		translate: { provider_id: 'openai', model: DEFAULT_TASK_MODELS.openai.translate, supports_vision: null },
		improve: { provider_id: 'openai', model: DEFAULT_TASK_MODELS.openai.improve, supports_vision: null },
		screen_question: { provider_id: 'openai', model: DEFAULT_TASK_MODELS.openai.screen_question, supports_vision: true },
	};
}

function defaultLocalCredentials(): LocalCredentials {
	return {
		base_url: DEFAULT_LOCAL_CHAT_URL,
		protocol: 'auto',
		detected_protocol: null,
		api_key: null,
	};
}

function defaultCredentials(): ProviderCredentials {
	return {
		openai_api_key: '',
		anthropic_api_key: '',
		gemini_api_key: '',
		grok_api_key: '',
		soniox_api_key: '',
		local: defaultLocalCredentials(),
		ollama_base_url: '',
		openai_base_url: '',
		soniox_base_url: DEFAULT_SONIOX_URL,
	};
}

function getProviderLabel(providerId: ProviderId): string {
	switch (providerId) {
		case 'openai':
			return 'OpenAI';
		case 'anthropic':
			return 'Anthropic';
		case 'gemini':
			return 'Gemini';
		case 'grok':
			return 'Grok';
		case 'local':
			return t('settings.provider_local');
	}
}

export function createDefaultProvidersConfig(): ProvidersConfig {
	return {
		credentials: defaultCredentials(),
		task_assignments: defaultTaskAssignments(),
	};
}

export function getAllProviderIds(): ProviderId[] {
	return [...PROVIDER_ORDER];
}

/**
 * Light normalization for the Local base URL as stored in config: trim
 * whitespace + trailing slashes. We deliberately do **not** append API paths
 * (`/api/chat`, `/v1/…`) here — the Rust side (`normalize_local_base_url`)
 * strips suffixes before every request so the user's pasted value is
 * preserved as-is and auto-detection can probe the origin cleanly.
 */
export function normalizeLocalChatUrl(url: string): string {
	const trimmed = url.trim().replace(/\/+$/, '');
	if (!trimmed) return DEFAULT_LOCAL_CHAT_URL;
	return trimmed;
}

export function getProviderOptions(taskKey: TaskKey): ProviderOption[] {
	return getProviderIds(taskKey).map((providerId) => ({
		value: providerId,
		label: getProviderLabel(providerId),
	}));
}

export function isProviderConfigured(
	providerId: ProviderId,
	credentials: ProviderCredentials,
	models: Partial<Record<ProviderId, ProviderModelInfo[]>>,
): boolean {
	switch (providerId) {
		case 'openai':
			return credentials.openai_api_key.trim().length > 0;
		case 'anthropic':
			return credentials.anthropic_api_key.trim().length > 0;
		case 'gemini':
			return credentials.gemini_api_key.trim().length > 0;
		case 'grok':
			return credentials.grok_api_key.trim().length > 0;
		case 'local':
			// Local: URL alone = configured (MASTER_PLAN D4). Discovery success is
			// informational only — keying this off `models.local.length` hid Local
			// from every dropdown when the local daemon was down. The readiness
			// store (`providers.svelte.ts::local_ready`) is the canonical source of
			// truth; this arm mirrors that rule so the dropdown-configured check and
			// the dashboard readiness badge never disagree.
			return credentials.local.base_url.trim().length > 0;
	}
}

export function getConfiguredProviderOptions(
	taskKey: TaskKey,
	credentials: ProviderCredentials,
	models: Partial<Record<ProviderId, ProviderModelInfo[]>>,
): ProviderOption[] {
	return getProviderIds(taskKey)
		.filter((providerId) => isProviderConfigured(providerId, credentials, models))
		.map((providerId) => ({
			value: providerId,
			label: getProviderLabel(providerId),
		}));
}

export function hasAnyConfiguredProvider(
	credentials: ProviderCredentials,
	models: Partial<Record<ProviderId, ProviderModelInfo[]>>,
): boolean {
	return PROVIDER_ORDER.some((providerId) => isProviderConfigured(providerId, credentials, models));
}

export function getDefaultModel(taskKey: TaskKey, providerId: string): string {
	const normalizedProviderId = normalizeProviderId(taskKey, providerId);
	return DEFAULT_TASK_MODELS[normalizedProviderId][taskKey];
}

export function normalizeProvidersConfig(config: ProvidersConfig): ProvidersConfig {
	const base = createDefaultProvidersConfig();
	const existingLocal = config.credentials.local ?? base.credentials.local;
	const credentials: ProviderCredentials = {
		...base.credentials,
		...config.credentials,
		local: {
			...base.credentials.local,
			...existingLocal,
			base_url: normalizeLocalChatUrl(existingLocal.base_url ?? base.credentials.local.base_url),
		},
		ollama_base_url: '',
		openai_base_url: '',
		soniox_base_url: DEFAULT_SONIOX_URL,
	};

	return {
		credentials,
		task_assignments: {
			grammar: normalizeTaskAssignment('grammar', config.task_assignments.grammar),
			translate: normalizeTaskAssignment('translate', config.task_assignments.translate),
			improve: normalizeTaskAssignment('improve', config.task_assignments.improve),
			screen_question: normalizeTaskAssignment('screen_question', config.task_assignments.screen_question),
		},
	};
}

function normalizeTaskAssignment(taskKey: TaskKey, assignment: TaskAssignment): TaskAssignment {
	const providerId = normalizeProviderId(taskKey, assignment.provider_id);
	// For Local, preserve empty-string model so the "Custom…" sentinel flow can
	// round-trip — a fresh Local assignment that the user picked "Custom" on
	// must not be silently replaced by a cloud default ("gemma3"). Empty-string
	// semantics: user picked Custom but hasn't typed a model id yet. For all
	// other providers, fall back to the documented default.
	const trimmed = assignment.model.trim();
	const model = providerId === 'local'
		? trimmed
		: (trimmed || getDefaultModel(taskKey, providerId));
	return {
		provider_id: providerId,
		model,
		supports_vision: assignment.supports_vision ?? null,
	};
}

function normalizeProviderId(taskKey: TaskKey, providerId: string): ProviderId {
	const validProviderIds = getProviderIds(taskKey);
	if (validProviderIds.includes(providerId as ProviderId)) {
		return providerId as ProviderId;
	}
	return defaultTaskAssignments()[taskKey].provider_id as ProviderId;
}

function getProviderIds(taskKey: TaskKey): ProviderId[] {
	return taskKey === 'screen_question' ? VISION_PROVIDER_ORDER : PROVIDER_ORDER;
}
