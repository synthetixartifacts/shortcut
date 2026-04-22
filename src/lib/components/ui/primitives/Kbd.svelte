<script lang="ts">
  /**
   * Kbd Primitive
   *
   * Displays keyboard keys/shortcuts with consistent styling.
   */

  interface Props {
    keys: string | string[];
    class?: string;
  }

  let { keys, class: className = '' }: Props = $props();

  const displayText = $derived(
    Array.isArray(keys) ? keys.join(' + ') : keys
  );
</script>

<kbd class="kbd {className}">{displayText}</kbd>

<style>
  .kbd {
    display: inline-block;
    padding: var(--spacing-xs) var(--spacing-sm);
    font-family: monospace;
    font-size: 0.8rem;
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    border: 1px solid var(--color-kbd-border);
    min-width: 1.5em;
    text-align: center;
  }
</style>
