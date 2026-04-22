<script lang="ts">
  /**
   * TermsList - Manage custom vocabulary terms
   */
  import { Input, Button } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';

  interface Props {
    terms: string[];
    onRemove: (term: string) => void;
    onAdd: (term: string) => void;
    disabled?: boolean;
  }

  let { terms, onRemove, onAdd, disabled = false }: Props = $props();
  let newTerm = $state('');

  function handleAdd(): void {
    const trimmed = newTerm.trim();
    if (trimmed && !terms.includes(trimmed)) {
      onAdd(trimmed);
      newTerm = '';
    }
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleAdd();
    }
  }
</script>

<div class="terms-list">
  <div class="add-term">
    <Input
      type="text"
      value={newTerm}
      placeholder={t('terms.placeholder')}
      {disabled}
      onchange={(v) => newTerm = v}
      onkeydown={handleKeydown}
    />
    <Button variant="secondary" onclick={handleAdd} {disabled}>{t('terms.button_add')}</Button>
  </div>

  {#if terms.length > 0}
    <ul class="terms">
      {#each terms as term}
        <li>
          <span class="term-text">{term}</span>
          <button
            class="remove-btn"
            onclick={() => onRemove(term)}
            {disabled}
            title={t('terms.button_remove')}
          >
            &times;
          </button>
        </li>
      {/each}
    </ul>
    <p class="term-count">{terms.length} {terms.length !== 1 ? t('terms.count_plural') : t('terms.count_singular')}</p>
  {:else}
    <p class="empty">{t('terms.empty')}</p>
  {/if}
</div>

<style>
  .terms-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .add-term {
    display: flex;
    gap: var(--spacing-xs);
  }

  .terms {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
  }

  .terms li {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    font-size: 0.85rem;
  }

  .term-text {
    font-family: monospace;
  }

  .remove-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    color: var(--color-text-muted);
    padding: 0;
  }

  .remove-btn:hover:not(:disabled) {
    color: var(--color-danger);
  }

  .remove-btn:disabled {
    cursor: not-allowed;
  }

  .term-count {
    margin: 0;
    font-size: 0.75rem;
    color: var(--color-text-hint);
  }

  .empty {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text-muted);
    font-style: italic;
  }
</style>
