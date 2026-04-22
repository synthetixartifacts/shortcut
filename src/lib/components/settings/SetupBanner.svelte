<script lang="ts">
  /**
   * SetupBanner - Welcome banner for first-time setup
   */
  import { t } from '$lib/i18n';

  interface Props {
    show: boolean;
  }

  let { show }: Props = $props();
</script>

{#if show}
  <div class="setup-banner">
    <h2>{t('settings.setup_welcome')}</h2>
    <p>{t('settings.setup_intro')}</p>
    <ul>
      <li>{t('settings.setup_item_key')}</li>
    </ul>
  </div>
{/if}

<style>
  .setup-banner {
    padding: var(--spacing-lg);
    background: var(--color-primary-light);
    border: 1px solid var(--color-primary);
    border-radius: var(--border-radius-lg);
    margin-bottom: var(--spacing-xl);
  }
  .setup-banner h2 {
    margin: 0 0 var(--spacing-sm);
    font-size: 1.1rem;
    color: var(--color-primary);
  }
  .setup-banner p {
    margin: 0 0 var(--spacing-sm);
    color: var(--color-text);
  }
  .setup-banner ul {
    margin: 0;
    padding-left: var(--spacing-lg);
    color: var(--color-text-muted);
  }
  .setup-banner li {
    margin-bottom: var(--spacing-xs);
  }
</style>
