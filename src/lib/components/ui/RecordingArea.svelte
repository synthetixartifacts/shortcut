<script lang="ts">
  /**
   * RecordingArea Component
   *
   * Visual area for capturing keyboard shortcuts.
   * Shows prompt, live preview, and validation state.
   */

  import { tick } from 'svelte';
  import { t } from '$lib/i18n';

  interface Props {
    isRecording: boolean;
    isActivelyRecording: boolean;
    hasRecording: boolean;
    isValid: boolean;
    livePreview: string;
    onKeyDown: (e: KeyboardEvent) => void;
    onKeyUp: (e: KeyboardEvent) => void;
    onBlur: () => void;
  }

  let {
    isRecording,
    isActivelyRecording,
    hasRecording,
    isValid,
    livePreview,
    onKeyDown,
    onKeyUp,
    onBlur,
  }: Props = $props();

  let inputRef: HTMLDivElement | undefined = $state(undefined);

  // Focus when recording starts
  $effect(() => {
    if (isRecording && inputRef) {
      tick().then(() => inputRef?.focus());
    }
  });

  // Expose focus method for parent
  export function focus() {
    inputRef?.focus();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="recording-area"
  class:has-recording={hasRecording}
  class:is-valid={isValid}
  class:is-invalid={hasRecording && !isValid}
  bind:this={inputRef}
  tabindex="0"
  onkeydown={onKeyDown}
  onkeyup={onKeyUp}
  onblur={onBlur}
  role="textbox"
  aria-label={t('shortcuts.recording_aria')}
>
  {#if !hasRecording && !isActivelyRecording}
    <span class="recording-prompt">{t('shortcuts.recording_prompt')}</span>
    <div class="modifier-hints">
      <kbd class="hint-key">{t('shortcuts.recording_hint_ctrl')}</kbd>
      <span class="hint-plus">+</span>
      <kbd class="hint-key">{t('shortcuts.recording_hint_shift')}</kbd>
      <span class="hint-plus">+</span>
      <kbd class="hint-key">{t('shortcuts.recording_hint_key')}</kbd>
    </div>
  {:else}
    <div class="live-preview">
      <kbd class="shortcut-display" class:valid={isValid} class:invalid={hasRecording && !isValid}>
        {livePreview || '...'}
      </kbd>
    </div>
  {/if}
</div>

<style>
  .recording-area {
    padding: var(--spacing-lg);
    background: var(--color-card-bg);
    border: 2px dashed var(--color-kbd-border);
    border-radius: var(--border-radius-md);
    text-align: center;
    cursor: text;
    transition: all 0.15s ease;
    min-height: 80px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .recording-area:focus {
    outline: none;
    border-style: solid;
    border-color: var(--color-primary);
    background: var(--color-preview-bg);
  }

  .recording-area.has-recording {
    border-style: solid;
  }

  .recording-area.is-valid {
    border-color: var(--color-success);
    background: var(--color-success-light);
  }

  .recording-area.is-invalid {
    border-color: var(--color-danger);
    background: var(--color-danger-light);
  }

  .recording-prompt {
    display: block;
    font-size: 0.9rem;
    color: var(--color-text-muted);
    margin-bottom: var(--spacing-sm);
  }

  .modifier-hints {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    opacity: 0.6;
  }

  .hint-key {
    padding: 2px 8px;
    font-family: monospace;
    font-size: 0.75rem;
    background: var(--color-kbd-bg);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-sm);
  }

  .hint-plus {
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .live-preview {
    display: flex;
    justify-content: center;
  }

  .shortcut-display {
    display: inline-block;
    padding: var(--spacing-sm) var(--spacing-lg);
    font-family: monospace;
    font-size: 1.2rem;
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    border: 1px solid var(--color-kbd-border);
    min-width: 120px;
    transition: all 0.15s ease;
  }

  .shortcut-display.valid {
    border-color: var(--color-success);
    background: var(--color-success-light);
  }

  .shortcut-display.invalid {
    border-color: var(--color-danger);
    background: var(--color-danger-light);
  }
</style>
