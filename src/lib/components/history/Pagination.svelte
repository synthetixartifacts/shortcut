<script lang="ts">
  /**
   * Pagination - Page navigation controls
   */
  import { Button } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';

  interface Props {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
  }

  let { currentPage, totalPages, onPageChange }: Props = $props();
</script>

<nav class="pagination" aria-label={t('history.pagination_aria')}>
  <Button
    variant="secondary"
    onclick={() => onPageChange(currentPage - 1)}
    disabled={currentPage <= 1}
  >
    {t('history.pagination_previous')}
  </Button>

  <span class="page-info">
    {t('history.pagination_page')} {currentPage} {t('history.pagination_of')} {totalPages}
  </span>

  <Button
    variant="secondary"
    onclick={() => onPageChange(currentPage + 1)}
    disabled={currentPage >= totalPages}
  >
    {t('history.pagination_next')}
  </Button>
</nav>

<style>
  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-md);
    padding: var(--spacing-lg) 0;
  }

  .page-info {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    min-width: 100px;
    text-align: center;
  }
</style>
