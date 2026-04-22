import { t } from '$lib/i18n';
import type { ProviderCredentials, ProviderModelInfo, ProvidersConfig, TaskAssignment, TaskAssignments } from '$lib/types';

export type TaskKey = keyof TaskAssignments;
export type ProviderId = 'openai' | 'anthropic' | 'gemini' | 'grok' | 'ollama';

export interface ProviderOption {
	value: ProviderId;
	label: string;
}

export interface ModelOption {
	value: string;
	label: string;
}

export const DEFAULT_LOCAL_CHAT_URL = 'http://localhost:11434/api/chat';
export const DEFAULT_SONIOX_URL = 'https://api.soniox.com';

const PROVIDER_ORDER: ProviderId[] = ['openai', 'anthropic', 'gemini', 'grok', 'ollama'];
const VISION_PROVIDER_ORDER: ProviderId[] = ['openai', 'anthropic', 'gemini', 'grok', 'ollama'];

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
	ollama: {
		grammar: 'gemma3',
		translate: 'gemma3',
		improve: 'gpt-oss:20b',
		screen_question: 'gemma3',
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

function defaultCredentials(): ProviderCredentials {
	return {
		openai_api_key: '',
		anthropic_api_key: '',
		gemini_api_key: '',
		grok_api_key: '',
		soniox_api_key: '',
		ollama_base_url: DEFAULT_LOCAL_CHAT_URL,
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
		case 'ollama':
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

export function normalizeLocalChatUrl(url: string): string {
	const trimmed = url.trim();
	if (!trimmed) return DEFAULT_LOCAL_CHAT_URL;

	try {
		const parsed = new URL(trimmed);
		const pathname = parsed.pathname.replace(/\/+$/, '');
		if (!pathname || pathname === '') {
			parsed.pathname = '/api/chat';
			parsed.search = '';
			parsed.hash = '';
			return parsed.toString();
		}
		if (pathname === '/api') {
			parsed.pathname = '/api/chat';
			return parsed.toString();
		}
		parsed.pathname = pathname;
		return parsed.toString();
	} catch {
		return trimmed.replace(/\/+$/, '') || DEFAULT_LOCAL_CHAT_URL;
	}
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
		case 'ollama':
			return (models.ollama ?? []).length > 0;
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
	const credentials: ProviderCredentials = {
		...base.credentials,
		...config.credentials,
		ollama_base_url: normalizeLocalChatUrl(config.credentials.ollama_base_url),
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
	const model = assignment.model.trim() || getDefaultModel(taskKey, providerId);
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
