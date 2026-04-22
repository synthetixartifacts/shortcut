/**
 * Overlay Chat Controller
 *
 * Manages conversation flow for the overlay chat.
 * Handles sending messages, listening for streaming responses,
 * and managing the conversation lifecycle.
 *
 * GENERIC: Does not know about screen-question or any specific feature.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invokeDynamic } from '$lib/api/tauri';
import { log, logError } from '$lib/utils/logger';
import {
	overlayChatState,
	addMessage,
	startStreaming,
	appendStreamChunk,
	completeStreaming,
	setError,
	clearConversation
} from '$lib/state/overlay-chat.svelte';
import type { ChatContext, OverlayChatConfig } from './types';

let chunkUnlisten: UnlistenFn | null = null;
let completeUnlisten: UnlistenFn | null = null;
let errorUnlisten: UnlistenFn | null = null;

/**
 * Initialize event listeners for streaming responses.
 */
export async function initListeners(config: OverlayChatConfig): Promise<void> {
	// Clean up any existing listeners
	cleanupListeners();

	chunkUnlisten = await listen<{ content: string }>(config.chunkEvent, (event) => {
		appendStreamChunk(event.payload.content);
	});

	completeUnlisten = await listen(config.completeEvent, () => {
		completeStreaming();
	});

	errorUnlisten = await listen<{ error: string }>(config.errorEvent, (event) => {
		setError(event.payload.error);
		completeStreaming();
	});

	await log('[OverlayChat] Event listeners initialized');
}

/**
 * Send a user message and trigger streaming response.
 */
export async function sendMessage(
	content: string,
	context: ChatContext,
	config: OverlayChatConfig
): Promise<void> {
	if (!content.trim() || overlayChatState.isStreaming) return;

	// Add user message to conversation
	addMessage('user', content);

	// Start streaming state (adds empty assistant message)
	startStreaming();

	try {
		// Build the messages array for the API
		const messages = overlayChatState.messages
			.filter((m) => !m.isStreaming)
			.map((m) => ({ role: m.role, content: m.content }));

		// Invoke the configured Tauri command
		// The command name and args depend on the feature using OverlayChat
		await invokeDynamic(config.sendCommand, {
			imageBase64: context.imageBase64 || '',
			imageMimeType: context.imageMimeType || '',
			messages
		});
	} catch (e) {
		logError('[OverlayChat] Send failed', e);
		setError(e instanceof Error ? e.message : String(e));
		completeStreaming();
	}
}

/**
 * Clean up event listeners.
 */
function cleanupListeners(): void {
	chunkUnlisten?.();
	chunkUnlisten = null;
	completeUnlisten?.();
	completeUnlisten = null;
	errorUnlisten?.();
	errorUnlisten = null;
}

/**
 * Reset the chat (clear conversation + clean up listeners).
 */
export function resetChat(): void {
	cleanupListeners();
	clearConversation();
}
