<script lang="ts">
  /**
   * Button Primitive
   *
   * Reusable button with consistent styling across variants.
   */
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    disabled?: boolean;
    type?: 'button' | 'submit' | 'reset';
    title?: string;
    onclick?: () => void;
    children: Snippet;
  }

  let {
    variant = 'primary',
    disabled = false,
    type = 'button',
    title,
    onclick,
    children
  }: Props = $props();
</script>

<button
  class="btn btn-{variant}"
  {type}
  {disabled}
  {title}
  {onclick}
>
  {@render children()}
</button>

<style>
  .btn {
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: 0.85rem;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    border: 1px solid transparent;
    transition: all 0.15s ease;
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--color-primary);
    color: var(--color-text-on-primary);
    border-color: var(--color-primary);
  }

  .btn-primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-secondary {
    background: var(--color-kbd-bg);
    color: var(--color-text);
    border-color: var(--color-kbd-border);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--color-kbd-border);
  }

  .btn-ghost {
    background: transparent;
    color: var(--color-text-muted);
    border-color: transparent;
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--color-kbd-bg);
    color: var(--color-text);
  }

  .btn-danger {
    background: var(--color-danger);
    color: var(--color-text-on-status);
    border-color: var(--color-danger);
  }

  .btn-danger:hover:not(:disabled) {
    filter: brightness(1.1);
  }
</style>
