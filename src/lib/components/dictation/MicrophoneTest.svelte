<script lang="ts">
  /**
   * MicrophoneTest - Test microphone recording with visual feedback
   *
   * Provides a button to run a short test recording with
   * real-time audio level visualization and result display.
   */
  import { runMicrophoneTest, type MicrophoneTestResult } from '$lib/features/dictation/microphone-test';
  import { Button, Icon } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';

  // State
  let isRunning = $state(false);
  let currentLevel = $state(0);
  let elapsed = $state(0);
  let result = $state<MicrophoneTestResult | null>(null);

  const TEST_DURATION = 3000; // 3 seconds

  async function handleTest() {
    isRunning = true;
    result = null;
    currentLevel = 0;
    elapsed = 0;

    try {
      const testResult = await runMicrophoneTest(TEST_DURATION, (level, ms) => {
        currentLevel = level;
        elapsed = ms;
      });

      result = testResult;
    } finally {
      isRunning = false;
      currentLevel = 0;
    }
  }

  // Derived values
  const progressPercent = $derived(Math.min(100, (elapsed / TEST_DURATION) * 100));
  const levelPercent = $derived(Math.min(100, currentLevel * 100));
</script>

<div class="microphone-test">
  <div class="test-header">
    <span class="test-label label-caps">{t('mic.test_label')}</span>
    {#if !isRunning && !result}
      <span class="test-hint">{t('mic.test_hint')}</span>
    {/if}
  </div>

  {#if isRunning}
    <div class="test-running">
      <div class="progress-bar">
        <div class="progress-fill" style="width: {progressPercent}%"></div>
      </div>
      <div class="level-meter">
        <div class="level-label">{t('mic.test_level')}</div>
        <div class="level-bar-container">
          <div
            class="level-bar"
            class:level-low={levelPercent < 20}
            class:level-medium={levelPercent >= 20 && levelPercent < 60}
            class:level-good={levelPercent >= 60}
            style="width: {levelPercent}%"
          ></div>
        </div>
      </div>
      <p class="test-instruction">{t('mic.test_instruction')}</p>
    </div>
  {:else if result}
    <div class="test-result" class:success={result.success} class:failure={!result.success}>
      <div class="result-icon">
        <Icon name={result.success ? 'check' : 'close'} size={18} />
      </div>
      <div class="result-content">
        <p class="result-message">{result.message}</p>
        {#if result.success}
          <p class="result-details">
            {t('mic.test_peak')} {Math.round(result.peakLevel * 100)}% |
            {t('mic.test_average')} {Math.round(result.averageLevel * 100)}%
          </p>
        {/if}
      </div>
    </div>
    <Button variant="secondary" onclick={handleTest}>
      {t('mic.test_again')}
    </Button>
  {:else}
    <Button variant="secondary" onclick={handleTest}>
      {t('mic.test_run')}
    </Button>
  {/if}
</div>

<style>
  .microphone-test {
    padding: var(--spacing-md);
    background: var(--color-surface);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-md);
  }

  .test-header {
    margin-bottom: var(--spacing-md);
  }

  .test-label {
    display: block;
    margin-bottom: var(--spacing-xs);
  }

  .test-hint {
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  /* Running state */
  .test-running {
    margin-bottom: var(--spacing-md);
  }

  .progress-bar {
    height: 4px;
    background: var(--color-kbd-border);
    border-radius: 2px;
    overflow: hidden;
    margin-bottom: var(--spacing-md);
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary);
    transition: width 0.1s linear;
  }

  .level-meter {
    margin-bottom: var(--spacing-sm);
  }

  .level-label {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    margin-bottom: var(--spacing-xs);
  }

  .level-bar-container {
    height: 24px;
    background: var(--color-background);
    border-radius: var(--border-radius-sm);
    overflow: hidden;
  }

  .level-bar {
    height: 100%;
    transition: width 0.05s ease-out;
    border-radius: var(--border-radius-sm);
  }

  .level-low {
    background: var(--color-text-muted);
  }

  .level-medium {
    background: var(--color-warning);
  }

  .level-good {
    background: var(--color-success);
  }

  .test-instruction {
    margin: var(--spacing-sm) 0 0;
    font-size: 0.9rem;
    color: var(--color-text);
    text-align: center;
  }

  /* Result state */
  .test-result {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    border-radius: var(--border-radius-sm);
    margin-bottom: var(--spacing-md);
  }

  .test-result.success {
    background: var(--color-success-light);
  }

  .test-result.failure {
    background: var(--color-danger-light);
  }

  .result-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    flex-shrink: 0;
  }

  .success .result-icon {
    background: var(--color-success);
    color: var(--color-text-on-status);
  }

  .failure .result-icon {
    background: var(--color-danger);
    color: var(--color-text-on-status);
  }

  .result-content {
    flex: 1;
  }

  .result-message {
    margin: 0;
    font-weight: 500;
  }

  .success .result-message {
    color: var(--color-success);
  }

  .failure .result-message {
    color: var(--color-danger);
  }

  .result-details {
    margin: var(--spacing-xs) 0 0;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }
</style>
