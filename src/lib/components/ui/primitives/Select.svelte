<script lang="ts">
  /**
   * Select - Base select primitive
   *
   * Low-level dropdown component mirroring the Input primitive conventions:
   * unidirectional value (no $bindable), options passed as an array, and
   * consumers provide an `onchange` callback for the new value.
   */
  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    value: string;
    options: Option[];
    disabled?: boolean;
    id?: string;
    ariaLabel?: string;
    onchange?: (value: string) => void;
  }

  let {
    value,
    options,
    disabled = false,
    id,
    ariaLabel,
    onchange,
  }: Props = $props();

  function handleChange(event: Event): void {
    const target = event.target as HTMLSelectElement;
    onchange?.(target.value);
  }
</script>

<select
  {value}
  {disabled}
  {id}
  aria-label={ariaLabel}
  class="select"
  onchange={handleChange}
>
  {#each options as option (option.value)}
    <option value={option.value}>{option.label}</option>
  {/each}
</select>

<style>
  .select {
    width: 100%;
    height: var(--input-height, 40px);
    padding: var(--input-padding, var(--spacing-sm) var(--spacing-md));
    border: var(--input-border, 1px solid var(--color-kbd-border));
    border-radius: var(--input-radius, var(--border-radius-sm));
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.9rem;
    cursor: pointer;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
    box-sizing: border-box;
  }

  .select:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary-light);
  }

  .select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
