<script lang="ts">
  /**
   * OnboardingStepStt — STT engine radiogroup + inline Parakeet download for
   * Windows users. Mirrors the ARIA radiogroup semantics of the original
   * route markup.
   */
  import { ModelDownload } from '$lib/components/dictation';
  import { ErrorBanner } from '$lib/components/ui/patterns';
  import {
    onboardingState,
    continueStt,
    selectSttEngine,
    skipAll,
    setStep,
    type SttEngineChoice,
  } from '$lib/state/onboarding.svelte';
  import { t } from '$lib/i18n';

  interface Props {
    onAdvance: () => void;
    onBack: () => void;
    onSkipAll: () => void;
  }

  let { onAdvance, onBack, onSkipAll }: Props = $props();

  const needsLocalDownload = $derived(
    onboardingState.selectedSttEngine === 'local-windows' && onboardingState.platform === 'windows'
  );

  function handleChoiceKeydown(event: KeyboardEvent, engine: SttEngineChoice): void {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    selectSttEngine(engine);
  }

  async function handleContinue(): Promise<void> {
    await continueStt();
    if (onboardingState.step === 'complete') onAdvance();
  }

  function handleBack(): void {
    setStep('llm');
    onBack();
  }

  async function handleSkipAll(): Promise<void> {
    await skipAll();
    onSkipAll();
  }

  function handleModelDownloadComplete(): void {
    onboardingState.localModelReady = true;
    onboardingState.sttError = null;
  }

  function handleModelDownloadCancel(): void {
    onboardingState.localModelReady = false;
  }
</script>

<div class="onboarding-header">
  <h1>{t('onboarding.stt_step_title')}</h1>
  <p class="tagline">{t('onboarding.stt_step_subtitle')}</p>
</div>

<div class="engine-choices" role="radiogroup" aria-label={t('onboarding.stt_step_title')}>
  <div
    class:selected={onboardingState.selectedSttEngine === 'soniox'}
    class="choice-card"
    role="radio"
    aria-checked={onboardingState.selectedSttEngine === 'soniox'}
    tabindex="0"
    onclick={() => selectSttEngine('soniox')}
    onkeydown={(event) => handleChoiceKeydown(event, 'soniox')}
  >
    <div class="choice-icon">&#9729;</div>
    <h2>{t('onboarding.cloud_title')}</h2>
    <p class="choice-desc">{t('providers.soniox_desc')}</p>
    <div class="choice-field">
      <span class="choice-field-label">{t('settings.field_soniox_key')}</span>
      <p class="choice-field-hint">{t('onboarding.stt_key_hint')}</p>
      <input
        class="choice-input"
        type="password"
        placeholder={t('settings.field_soniox_key_placeholder')}
        bind:value={onboardingState.sonioxKey}
        onfocus={() => selectSttEngine('soniox')}
      />
    </div>
    <span class="choice-hint">{t('onboarding.cloud_hint')}</span>
  </div>
  {#if onboardingState.platform === 'windows'}
    <div
      class:selected={onboardingState.selectedSttEngine === 'local-windows'}
      class="choice-card"
      role="radio"
      aria-checked={onboardingState.selectedSttEngine === 'local-windows'}
      tabindex="0"
      onclick={() => selectSttEngine('local-windows')}
      onkeydown={(event) => handleChoiceKeydown(event, 'local-windows')}
    >
      <div class="choice-icon">&#128187;</div>
      <h2>{t('onboarding.local_title')}</h2>
      <p class="choice-desc">{t('onboarding.local_desc')}</p>
      <span class="choice-hint">{t('onboarding.local_hint')}</span>
    </div>
  {:else}
    <div class="choice-card disabled" role="presentation">
      <div class="choice-icon">&#128187;</div>
      <h2>{t('onboarding.local_title')}</h2>
      <p class="choice-desc">{t('onboarding.local_coming_soon')}</p>
    </div>
  {/if}
</div>

{#if needsLocalDownload}
  <div class="download-inline-step">
    <ModelDownload
      onComplete={handleModelDownloadComplete}
      onCancel={handleModelDownloadCancel}
    />
  </div>
{/if}

{#if onboardingState.sttError}
  <ErrorBanner message={onboardingState.sttError} />
{/if}

<div class="actions">
  <button
    class="btn-primary"
    onclick={handleContinue}
    disabled={onboardingState.isSttSaving || (needsLocalDownload && !onboardingState.localModelReady)}
  >
    {onboardingState.isSttSaving ? t('common.saving') : t('onboarding.continue')}
  </button>
</div>
<div class="secondary-actions">
  <button class="btn-skip" onclick={handleBack}>
    {t('onboarding.stt_back')}
  </button>
  <button class="btn-skip" onclick={handleSkipAll}>{t('onboarding.configure_later')}</button>
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

  .engine-choices {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .download-inline-step {
    margin-bottom: var(--spacing-lg);
  }

  .choice-card {
    background: var(--color-surface);
    border: 2px solid var(--color-kbd-border);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-xl) var(--spacing-lg);
    cursor: pointer;
    text-align: left;
    transition: all 0.2s ease;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--spacing-sm);
    min-height: 100%;
  }

  .choice-card.selected {
    border-color: var(--color-primary);
    box-shadow: var(--shadow-card);
    background: color-mix(in srgb, var(--color-primary) 6%, var(--color-surface));
  }

  .choice-card:not(.disabled):hover {
    border-color: var(--color-primary);
    box-shadow: var(--shadow-card);
    transform: translateY(-2px);
  }

  .choice-card:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .choice-card.disabled { opacity: 0.5; cursor: default; }
  .choice-icon { font-size: 2rem; line-height: 1; }
  .choice-card h2 { margin: 0; font-size: 1.1rem; color: var(--color-text); }
  .choice-desc { margin: 0; font-size: 0.85rem; color: var(--color-text-muted); line-height: 1.4; }
  .choice-field { width: 100%; display: flex; flex-direction: column; gap: 4px; margin-top: var(--spacing-xs); }
  .choice-field-label { font-size: 0.85rem; font-weight: 600; color: var(--color-text); }
  .choice-field-hint { margin: 0; font-size: 0.8rem; color: var(--color-text-muted); line-height: 1.45; }

  .choice-input {
    width: 100%;
    padding: 6px var(--spacing-sm);
    font-size: 0.85rem;
    font-family: monospace;
    background: var(--color-background);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-sm);
    color: var(--color-text);
  }
  .choice-input:focus { outline: none; border-color: var(--color-primary); }

  .choice-hint {
    margin-top: auto;
    padding-top: var(--spacing-sm);
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--color-primary);
  }

  .actions { display: flex; flex-direction: column; align-items: center; gap: var(--spacing-sm); }

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

  .secondary-actions { display: flex; flex-direction: column; align-items: center; gap: var(--spacing-xs); }

  @media (max-width: 720px) {
    .engine-choices { grid-template-columns: 1fr; }
  }
</style>
