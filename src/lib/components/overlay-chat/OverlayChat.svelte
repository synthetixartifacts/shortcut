<script lang="ts">
	/**
	 * OverlayChat - Reusable overlay chat container
	 *
	 * Renders messages, input, and optional context thumbnail.
	 * GENERIC: works with any context type and agent.
	 */

	import { onMount, onDestroy } from 'svelte';
	import { overlayChatState } from '$lib/state/overlay-chat.svelte';
	import { initListeners, resetChat, sendMessage } from '$lib/features/overlay-chat';
	import type { ChatContext, OverlayChatConfig } from '$lib/features/overlay-chat/types';
	import { AUTO_SCROLL_THRESHOLD } from '$lib/features/overlay-chat/constants';
	import { t } from '$lib/i18n';
	import { Icon } from '$lib/components/ui/primitives';
	import OverlayChatMessage from './OverlayChatMessage.svelte';
	import OverlayChatInput from './OverlayChatInput.svelte';

	interface Props {
		context: ChatContext;
		config: OverlayChatConfig;
		onClose: () => void;
	}

	let { context, config, onClose }: Props = $props();

	let messagesContainer: HTMLDivElement | undefined = $state(undefined);

	// Auto-scroll to bottom when new messages arrive
	$effect(() => {
		// Track messages length to trigger scroll
		const _len = overlayChatState.messages.length;
		const lastMsg = overlayChatState.messages.at(-1);
		const _content = lastMsg?.content;

		if (messagesContainer) {
			const { scrollTop, scrollHeight, clientHeight } = messagesContainer;
			const isNearBottom = scrollHeight - scrollTop - clientHeight < AUTO_SCROLL_THRESHOLD;
			if (isNearBottom) {
				requestAnimationFrame(() => {
					messagesContainer?.scrollTo({
						top: messagesContainer.scrollHeight,
						behavior: 'smooth'
					});
				});
			}
		}
	});

	function handleSend(content: string): void {
		sendMessage(content, context, config);
	}

	function handleKeyDown(e: KeyboardEvent): void {
		if (e.key === 'Escape') {
			onClose();
		}
	}

	onMount(async () => {
		await initListeners(config);
	});

	onDestroy(() => {
		resetChat();
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay-chat" onkeydown={handleKeyDown}>
	<!-- Header with close button -->
	<div class="chat-header">
		{#if context.type === 'screenshot'}
			<span class="context-badge">{t('overlay_chat.context_screenshot')}</span>
		{:else if context.type === 'text'}
			<span class="context-badge">{t('overlay_chat.context_text')}</span>
		{:else}
			<span></span>
		{/if}
		<button class="close-button" onclick={onClose} title="{t('common.close')} (Escape)">
			<Icon name="close" size={14} />
		</button>
	</div>

	<!-- Image thumbnail (if screenshot context) -->
	{#if context.type === 'screenshot' && context.imageBase64}
		<div class="image-preview">
			<img
				src="data:{context.imageMimeType || 'image/jpeg'};base64,{context.imageBase64}"
				alt=""
				class="thumbnail"
			/>
		</div>
	{/if}

	<!-- Messages area -->
	<div class="messages-area" bind:this={messagesContainer}>
		{#each overlayChatState.messages as message (message.id)}
			<OverlayChatMessage {message} />
		{/each}

		{#if overlayChatState.error}
			<div class="error-message">{overlayChatState.error}</div>
		{/if}
	</div>

	<!-- Input -->
	<OverlayChatInput
		placeholder={config.placeholder}
		disabled={overlayChatState.isStreaming}
		onSend={handleSend}
	/>
</div>

<style>
	.overlay-chat {
		display: flex;
		flex-direction: column;
		width: 100%;
		height: 100%;
		max-width: 500px;
		max-height: 460px;
		background: var(--color-overlay-bg);
		border-radius: 12px;
		border: 1px solid var(--color-overlay-border);
		overflow: hidden;
		box-shadow: var(--shadow-modal);
		color: var(--color-overlay-text);
	}

	.chat-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-overlay-border);
	}

	.context-badge {
		font-size: 0.7rem;
		padding: 2px 8px;
		background: var(--color-overlay-badge-bg);
		border-radius: 4px;
		color: var(--color-overlay-badge-text);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.close-button {
		padding: 4px;
		background: transparent;
		border: none;
		color: var(--color-overlay-close-color);
		cursor: pointer;
		border-radius: 4px;
		transition: all 0.15s ease;
	}

	.close-button:hover {
		background: var(--color-overlay-surface);
		color: var(--color-overlay-text);
	}

	.image-preview {
		padding: 8px 12px 4px;
		border-bottom: 1px solid var(--color-overlay-border);
	}

	.thumbnail {
		width: 100%;
		max-height: 120px;
		object-fit: contain;
		border-radius: 6px;
		opacity: 0.85;
	}

	.messages-area {
		flex: 1;
		overflow-y: auto;
		padding: 8px 12px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-height: 100px;
	}

	.error-message {
		padding: 8px 12px;
		background: var(--color-danger-light);
		border-radius: 8px;
		color: var(--color-danger);
		font-size: 0.8rem;
	}

	/* Scrollbar styling */
	.messages-area::-webkit-scrollbar {
		width: 4px;
	}

	.messages-area::-webkit-scrollbar-track {
		background: transparent;
	}

	.messages-area::-webkit-scrollbar-thumb {
		background: var(--color-overlay-surface);
		border-radius: 2px;
	}
</style>
