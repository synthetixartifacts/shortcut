<script lang="ts">
  /**
   * Input - Base input primitive
   *
   * Low-level input component that handles all input variants.
   * Use FormField pattern for labeled inputs.
   */
  interface Props {
    type?: 'text' | 'password' | 'email' | 'url';
    value: string;
    placeholder?: string;
    disabled?: boolean;
    id?: string;
    monospace?: boolean;
    onchange?: (value: string) => void;
    onkeydown?: (event: KeyboardEvent) => void;
    onfocus?: () => void;
    onblur?: () => void;
  }

  let {
    type = 'text',
    value,
    placeholder,
    disabled = false,
    id,
    monospace = false,
    onchange,
    onkeydown,
    onfocus,
    onblur,
  }: Props = $props();

  function handleInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    onchange?.(target.value);
  }
</script>

<input
  {type}
  {value}
  {placeholder}
  {disabled}
  {id}
  class="input"
  class:monospace
  oninput={handleInput}
  {onkeydown}
  {onfocus}
  {onblur}
/>

<style>
  .input {
    width: 100%;
    height: var(--input-height, 40px);
    padding: var(--input-padding, var(--spacing-sm) var(--spacing-md));
    border: var(--input-border, 1px solid var(--color-kbd-border));
    border-radius: var(--input-radius, var(--border-radius-sm));
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.9rem;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
    box-sizing: border-box;
  }

  .input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary-light);
  }

  .input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .input.monospace {
    font-family: monospace;
  }

  .input::placeholder {
    color: var(--color-text-hint);
  }
</style>
