<script lang="ts">
  /**
   * History Page - View and manage dictation history
   */
  import { onMount } from 'svelte';

  // State
  import {
    historyState,
    loadHistory,
    deleteEntry,
    clearAllHistory,
    searchHistory,
  } from '$lib/state/history.svelte';

  // Components
  import { PageHeader, Button } from '$lib/components/ui';
  import { Input } from '$lib/components/ui/primitives';
  import { HistoryList, Pagination } from '$lib/components/history';
  import { t } from '$lib/i18n';

  // Loading and feedback state
  let isClearing = $state(false);
  let searchInput = $state('');
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  onMount(async () => {
    await loadHistory(1);
  });

  function handleSearchInput(value: string): void {
    searchInput = value;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      searchHistory(value);
    }, 300);
  }

  async function handleCopy(text: string): Promise<void> {
    await navigator.clipboard.writeText(text);
  }

  async function handleDelete(id: string): Promise<void> {
    await deleteEntry(id);
  }

  async function handleClearAll(): Promise<void> {
    if (!confirm(t('history.clear_confirm'))) return;
    isClearing = true;
    try {
      await clearAllHistory();
    } finally {
      isClearing = false;
    }
  }

  async function handlePageChange(page: number): Promise<void> {
    await loadHistory(page);
  }
</script>

<div class="page-history">
  <PageHeader
    title={t('history.title')}
    subtitle={historyState.searchQuery
      ? `${historyState.total} ${t('history.search_results')}`
      : `${historyState.total} ${historyState.total === 1 ? t('history.subtitle_singular') : t('history.subtitle_plural')}`}
  >
    {#snippet actions()}
      <Button
        variant="danger"
        onclick={handleClearAll}
        disabled={historyState.total === 0 || isClearing}
      >
        {isClearing ? t('history.button_clearing') : t('history.button_clear_all')}
      </Button>
    {/snippet}
  </PageHeader>

  <div class="search-bar">
    <Input
      value={searchInput}
      placeholder={t('history.search_placeholder')}
      onchange={handleSearchInput}
    />
  </div>

  {#if historyState.isLoading}
    <p class="loading">{t('history.loading')}</p>
  {:else if historyState.error}
    <p class="error">{historyState.error}</p>
  {:else if historyState.searchQuery && historyState.entries.length === 0}
    <p class="no-results">{t('history.search_no_results')}</p>
  {:else}
    <HistoryList
      entries={historyState.entries}
      onCopy={handleCopy}
      onDelete={handleDelete}
    />

    {#if historyState.totalPages > 1}
      <Pagination
        currentPage={historyState.currentPage}
        totalPages={historyState.totalPages}
        onPageChange={handlePageChange}
      />
    {/if}
  {/if}
</div>

<style>
  .page-history {
    max-width: 100%;
  }

  .search-bar {
    margin-bottom: var(--spacing-md);
  }

  .loading,
  .error,
  .no-results {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--color-text-muted);
  }

  .error {
    color: var(--color-danger);
  }
</style>
