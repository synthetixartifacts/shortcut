<script lang="ts">
  /**
   * PromptEditor - Debounced textarea + Reset button for per-action prompt fields.
   *
   * Shared by /actions/{improve,grammar,translate,screen-question} to edit either
   * the user prompt or the system prompt. Parent owns the state; this component
   * is a controlled input that debounces save calls (500ms) and exposes a reset
   * button when the value diverges from `defaultValue`. Per-field save feedback
   * is surfaced via an optional `saveStatus` prop wired into the inner
   * `<FormField>` — the page owns the `SaveStatus` instance on its state module.
   */
  import { Button } from '$lib/components/ui/primitives';
  import { FormField } from '$lib/components/ui/patterns';
  import type { SaveStatus } from '$lib/utils/save-status.svelte';

  interface Props {
    /** Current value (controlled). */
    value: string;
    /** Default value sourced from Rust — used as placeholder and for modified comparison. */
    defaultValue: string;
    /** Field label rendered above the textarea. */
    label: string;
    /** Optional hint shown under the textarea. */
    hint?: string;
    /** Called on every keystroke with the new value. Parent is expected to debounce persistence. */
    onInput: (value: string) => void;
    /** Called when the user clicks the reset button. */
    onReset: () => void | Promise<void>;
    /** Label for the reset button. */
    resetLabel: string;
    /** Optional per-field save-status. When non-idle, a SaveIndicator replaces the hint line. */
    saveStatus?: SaveStatus;
    /** Force Reset to be hidden (e.g. when defaultValue is empty and user clears the field). Default: compare value !== defaultValue. */
    isModified?: boolean | null;
  }

  let {
    value,
    defaultValue,
    label,
    hint,
    onInput,
    onReset,
    resetLabel,
    saveStatus,
    isModified = null,
  }: Props = $props();

  const shouldShowReset = $derived(
    isModified !== null ? isModified : value !== defaultValue,
  );

  function handleInput(e: Event): void {
    onInput((e.target as HTMLTextAreaElement).value);
  }
</script>

<FormField {label} {hint} {saveStatus}>
  <textarea
    class="textarea"
    {value}
    placeholder={defaultValue}
    oninput={handleInput}
  ></textarea>
</FormField>

{#if shouldShowReset}
  <div class="prompt-actions">
    <Button variant="secondary" onclick={onReset}>{resetLabel}</Button>
  </div>
{/if}

<style>
  .textarea {
    width: 100%;
    min-height: 160px;
    padding: var(--spacing-sm) var(--spacing-md);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-sm);
    background: var(--color-surface);
    color: var(--color-text);
    font-family: inherit;
    font-size: 0.85rem;
    line-height: 1.5;
    resize: vertical;
  }
  .textarea:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary-light);
  }
  .prompt-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    margin-top: var(--spacing-md);
  }
</style>
