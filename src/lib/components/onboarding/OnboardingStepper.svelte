<script lang="ts">
  /**
   * OnboardingStepper — renders the current step and wires the scroll-to-top
   * behaviour that fires on every step transition.
   *
   * Keeps the route shell tiny: it just mounts `<OnboardingStepper />` with
   * a goto callback and the state machine takes care of the rest.
   */
  import OnboardingStepLlm from './OnboardingStepLlm.svelte';
  import OnboardingStepStt from './OnboardingStepStt.svelte';
  import OnboardingStepComplete from './OnboardingStepComplete.svelte';
  import { onboardingState } from '$lib/state/onboarding.svelte';

  interface Props {
    onFinish: () => void;
    pageElement?: HTMLElement | null;
  }

  let { onFinish, pageElement = null }: Props = $props();

  function scrollTop(): void {
    if (pageElement) {
      pageElement.scrollTo({ top: 0 });
      return;
    }
    window.scrollTo({ top: 0 });
  }
</script>

{#if onboardingState.step === 'llm'}
  <OnboardingStepLlm onAdvance={scrollTop} />
{:else if onboardingState.step === 'stt'}
  <OnboardingStepStt
    onAdvance={scrollTop}
    onBack={scrollTop}
    onSkipAll={onFinish}
  />
{:else}
  <OnboardingStepComplete onFinish={onFinish} onBack={scrollTop} />
{/if}
