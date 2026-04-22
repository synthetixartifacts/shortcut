<script lang="ts">
  /**
   * FormField - Label + input wrapper with error/hint/save-status handling.
   *
   * Provides consistent layout for form inputs with labels. When a
   * `saveStatus` prop is supplied AND its current status is not `idle`,
   * the inline <SaveIndicator> replaces the hint/error message line.
   */
  import type { Snippet } from 'svelte';
  import SaveIndicator from './SaveIndicator.svelte';
  import type { SaveStatus } from '$lib/utils/save-status.svelte';

  interface Props {
    label: string;
    id?: string;
    error?: string | null;
    hint?: string;
    saveStatus?: SaveStatus;
    children: Snippet;
  }

  let { label, id, error, hint, saveStatus, children }: Props = $props();

  // Generate ID if not provided
  const fieldId = $derived(id || `field-${label.toLowerCase().replace(/\s+/g, '-')}`);
</script>

<div class="form-field" class:has-error={!!error}>
  <label class="field-label" for={fieldId}>
    {label}
  </label>

  <div class="field-input">
    {@render children()}
  </div>

  {#if saveStatus && saveStatus.status !== 'idle'}
    <div class="field-message">
      <SaveIndicator status={saveStatus} />
    </div>
  {:else if error}
    <p class="field-error">{error}</p>
  {:else if hint}
    <p class="field-hint">{hint}</p>
  {/if}
</div>

<style>
  .form-field {
    margin-bottom: var(--form-field-gap, var(--spacing-md));
  }

  .field-label {
    display: block;
    margin-bottom: var(--spacing-xs);
    font-size: var(--form-label-size, 0.85rem);
    font-weight: 500;
    color: var(--color-text);
  }

  .field-input {
    display: flex;
    gap: var(--spacing-xs);
  }

  .field-message {
    margin: var(--spacing-xs) 0 0;
  }

  .field-error {
    margin: var(--spacing-xs) 0 0;
    font-size: 0.8rem;
    color: var(--color-danger);
  }

  .field-hint {
    margin: var(--spacing-xs) 0 0;
    font-size: 0.8rem;
    color: var(--color-text-hint);
  }

  .has-error :global(input) {
    border-color: var(--color-danger);
  }
</style>
