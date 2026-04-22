<script lang="ts">
  /**
   * EmptyHistory - Empty state for history list
   */
  import { t } from '$lib/i18n';
</script>

<div class="empty-state">
  <p class="empty-title">{t('history.empty_title')}</p>
  <p class="empty-description">{t('history.empty_description')}</p>
</div>

<style>
  .empty-state {
    text-align: center;
    padding: var(--spacing-xl) var(--spacing-lg);
    color: var(--color-text-muted);
  }

  .empty-title {
    margin: 0 0 var(--spacing-xs) 0;
    font-size: 1rem;
    color: var(--color-text);
  }

  .empty-description {
    margin: 0;
    font-size: 0.875rem;
  }
</style>
