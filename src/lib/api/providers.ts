import { invokeWithErrorHandling } from './core';
import type {
  ProvidersConfig,
  ProviderModelInfo,
  ProviderStatusReport,
} from '$lib/types';

/**
 * Transform text using the configured LLM provider for the given task.
 * task: "grammar" | "translate" | "improve"
 */
export async function transformText(task: string, text: string): Promise<string> {
  return invokeWithErrorHandling<string>('transform_text', { task, text });
}

export async function getProviderStatus(): Promise<ProviderStatusReport> {
  return invokeWithErrorHandling<ProviderStatusReport>('get_provider_status');
}

export async function getProvidersConfig(): Promise<ProvidersConfig> {
  return invokeWithErrorHandling<ProvidersConfig>('get_providers_config');
}

export async function getProviderModels(providerId: string): Promise<ProviderModelInfo[]> {
  return invokeWithErrorHandling<ProviderModelInfo[]>('get_provider_models', { providerId });
}

export async function updateProvidersConfig(providers: ProvidersConfig): Promise<void> {
  await invokeWithErrorHandling<void>('update_providers_config', { providers });
}
