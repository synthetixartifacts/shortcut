<script lang="ts">
  /**
   * OnboardingStepComplete — success screen with "Open app" + "Back" buttons.
   */
  import { setStep } from '$lib/state/onboarding.svelte';
  import { t } from '$lib/i18n';

  interface Props {
    onFinish: () => void;
    onBack: () => void;
  }

  let { onFinish, onBack }: Props = $props();

  function handleBack(): void {
    setStep('stt');
    onBack();
  }
</script>

<div class="onboarding-header complete-header">
  <div class="complete-icon">&#9989;</div>
  <h1>{t('onboarding.complete_title')}</h1>
  <p class="tagline">{t('onboarding.complete_subtitle')}</p>
</div>

<div class="actions">
  <button class="btn-primary" onclick={onFinish}>{t('onboarding.open_app')}</button>
  <button class="btn-skip" onclick={handleBack}>
    {t('onboarding.complete_back')}
  </button>
</div>

<style>
  .onboarding-header {
    text-align: center;
    max-width: 560px;
    margin: 0 auto var(--spacing-xl);
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

  .complete-header { padding: var(--spacing-xl) 0; }
  .complete-icon { font-size: 3rem; margin-bottom: var(--spacing-md); }

  .actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .btn-primary {
    width: 100%;
    max-width: 320px;
    padding: 10px var(--spacing-lg);
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--color-text-on-primary);
    background: var(--color-primary);
    border: none;
    border-radius: var(--border-radius-md);
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .btn-primary:hover:not(:disabled) { opacity: 0.9; }
  .btn-primary:disabled { opacity: 0.5; cursor: default; }

  .btn-skip {
    background: none;
    border: none;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    cursor: pointer;
    padding: var(--spacing-sm);
    text-decoration: underline;
    transition: color 0.15s ease;
    display: block;
    margin: var(--spacing-sm) auto 0;
  }

  .btn-skip:hover { color: var(--color-text); }
</style>
