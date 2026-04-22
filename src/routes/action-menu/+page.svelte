<script lang="ts">
	/**
	 * Action Menu Window Page
	 *
	 * Rendered in the floating action-menu window.
	 * Displays a radial pie menu for action selection.
	 * Receives show events and emits action selections via Tauri events.
	 */

	import { onMount, onDestroy } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { PieMenu } from '$lib/components/action-menu';
	import { MENU_ITEMS, AUTO_DISMISS_MS, selectAction } from '$lib/features/action-menu';
	import type { ShortcutAction } from '$lib/types';

	let unlistenFn: UnlistenFn | null = null;
	let dismissTimer: ReturnType<typeof setTimeout> | null = null;

	function resetDismissTimer(): void {
		if (dismissTimer) clearTimeout(dismissTimer);
		dismissTimer = setTimeout(() => {
			selectAction('');
		}, AUTO_DISMISS_MS);
	}

	function handleBackgroundClick(event: PointerEvent): void {
		// Only dismiss if the click is on the background, not on a wedge
		if (event.target === event.currentTarget) {
			selectAction('');
		}
	}

	function handleSelect(action: ShortcutAction): void {
		if (dismissTimer) clearTimeout(dismissTimer);
		if (action) {
			selectAction(action);
		}
	}

	onMount(async () => {
		unlistenFn = await listen('action-menu-show', () => {
			resetDismissTimer();
		});
		// Start dismiss timer immediately (window is shown on creation)
		resetDismissTimer();
	});

	onDestroy(() => {
		unlistenFn?.();
		if (dismissTimer) clearTimeout(dismissTimer);
	});
</script>

<svelte:head>
	<title>ShortCut Action Menu</title>
</svelte:head>

<main onpointerdown={handleBackgroundClick} onmousemove={resetDismissTimer}>
	<PieMenu items={MENU_ITEMS} onSelect={handleSelect} />
</main>

<style>
	:global(html) {
		margin: 0;
		padding: 0;
		background: transparent !important;
		background-color: transparent !important;
		overflow: hidden;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		background: transparent !important;
		background-color: transparent !important;
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
		padding: 0;
		margin: 0;
		background: transparent !important;
	}
</style>
