<script lang="ts">
  /**
   * ValidationMessage Component
   *
   * Displays validation feedback with icon and styled message.
   */

  import { Icon } from './primitives';

  interface Props {
    type: 'success' | 'error';
    message: string;
  }

  let { type, message }: Props = $props();
</script>

<div class="validation-message {type}">
  {#if type === 'success'}
    <Icon name="check" size={14} />
  {:else}
    <Icon name="warning" size={14} />
  {/if}
  <span>{message}</span>
</div>

<style>
  .validation-message {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--border-radius-sm);
    font-size: 0.8rem;
  }

  .validation-message.success {
    background: var(--color-success-light);
    color: var(--color-success);
    border: 1px solid var(--color-success);
  }

  .validation-message.error {
    background: var(--color-danger-light);
    color: var(--color-danger);
    border: 1px solid var(--color-danger);
  }
</style>
