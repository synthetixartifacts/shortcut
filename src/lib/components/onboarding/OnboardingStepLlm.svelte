<script lang="ts">
  /**
   * OnboardingStepLlm — first step of the wizard. Wraps the shared
   * `OnboardingLlmStep` (the form fields are reused in Settings) with the
   * logo header and the state-driven continue/skip handlers.
   */
  import { OnboardingLlmStep } from '$lib/components/dictation';
  import { ErrorBanner } from '$lib/components/ui/patterns';
  import {
    onboardingState,
    saveLlmAndContinue,
    setStep,
  } from '$lib/state/onboarding.svelte';
  import { t } from '$lib/i18n';

  interface Props {
    onAdvance: () => void;
  }

  let { onAdvance }: Props = $props();

  async function handleContinue(): Promise<void> {
    await saveLlmAndContinue();
    if (onboardingState.step === 'stt') onAdvance();
  }

  function handleSkip(): void {
    setStep('stt');
    onAdvance();
  }
</script>

<div class="onboarding-header onboarding-header--with-logo">
  <img src="/icon.png" alt="ShortCut" class="onboarding-logo" />
  <div class="onboarding-header-copy">
    <h1>{t('onboarding.welcome_title')}</h1>
    <p class="tagline">{t('onboarding.llm_step_subtitle')}</p>
  </div>
</div>

<OnboardingLlmStep
  bind:openaiKey={onboardingState.openaiKey}
  bind:anthropicKey={onboardingState.anthropicKey}
  bind:geminiKey={onboardingState.geminiKey}
  bind:grokKey={onboardingState.grokKey}
  bind:localUrl={onboardingState.localUrl}
  isSaving={onboardingState.isSaving}
  onContinue={handleContinue}
  onSkip={handleSkip}
/>

{#if onboardingState.saveError}
  <ErrorBanner message={onboardingState.saveError} />
{/if}

<style>
  .onboarding-header {
    text-align: center;
    max-width: 560px;
    margin: 0 auto var(--spacing-xl);
  }

  .onboarding-header--with-logo {
    display: flex;
    align-items: center;
    gap: var(--spacing-lg);
    text-align: left;
  }

  .onboarding-header-copy {
    min-width: 0;
  }

  .onboarding-logo {
    height: 72px;
    width: auto;
    flex-shrink: 0;
    margin: 0;
    display: block;
  }

  .onboarding-header h1 {
    margin: 0;
    font-size: 1.5rem;
    color: var(--color-text);
  }

  .tagline {
    margin: var(--spacing-xs) 0 0;
    color: var(--color-text-muted);
    font-size: 0.9rem;
  }

  @media (max-width: 720px) {
    .onboarding-header--with-logo {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
