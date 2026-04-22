<script lang="ts">
  /**
   * Root Layout - App shell with sidebar navigation
   *
   * Handles:
   * - Provider readiness check on startup (replaces auth gate)
   * - First-run redirect to /onboarding when no providers are configured
   * - Sidebar navigation for main app
   * - Special case: indicator/action-menu/screen-question windows without sidebar
   * - Global shortcut and dictation controller initialization
   */
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { Sidebar } from '$lib/components/layout';
  import { getShortcutDispatcher } from '$lib/features/shortcuts';
  import { getDictationController } from '$lib/features/dictation';
  import { providerReadiness, checkProviderReadiness } from '$lib/state/providers.svelte';
  import { loadSettings } from '$lib/state/settings.svelte';
  import { appSettingsState, syncAppSettings } from '$lib/state/app-settings.svelte';
  import { engineState, loadEngineState } from '$lib/state/engine.svelte';
  import { addLogEntry, type LogLevel } from '$lib/state/debug.svelte';
  import { getConfig } from '$lib/api/tauri';
  import { t } from '$lib/i18n';
  import '$lib/styles/app.css';
  import type { Snippet } from 'svelte';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  // Route type checks
  const isIndicator = $derived(page.url.pathname.startsWith('/indicator'));
  const isActionMenu = $derived(page.url.pathname.startsWith('/action-menu'));
  const isScreenQuestion = $derived(page.url.pathname.startsWith('/screen-question'));
  const isOnboarding = $derived(page.url.pathname.startsWith('/onboarding'));

  // Global controllers (must persist across all pages)
  const shortcutDispatcher = getShortcutDispatcher();
  const dictationController = getDictationController();

  // Track if controllers have been initialized
  let controllersInitialized = $state(false);

  // Backend-emitted debug log listener (populates the Debug Logs page)
  let debugLogUnlisten: UnlistenFn | null = null;

  // Initialize controllers once provider check is done and we're in the main app
  $effect(() => {
    // Skip for overlay windows and onboarding
    if (isIndicator || isActionMenu || isScreenQuestion || isOnboarding) return;
    // Skip if already initialized or still checking
    if (controllersInitialized || providerReadiness.isChecking) return;

    (async () => {
      await Promise.all([
        dictationController.initialize(),
        shortcutDispatcher.initialize(),
        loadSettings().then(() => syncAppSettings()),
      ]);
      controllersInitialized = true;
    })();
  });

  // Apply theme to HTML element reactively
  $effect(() => {
    document.documentElement.setAttribute('data-theme', appSettingsState.theme);
  });

  onMount(async () => {
    // Skip everything for overlay windows (they're separate windows)
    if (isIndicator || isActionMenu || isScreenQuestion) return;

    // Forward backend-emitted debug events into the shared log buffer so the
    // Debug Logs page shows the exact prompt JSON sent to providers.
    debugLogUnlisten = await listen<{ level?: string; message: string }>(
      'debug-log',
      (event) => {
        const level = (event.payload.level as LogLevel) || 'info';
        addLogEntry(level, event.payload.message);
      },
    );

    // Skip provider check for onboarding (it manages its own flow)
    if (isOnboarding) return;

    // Fetch engine state FIRST so providerReadiness.local_engine_active reflects
    // the real active engine before checkProviderReadiness() computes
    // stt_configured. Previously this was set BEFORE engine state had loaded,
    // so local-engine users briefly saw stt_configured=false.
    await loadEngineState();
    providerReadiness.local_engine_active =
      engineState.activeEngine === 'local-windows' ||
      engineState.activeEngine === 'local-macos';

    // Check which providers have API keys (local config read, no network)
    await checkProviderReadiness();

    // First-run detection: no providers configured AND first_run not completed
    const config = await getConfig();
    const firstRunCompleted = config.transcription?.first_run_completed ?? false;
    const anyReady =
      providerReadiness.any_llm_configured ||
      providerReadiness.soniox_ready ||
      providerReadiness.local_engine_active;

    if (!firstRunCompleted && !anyReady) {
      goto('/onboarding');
    }
  });

  onDestroy(() => {
    if (debugLogUnlisten) {
      debugLogUnlisten();
      debugLogUnlisten = null;
    }
    // Only cleanup if not an overlay window or onboarding page
    if (!isIndicator && !isActionMenu && !isScreenQuestion && !isOnboarding) {
      dictationController.cleanup();
      shortcutDispatcher.cleanup();
    }
  });
</script>

{#if isIndicator || isActionMenu || isScreenQuestion}
  <!-- Overlay windows: no layout chrome -->
  {@render children()}
{:else if isOnboarding}
  <!-- Onboarding: full screen, no sidebar -->
  {@render children()}
{:else if providerReadiness.isChecking && !providerReadiness.providers_checked}
  <!-- Brief loading state while reading local config -->
  <div class="app-loading">
    <div class="loading-content">
      <div class="loading-spinner"></div>
      <p>{t('common.loading')}</p>
    </div>
  </div>
{:else}
  <!-- Main app — always shown regardless of provider configuration -->
  <div class="app-shell">
    <Sidebar />
    <main class="main-content">
      {@render children()}
    </main>
  </div>
{/if}

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    background: var(--color-background);
  }

  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-xl);
  }

  .app-loading {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-background);
  }

  .loading-content {
    text-align: center;
    color: var(--color-text-muted);
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    margin: 0 auto var(--spacing-md);
    border: 3px solid var(--color-kbd-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
