<script lang="ts">
  /**
   * Text Transform History Page
   *
   * View and manage successful results from Grammar Fix / Translate / Improve.
   * Mirrors the dictation `/history` page layout but with an action filter and
   * the transform-specific components. Clear-all uses a Modal (per UX spec)
   * rather than the older `confirm()` pattern.
   */
  import { onMount } from 'svelte';

  import {
    textTransformHistoryState,
    loadTextTransformHistory,
    searchTextTransformHistory,
    setTextTransformActionFilter,
    deleteTextTransformEntry,
    clearAllTextTransformHistory,
  } from '$lib/state/text-transform-history.svelte';

  import { PageHeader, Button } from '$lib/components/ui';
  import Modal from '$lib/components/ui/Modal.svelte';
  import { Input } from '$lib/components/ui/primitives';
  import { Pagination } from '$lib/components/history';
  import {
    TextTransformHistoryList,
    EmptyTextTransformHistory,
    ActionFilter,
  } from '$lib/components/text-transform-history';
  import { t } from '$lib/i18n';

  // Local UI state
  let isClearing = $state(false);
  let confirmOpen = $state(false);
  let searchInput = $state('');
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  onMount(async () => {
    await loadTextTransformHistory(1);
  });

  function handleSearchInput(value: string): void {
    searchInput = value;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      void searchTextTransformHistory(value);
    }, 300);
  }

  async function handleCopy(text: string): Promise<void> {
    await navigator.clipboard.writeText(text);
  }

  async function handleDelete(id: string): Promise<void> {
    await deleteTextTransformEntry(id);
  }

  function openClearConfirm(): void {
    confirmOpen = true;
  }

  function closeClearConfirm(): void {
    if (!isClearing) {
      confirmOpen = false;
    }
  }

  async function confirmClearAll(): Promise<void> {
    isClearing = true;
    try {
      await clearAllTextTransformHistory();
      confirmOpen = false;
    } finally {
      isClearing = false;
    }
  }

  async function handlePageChange(page: number): Promise<void> {
    await loadTextTransformHistory(page);
  }

  // Subtitle: count + singular/plural label, or "N results" for an active search.
  let subtitle = $derived.by(() => {
    const total = textTransformHistoryState.total;
    if (textTransformHistoryState.searchQuery) {
      return `${total} ${t('text_transform_history.search_results')}`;
    }
    const label =
      total === 1
        ? t('text_transform_history.subtitle_singular')
        : t('text_transform_history.subtitle_plural');
    return `${total} ${label}`;
  });

  // True when the empty list is the result of an active search/filter,
  // false when it is a genuinely empty history.
  let isFilteredEmpty = $derived(
    textTransformHistoryState.entries.length === 0 &&
      (textTransformHistoryState.searchQuery !== '' ||
        textTransformHistoryState.actionFilter !== 'all')
  );
</script>

<div class="page-text-transform-history">
  <PageHeader title={t('text_transform_history.title')} {subtitle}>
    {#snippet actions()}
      <Button
        variant="danger"
        onclick={openClearConfirm}
        disabled={textTransformHistoryState.total === 0 || isClearing}
      >
        {isClearing
          ? t('text_transform_history.button_clearing')
          : t('text_transform_history.button_clear_all')}
      </Button>
    {/snippet}
  </PageHeader>

  <div class="search-bar">
    <Input
      value={searchInput}
      placeholder={t('text_transform_history.search_placeholder')}
      onchange={handleSearchInput}
    />
  </div>

  <ActionFilter
    value={textTransformHistoryState.actionFilter}
    onChange={(filter) => void setTextTransformActionFilter(filter)}
  />

  {#if textTransformHistoryState.isLoading}
    <p class="loading">{t('text_transform_history.loading')}</p>
  {:else if textTransformHistoryState.error}
    <p class="error">{textTransformHistoryState.error}</p>
  {:else if textTransformHistoryState.entries.length === 0}
    <EmptyTextTransformHistory filtered={isFilteredEmpty} />
  {:else}
    <TextTransformHistoryList
      entries={textTransformHistoryState.entries}
      onCopy={handleCopy}
      onDelete={handleDelete}
    />

    {#if textTransformHistoryState.totalPages > 1}
      <Pagination
        currentPage={textTransformHistoryState.currentPage}
        totalPages={textTransformHistoryState.totalPages}
        onPageChange={handlePageChange}
      />
    {/if}
  {/if}
</div>

<Modal
  isOpen={confirmOpen}
  title={t('text_transform_history.clear_confirm_title')}
  onClose={closeClearConfirm}
  closeOnBackdropClick={!isClearing}
  closeOnEscape={!isClearing}
>
  <p>{t('text_transform_history.clear_confirm_body')}</p>
  {#snippet footer()}
    <Button variant="secondary" onclick={closeClearConfirm} disabled={isClearing}>
      {t('common.cancel')}
    </Button>
    <Button variant="danger" onclick={confirmClearAll} disabled={isClearing}>
      {isClearing
        ? t('text_transform_history.button_clearing')
        : t('text_transform_history.clear_confirm_button')}
    </Button>
  {/snippet}
</Modal>

<style>
  .page-text-transform-history {
    max-width: 100%;
  }

  .search-bar {
    margin-bottom: var(--spacing-md);
  }

  .loading,
  .error {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--color-text-muted);
  }

  .error {
    color: var(--color-danger);
  }
</style>
