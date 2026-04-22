<script lang="ts">
  /**
   * Indicator Window Page
   *
   * Rendered in the floating indicator window.
   * Receives activity state via Tauri events from the main window.
   */

  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { IndicatorWindow } from '$lib/components/indicator';
  import type { ActivityType, ActivityState, ActivityInfo } from '$lib/features/indicator';

  // Reactive state
  let activityType = $state<ActivityType>('processing');
  let activityState = $state<ActivityState>('active');
  let message = $state<string>('');

  let unlistenFn: UnlistenFn | null = null;

  onMount(async () => {
    // Listen for activity updates from main window
    unlistenFn = await listen<ActivityInfo>('indicator-update', (event) => {
      activityType = event.payload.type;
      activityState = event.payload.state;
      message = event.payload.message || '';
    });
  });

  onDestroy(() => {
    unlistenFn?.();
  });
</script>

<svelte:head>
  <title>ShortCut Indicator</title>
</svelte:head>

<main>
  <IndicatorWindow {activityType} {activityState} {message} />
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

  /* Reset the SvelteKit wrapper div */
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
