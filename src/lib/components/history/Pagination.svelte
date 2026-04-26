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

<nav class="pagination" aria-label={t('pagination.aria')}>
  <Button
    variant="secondary"
    onclick={() => onPageChange(currentPage - 1)}
    disabled={currentPage <= 1}
  >
    {t('pagination.previous')}
  </Button>

  <span class="page-info">
    {t('pagination.page')} {currentPage} {t('pagination.of')} {totalPages}
  </span>

  <Button
    variant="secondary"
    onclick={() => onPageChange(currentPage + 1)}
    disabled={currentPage >= totalPages}
  >
    {t('pagination.next')}
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
