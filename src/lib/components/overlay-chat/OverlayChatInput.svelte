<script lang="ts">
	/**
	 * OverlayChatInput - Text input with send button
	 *
	 * Handles Enter to send, Shift+Enter for newlines.
	 * Shows loading state during streaming.
	 */

	import { t } from '$lib/i18n';
	import { Icon } from '$lib/components/ui/primitives';

	interface Props {
		placeholder?: string;
		disabled?: boolean;
		onSend: (message: string) => void;
	}

	let { placeholder = '', disabled = false, onSend }: Props = $props();

	let inputValue = $state('');
	let inputRef: HTMLTextAreaElement | undefined = $state(undefined);

	function handleKeyDown(e: KeyboardEvent): void {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	}

	function handleSend(): void {
		const trimmed = inputValue.trim();
		if (!trimmed || disabled) return;
		onSend(trimmed);
		inputValue = '';
		// Reset textarea height
		if (inputRef) inputRef.style.height = 'auto';
	}

	function handleInput(): void {
		// Auto-resize textarea
		if (inputRef) {
			inputRef.style.height = 'auto';
			inputRef.style.height = Math.min(inputRef.scrollHeight, 120) + 'px';
		}
	}

	// Auto-focus on mount
	$effect(() => {
		if (inputRef && !disabled) {
			inputRef.focus();
		}
	});
</script>

<div class="chat-input-container">
	<textarea
		bind:this={inputRef}
		bind:value={inputValue}
		onkeydown={handleKeyDown}
		oninput={handleInput}
		{placeholder}
		{disabled}
		rows="1"
		class="chat-input"
	></textarea>
	<button
		class="send-button"
		onclick={handleSend}
		disabled={disabled || !inputValue.trim()}
		title={t('common.send') || 'Send'}
	>
		<Icon name="send" size={16} />
	</button>
</div>

<style>
	.chat-input-container {
		display: flex;
		gap: 8px;
		padding: 8px 12px;
		background: var(--color-overlay-input-bg);
		border-radius: 8px;
		border: 1px solid var(--color-overlay-input-border);
	}

	.chat-input {
		flex: 1;
		background: transparent;
		border: none;
		color: var(--color-overlay-text);
		font-size: 0.9rem;
		font-family: inherit;
		resize: none;
		outline: none;
		min-height: 24px;
		max-height: 120px;
		line-height: 1.4;
	}

	.chat-input::placeholder {
		color: var(--color-overlay-text-muted);
	}

	.send-button {
		align-self: flex-end;
		padding: 6px;
		background: var(--color-overlay-button-bg);
		border: none;
		border-radius: 6px;
		color: var(--color-overlay-text);
		cursor: pointer;
		transition: background 0.15s ease;
		flex-shrink: 0;
	}

	.send-button:hover:not(:disabled) {
		background: var(--color-overlay-button-hover);
	}

	.send-button:disabled {
		opacity: 0.3;
		cursor: default;
	}
</style>
