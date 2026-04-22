<script lang="ts">
	/**
	 * Screen Question Window Page
	 *
	 * Rendered in the floating screen-question overlay window.
	 * Listens for "screen-captured" event from Rust, then displays
	 * the OverlayChat component with the screenshot context.
	 */

	import { onMount, onDestroy } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { OverlayChat } from '$lib/components/overlay-chat';
	import { clearConversation } from '$lib/state/overlay-chat.svelte';
	import { hideScreenQuestion } from '$lib/api/tauri';
	import { appSettingsState, syncAppSettings } from '$lib/state/app-settings.svelte';
	import { loadSettings } from '$lib/state/settings.svelte';
	import { t } from '$lib/i18n';
	import type { ChatContext, OverlayChatConfig } from '$lib/features/overlay-chat/types';

	let imageBase64 = $state('');
	let imageMimeType = $state('image/jpeg');
	let isReady = $state(false);
	let unlistenCapture: UnlistenFn | null = null;
	let captureTimeout: ReturnType<typeof setTimeout> | null = null;

	/** Max time to wait for screen-captured event before closing (e.g. after refresh) */
	const CAPTURE_TIMEOUT_MS = 5000;

	/** Chat context: screenshot from Rust */
	const context = $derived<ChatContext>({
		type: 'screenshot',
		imageBase64,
		imageMimeType
	});

	/** Chat configuration for screen analysis (placeholder is i18n-reactive) */
	const chatConfig = $derived<OverlayChatConfig>({
		placeholder: t('screen_question.placeholder'),
		chunkEvent: 'screen-answer-chunk',
		completeEvent: 'screen-answer-complete',
		errorEvent: 'screen-answer-error',
		sendCommand: 'send_screen_question'
	});

	async function handleClose(): Promise<void> {
		clearConversation();
		imageBase64 = '';
		isReady = false;
		try {
			await hideScreenQuestion();
		} catch (e) {
			console.error('Failed to hide screen question:', e);
		}
	}

	onMount(async () => {
		// Load settings from backend — this overlay skips the main layout init
		await loadSettings();
		syncAppSettings();
		document.documentElement.setAttribute('data-theme', appSettingsState.theme);

		unlistenCapture = await listen<{ image_base64: string; image_mime_type: string }>(
			'screen-captured',
			(event) => {
				if (captureTimeout) clearTimeout(captureTimeout);
				clearConversation();
				imageBase64 = event.payload.image_base64;
				imageMimeType = event.payload.image_mime_type;
				isReady = true;
			}
		);

		// Safety: if no capture event arrives (e.g. page was refreshed), close the window
		captureTimeout = setTimeout(() => {
			if (!isReady) {
				console.warn('No screen-captured event received, closing window');
				handleClose();
			}
		}, CAPTURE_TIMEOUT_MS);
	});

	onDestroy(() => {
		unlistenCapture?.();
		if (captureTimeout) clearTimeout(captureTimeout);
	});
</script>

<svelte:head>
	<title>ShortCut Screen Question</title>
</svelte:head>

<main>
	{#if isReady}
		<OverlayChat {context} config={chatConfig} onClose={handleClose} />
	{:else}
		<div class="loading-state">
			<div class="loading-content">
				<div class="spinner"></div>
				<p class="loading-text">{t('screen_question.capturing')}</p>
			</div>
		</div>
	{/if}
</main>

<style>
	:global(html) {
		margin: 0;
		padding: 0;
		background: transparent !important;
		overflow: hidden;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		background: transparent !important;
		overflow: hidden;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
	}

	:global(body > div) {
		background: transparent !important;
	}

	main {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100vh;
		width: 100vw;
		padding: 10px;
		margin: 0;
		box-sizing: border-box;
	}

	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		max-width: 500px;
		background: var(--color-overlay-bg);
		border: 1px solid var(--color-overlay-border);
		border-radius: 12px;
		padding: 40px 20px;
		box-shadow: var(--shadow-modal);
	}

	.loading-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
	}

	.loading-text {
		color: var(--color-overlay-text-muted);
		font-size: 13px;
		margin: 0;
	}

	.spinner {
		width: 28px;
		height: 28px;
		border: 2px solid var(--color-overlay-border);
		border-top-color: var(--color-primary);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
