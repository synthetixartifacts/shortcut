<script lang="ts">
	/**
	 * OverlayChatMessage - Single message bubble
	 *
	 * Renders user messages as plain text, assistant messages as markdown.
	 * Shows a blinking cursor for streaming messages.
	 */

	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import type { ChatMessage } from '$lib/features/overlay-chat/types';

	interface Props {
		message: ChatMessage;
	}

	let { message }: Props = $props();

	// Configure marked for compact output
	marked.setOptions({ breaks: true, gfm: true });

	const renderedHtml = $derived(
		message.role === 'assistant' && message.content
			? DOMPurify.sanitize(marked.parse(message.content) as string, {
					ADD_ATTR: ['target', 'rel']
				})
			: ''
	);
</script>

<div
	class="message"
	class:user={message.role === 'user'}
	class:assistant={message.role === 'assistant'}
>
	{#if message.role === 'assistant'}
		<div class="message-content markdown-body">
			{@html renderedHtml}{#if message.isStreaming}<span class="cursor">|</span>{/if}
		</div>
	{:else}
		<div class="message-content">
			{message.content}
		</div>
	{/if}
</div>

<style>
	.message {
		padding: 8px 12px;
		border-radius: 8px;
		max-width: 90%;
		word-wrap: break-word;
		font-size: 0.85rem;
		line-height: 1.5;
	}

	.user {
		align-self: flex-end;
		background: var(--color-overlay-user-bg);
		color: var(--color-overlay-text);
		white-space: pre-wrap;
	}

	.assistant {
		align-self: flex-start;
		background: var(--color-overlay-assistant-bg);
		color: var(--color-overlay-text);
	}

	.cursor {
		animation: blink 1s step-end infinite;
		color: var(--color-overlay-text-muted);
	}

	@keyframes blink {
		50% {
			opacity: 0;
		}
	}

	/* Markdown body styles */
	.markdown-body :global(h1),
	.markdown-body :global(h2),
	.markdown-body :global(h3) {
		margin: 0.5em 0 0.25em;
		font-weight: 600;
	}
	.markdown-body :global(h1) { font-size: 1.15em; }
	.markdown-body :global(h2) { font-size: 1.05em; }
	.markdown-body :global(h3) { font-size: 1em; }

	.markdown-body :global(p) {
		margin: 0.3em 0;
	}

	.markdown-body :global(ul),
	.markdown-body :global(ol) {
		margin: 0.3em 0;
		padding-left: 1.4em;
	}

	.markdown-body :global(li) {
		margin: 0.15em 0;
	}

	.markdown-body :global(code) {
		background: var(--color-overlay-code-bg);
		padding: 0.15em 0.35em;
		border-radius: 3px;
		font-size: 0.88em;
		font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
	}

	.markdown-body :global(pre) {
		background: var(--color-overlay-code-bg);
		padding: 0.6em;
		border-radius: 6px;
		overflow-x: auto;
		margin: 0.4em 0;
	}

	.markdown-body :global(pre code) {
		background: none;
		padding: 0;
	}

	.markdown-body :global(strong) {
		font-weight: 600;
	}

	.markdown-body :global(a) {
		color: var(--color-primary);
		text-decoration: none;
	}

	.markdown-body :global(blockquote) {
		border-left: 3px solid var(--color-primary);
		margin: 0.4em 0;
		padding: 0.2em 0.6em;
		opacity: 0.85;
	}

	.markdown-body :global(table) {
		border-collapse: collapse;
		width: 100%;
		margin: 0.4em 0;
		font-size: 0.9em;
	}

	.markdown-body :global(th),
	.markdown-body :global(td) {
		border: 1px solid var(--color-overlay-border);
		padding: 0.3em 0.6em;
		text-align: left;
	}

	.markdown-body :global(th) {
		font-weight: 600;
	}

	.markdown-body :global(hr) {
		border: none;
		border-top: 1px solid var(--color-overlay-border);
		margin: 0.5em 0;
	}
</style>
