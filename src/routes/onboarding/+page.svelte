<script lang="ts">
  /**
   * Onboarding Page — thin composition shell. All state lives in
   * `$lib/state/onboarding.svelte.ts`; each step is its own component under
   * `$lib/components/onboarding/`.
   *
   * Step 1 (llm):      Provider API key entry (LLM only)
   * Step 2 (stt):      STT engine choice + inline local model download
   * Step 3 (complete): Done
   *
   * All steps are skippable. Completing saves settings and navigates home.
   */
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { OnboardingStepper } from '$lib/components/onboarding';
  import { resolvePlatform } from '$lib/state/onboarding.svelte';
  import { t } from '$lib/i18n';

  let pageElement: HTMLDivElement | null = null;

  onMount(() => {
    void resolvePlatform();
  });

  function finish(): void {
    goto('/');
  }
</script>

<svelte:head>
  <title>{t('onboarding.page_title')}</title>
</svelte:head>

<div class="onboarding-page" bind:this={pageElement}>
  <div class="onboarding-container">
    <OnboardingStepper onFinish={finish} {pageElement} />
  </div>
</div>

<style>
  .onboarding-page {
    height: 100vh;
    height: 100dvh;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--color-background);
    padding: var(--spacing-xl) var(--spacing-lg);
  }

  .onboarding-container {
    width: 100%;
    max-width: 880px;
    margin: 0 auto;
    padding-bottom: var(--spacing-xl);
  }

  @media (max-width: 720px) {
    .onboarding-page {
      padding: var(--spacing-xl) var(--spacing-md);
    }
  }
</style>
