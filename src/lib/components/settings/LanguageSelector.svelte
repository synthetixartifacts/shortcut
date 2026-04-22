<script lang="ts">
  /**
   * Language Selector - Multi-select for language hints
   */
  import { t } from '$lib/i18n';

  interface Props {
    selected: string[];
    onchange: (languages: string[]) => void;
  }

  let { selected, onchange }: Props = $props();

  const availableLanguages = $derived([
    { code: 'en', name: t('languages.en') },
    { code: 'fr', name: t('languages.fr') },
    { code: 'es', name: t('languages.es') },
    { code: 'de', name: t('languages.de') },
    { code: 'it', name: t('languages.it') },
    { code: 'pt', name: t('languages.pt') },
    { code: 'zh', name: t('languages.zh') },
    { code: 'ja', name: t('languages.ja') },
  ]);

  function toggleLanguage(code: string): void {
    const newSelected = selected.includes(code)
      ? selected.filter(l => l !== code)
      : [...selected, code];
    onchange(newSelected);
  }
</script>

<fieldset class="language-selector">
  <legend class="input-label">{t('language_selector.legend')}</legend>
  <p class="hint-text">{t('language_selector.hint')}</p>
  <div class="language-chips" role="group" aria-label={t('language_selector.legend')}>
    {#each availableLanguages as lang (lang.code)}
      <button
        type="button"
        class="language-chip"
        class:selected={selected.includes(lang.code)}
        onclick={() => toggleLanguage(lang.code)}
      >
        {lang.name}
      </button>
    {/each}
  </div>
</fieldset>

<style>
  .language-selector {
    margin-bottom: var(--spacing-md);
    border: none;
    padding: 0;
  }
  .input-label {
    display: block;
    margin-bottom: var(--spacing-xs);
    font-size: 0.85rem;
    font-weight: 500;
    padding: 0;
  }
  .hint-text {
    margin: 0 0 var(--spacing-sm);
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }
  .language-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
  }
  .language-chip {
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-sm);
    background: var(--color-kbd-bg);
    color: var(--color-text);
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.15s ease;
  }
  .language-chip:hover {
    border-color: var(--color-primary);
  }
  .language-chip.selected {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: var(--color-text-on-primary);
  }
</style>
