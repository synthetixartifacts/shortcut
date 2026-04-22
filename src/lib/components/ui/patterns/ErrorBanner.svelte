<script lang="ts">
  /**
   * ErrorBanner - Displays an error message in a styled banner
   */
  interface Props {
    message: string;
  }

  let { message }: Props = $props();
</script>

{#if message}
  <div class="error-banner" role="alert">
    {message}
  </div>
{/if}

<style>
  .error-banner {
    padding: var(--spacing-md);
    background: var(--color-danger-light);
    border: 1px solid var(--color-danger);
    border-radius: var(--border-radius-md);
    color: var(--color-danger);
    margin-bottom: var(--spacing-lg);
  }
</style>
