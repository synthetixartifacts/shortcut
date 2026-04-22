<script lang="ts">
  /**
   * Card - Reusable card pattern
   *
   * Base card component with variants for different use cases.
   */
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'default' | 'interactive' | 'status';
    padding?: 'sm' | 'md' | 'lg';
    active?: boolean;
    children: Snippet;
  }

  let {
    variant = 'default',
    padding = 'lg',
    active = false,
    children,
  }: Props = $props();
</script>

<div class="card card-{variant} padding-{padding}" class:active>
  {@render children()}
</div>

<style>
  .card {
    background: var(--color-card-bg);
    border-radius: var(--card-radius, var(--border-radius-lg));
    box-shadow: var(--card-shadow, var(--shadow-card));
    transition: all 0.15s ease;
  }

  /* Padding variants */
  .padding-sm { padding: var(--spacing-sm); }
  .padding-md { padding: var(--spacing-md); }
  .padding-lg { padding: var(--spacing-lg); }

  /* Variant: Interactive */
  .card-interactive {
    cursor: pointer;
    border: 1px solid transparent;
  }

  .card-interactive:hover {
    background: var(--color-primary-light);
    border-color: var(--color-primary-border);
  }

  .card-interactive.active {
    border-color: var(--color-primary);
  }

  /* Variant: Status */
  .card-status {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .card-status.active {
    background: var(--color-card-recording-bg);
    border: 2px solid var(--color-danger);
  }
</style>
