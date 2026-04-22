/**
 * Overlay Chat Types
 *
 * Generic types for the reusable overlay chat system.
 * NOT tied to any specific feature (screen-question, quick-ask, etc.)
 */

/** A single message in the chat */
export interface ChatMessage {
	id: string;
	role: 'user' | 'assistant';
	content: string;
	isStreaming?: boolean;
}

/** Context provided to the chat (what the AI is analyzing) */
export interface ChatContext {
	type: 'screenshot' | 'text' | 'none';
	/** Base64 image data (for screenshot context) */
	imageBase64?: string;
	/** Image MIME type (e.g., "image/jpeg") */
	imageMimeType?: string;
	/** Selected text (for text context) */
	selectedText?: string;
}

/** Configuration for an overlay chat instance */
export interface OverlayChatConfig {
	/** Placeholder text for the input field */
	placeholder: string;
	/** Event name for streaming chunks from Rust */
	chunkEvent: string;
	/** Event name for stream completion from Rust */
	completeEvent: string;
	/** Event name for stream errors from Rust */
	errorEvent: string;
	/** Tauri command to invoke for sending messages */
	sendCommand: string;
}
