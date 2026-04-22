/**
 * Overlay Chat State
 *
 * Reactive state for the overlay chat conversation.
 * Manages messages array, streaming status, and error state.
 */

import type { ChatMessage } from '$lib/features/overlay-chat/types';

/** Overlay chat state */
export const overlayChatState = $state<{
	messages: ChatMessage[];
	isStreaming: boolean;
	error: string | null;
}>({
	messages: [],
	isStreaming: false,
	error: null
});

/** Add a message to the conversation */
export function addMessage(role: 'user' | 'assistant', content: string): void {
	overlayChatState.messages.push({
		id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
		role,
		content
	});
}

/** Start streaming: add an empty assistant message with isStreaming flag */
export function startStreaming(): void {
	overlayChatState.isStreaming = true;
	overlayChatState.error = null;
	overlayChatState.messages.push({
		id: `msg-${Date.now()}-stream`,
		role: 'assistant',
		content: '',
		isStreaming: true
	});
}

/** Append a chunk to the currently streaming message */
export function appendStreamChunk(content: string): void {
	const streamingMsg = overlayChatState.messages.find((m) => m.isStreaming);
	if (streamingMsg) {
		streamingMsg.content += content;
	}
}

/** Mark streaming as complete */
export function completeStreaming(): void {
	overlayChatState.isStreaming = false;
	const streamingMsg = overlayChatState.messages.find((m) => m.isStreaming);
	if (streamingMsg) {
		streamingMsg.isStreaming = false;
	}
}

/** Set an error message */
export function setError(error: string): void {
	overlayChatState.error = error;
}

/** Clear the entire conversation */
export function clearConversation(): void {
	overlayChatState.messages = [];
	overlayChatState.isStreaming = false;
	overlayChatState.error = null;
}
