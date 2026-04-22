<script lang="ts">
  /**
   * ShortcutRecorder Component
   *
   * Captures keyboard shortcuts from user input.
   * Requirements: 1-3 modifiers + 1 key (2-4 key combination)
   *
   * Refactored to use:
   * - createShortcutRecorder for state management
   * - RecordingArea for the recording UI
   * - ValidationMessage for feedback
   * - Button primitive for actions
   */

  import { Button, Icon } from './primitives';
  import RecordingArea from './RecordingArea.svelte';
  import ValidationMessage from './ValidationMessage.svelte';
  import {
    createShortcutRecorder,
    type ShortcutRecorderState,
  } from '$lib/features/shortcuts/shortcut-recorder.svelte';
  import { shortcutToDisplay } from '$lib/features/shortcuts';
  import { t } from '$lib/i18n';

  interface Props {
    value: string;
    defaultValue: string;
    onSave: (shortcut: string) => void;
    onReset: () => void;
    disabled?: boolean;
  }

  let { value, defaultValue, onSave, onReset, disabled = false }: Props = $props();

  // Create recorder state machine
  const recorder: ShortcutRecorderState = createShortcutRecorder();

  // Reference to recording area for focus management
  let recordingAreaRef: { focus: () => void } | undefined = $state(undefined);

  // Derived display values
  const displayValue = $derived(shortcutToDisplay(value, recorder.platform));
  const displayDefault = $derived(shortcutToDisplay(defaultValue, recorder.platform));
  const isDefault = $derived(value === defaultValue);

  function startRecording(): void {
    if (disabled) return;
    recorder.startRecording();
  }

  function clearRecording(): void {
    recorder.clearRecording();
    recordingAreaRef?.focus();
  }

  function cancelRecording(): void {
    recorder.cancelRecording();
  }

  function saveRecording(): void {
    if (recorder.recordedShortcut && recorder.isValid) {
      onSave(recorder.recordedShortcut);
      recorder.finishRecording();
    }
  }

  function handleReset(): void {
    onReset();
  }
</script>

<div class="shortcut-recorder" class:disabled class:recording={recorder.isRecording}>
  {#if recorder.mode === 'display'}
    <!-- Display Mode -->
    <div class="current-shortcut">
      <span class="label">{t('shortcuts.recorder_current')}</span>
      <kbd class="shortcut-display">{displayValue}</kbd>
      {#if !isDefault}
        <span class="modified-badge">{t('shortcuts.recorder_modified')}</span>
      {/if}
    </div>

    <Button variant="primary" onclick={startRecording} disabled={disabled}>
      {t('shortcuts.recorder_record')}
    </Button>

    <div class="actions">
      <Button variant="secondary" onclick={handleReset} disabled={disabled || isDefault}>
        {t('shortcuts.recorder_reset')}
      </Button>
    </div>

    {#if !isDefault}
      <div class="default-hint">
        {t('shortcuts.recorder_default_hint')} <kbd>{displayDefault}</kbd>
      </div>
    {/if}
  {:else}
    <!-- Recording Mode -->
    <div class="requirement-badge">
      <span class="requirement-icon">{t('shortcuts.recorder_requirement_badge')}</span>
      <span class="requirement-text">{t('shortcuts.recorder_requirement_text')}</span>
    </div>

    <RecordingArea
      bind:this={recordingAreaRef}
      isRecording={recorder.isRecording}
      isActivelyRecording={recorder.isActivelyRecording}
      hasRecording={recorder.recordedShortcut !== null}
      isValid={recorder.isValid}
      livePreview={recorder.livePreview}
      onKeyDown={recorder.handleKeyDown}
      onKeyUp={recorder.handleKeyUp}
      onBlur={recorder.handleBlur}
    />

    {#if recorder.validationMessage}
      <ValidationMessage
        type={recorder.validationMessage.type}
        message={recorder.validationMessage.text}
      />
    {/if}

    <div class="recording-actions">
      <Button
        variant="ghost"
        onclick={clearRecording}
        disabled={!recorder.recordedShortcut}
        title={t('shortcuts.recorder_clear_title')}
      >
        <Icon name="refresh" size={14} />
        {t('shortcuts.recorder_clear')}
      </Button>
      <div class="action-spacer"></div>
      <Button variant="secondary" onclick={cancelRecording}>
        {t('common.cancel')}
      </Button>
      <Button variant="primary" onclick={saveRecording} disabled={!recorder.isValid}>
        {t('common.save')}
      </Button>
    </div>
  {/if}
</div>

<style>
  .shortcut-recorder {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .shortcut-recorder.disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  .current-shortcut {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-wrap: wrap;
  }

  .label {
    font-size: 0.9rem;
    color: var(--color-text-muted);
  }

  .shortcut-display {
    display: inline-block;
    padding: var(--spacing-xs) var(--spacing-md);
    font-family: monospace;
    font-size: 0.9rem;
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    border: 1px solid var(--color-kbd-border);
  }

  .modified-badge {
    font-size: 0.7rem;
    padding: 2px 6px;
    background: var(--color-primary);
    color: var(--color-text-on-primary);
    border-radius: var(--border-radius-sm);
  }

  .actions {
    display: flex;
    gap: var(--spacing-sm);
  }

  .default-hint {
    font-size: 0.8rem;
    color: var(--color-text-hint);
  }

  .default-hint kbd {
    font-size: 0.75rem;
    padding: 2px 6px;
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    border: 1px solid var(--color-kbd-border);
  }

  .requirement-badge {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-primary-light);
    border: 1px solid var(--color-primary-border);
    border-radius: var(--border-radius-sm);
    font-size: 0.8rem;
  }

  .requirement-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 6px;
    min-width: 20px;
    height: 20px;
    background: var(--color-primary);
    color: var(--color-text-on-primary);
    border-radius: 10px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .requirement-text {
    color: var(--color-text-muted);
  }

  .recording-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .action-spacer {
    flex: 1;
  }
</style>
