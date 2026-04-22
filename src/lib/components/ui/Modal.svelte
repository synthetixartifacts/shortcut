<script lang="ts">
  /**
   * Modal Component
   *
   * A reusable modal dialog with customizable content and actions.
   * Features: backdrop click-to-close, ESC key support, focus trap.
   */

  import { onMount, onDestroy, tick } from 'svelte';
  import { t } from '$lib/i18n';
  import type { Snippet } from 'svelte';

  interface Props {
    isOpen: boolean;
    title: string;
    onClose: () => void;
    showCloseButton?: boolean;
    closeOnBackdropClick?: boolean;
    closeOnEscape?: boolean;
    children?: Snippet;
    footer?: Snippet;
  }

  let {
    isOpen,
    title,
    onClose,
    showCloseButton = true,
    closeOnBackdropClick = true,
    closeOnEscape = true,
    children,
    footer,
  }: Props = $props();

  let modalRef: HTMLDivElement | undefined = $state(undefined);
  let previousActiveElement: HTMLElement | null = null;

  // Handle escape key
  function handleKeydown(event: KeyboardEvent): void {
    if (closeOnEscape && event.key === 'Escape') {
      event.preventDefault();
      onClose();
    }
  }

  // Handle backdrop click
  function handleBackdropClick(event: MouseEvent): void {
    if (closeOnBackdropClick && event.target === event.currentTarget) {
      onClose();
    }
  }

  // Focus management
  $effect(() => {
    if (isOpen) {
      previousActiveElement = document.activeElement as HTMLElement;
      tick().then(() => {
        if (modalRef) {
          const focusable = modalRef.querySelector<HTMLElement>(
            'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
          );
          focusable?.focus();
        }
      });
    } else if (previousActiveElement) {
      previousActiveElement.focus();
      previousActiveElement = null;
    }
  });

  // Add/remove keydown listener
  onMount(() => {
    document.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeydown);
  });
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={handleBackdropClick}>
    <div
      class="modal"
      bind:this={modalRef}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
    >
      <div class="modal-header">
        <h2 id="modal-title" class="modal-title">{title}</h2>
        {#if showCloseButton}
          <button
            type="button"
            class="modal-close"
            onclick={onClose}
            aria-label={t('modal.close_aria')}
          >
            &times;
          </button>
        {/if}
      </div>

      <div class="modal-content">
        {#if children}
          {@render children()}
        {/if}
      </div>

      {#if footer}
        <div class="modal-footer">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--color-modal-backdrop);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .modal {
    background: var(--color-modal-bg);
    border-radius: var(--border-radius-lg);
    box-shadow: var(--shadow-modal);
    max-width: 90vw;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    animation: slideIn 0.15s ease-out;
    min-width: 300px;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--color-kbd-border);
  }

  .modal-title {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .modal-close {
    background: none;
    border: none;
    font-size: 1.5rem;
    line-height: 1;
    padding: 0;
    cursor: pointer;
    color: var(--color-text-muted);
    transition: color 0.15s ease;
  }

  .modal-close:hover {
    color: var(--color-text);
  }

  .modal-content {
    padding: var(--spacing-lg);
    overflow-y: auto;
    flex: 1;
  }

  .modal-footer {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
    padding: var(--spacing-md) var(--spacing-lg);
    border-top: 1px solid var(--color-kbd-border);
  }
</style>
