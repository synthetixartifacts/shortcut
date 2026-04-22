<script lang="ts">
  /**
   * Provider Settings — API keys and task assignments.
   *
   * State + actions live in `src/lib/state/providers-settings.svelte.ts`.
   * Form sections are `ProviderCredentialsForm` + `TaskAssignmentsForm`.
   */
  import { onMount } from 'svelte';
  import { page } from '$app/state';

  import { settingsState, loadSettings } from '$lib/state/settings.svelte';
  import { loadProvidersSettings } from '$lib/state/providers-settings.svelte';

  import {
    SetupBanner,
    ProviderCredentialsForm,
    TaskAssignmentsForm,
  } from '$lib/components/settings';
  import { PageHeader } from '$lib/components/ui';
  import { ErrorBanner } from '$lib/components/ui/patterns';
  import { t } from '$lib/i18n';

  const isSetupMode = $derived(page.url.searchParams.has('setup'));

  onMount(async () => {
    await loadSettings();
    await loadProvidersSettings();
  });
</script>

<div class="page-providers">
  <PageHeader
    title={t('settings.title')}
    subtitle={t('settings.subtitle')}
    backHref="/settings"
    backLabel={t('settings_hub.back_label')}
  />

  <SetupBanner show={isSetupMode} />

  {#if settingsState.isLoading}
    <p class="loading">{t('settings.loading')}</p>
  {:else if settingsState.error}
    <ErrorBanner message={settingsState.error} />
  {:else}
    <ProviderCredentialsForm />
    <TaskAssignmentsForm />

    <div class="settings-info">
      <p>
        {t('settings.info_prefix')}
        <a href="/actions/dictation">{t('settings.info_link')}</a>
      </p>
    </div>
  {/if}
</div>

<style>
  .page-providers {
    max-width: var(--form-max-width);
  }

  .loading {
    color: var(--color-text-muted);
  }

  .settings-info {
    margin-top: var(--spacing-lg);
    padding: var(--spacing-md);
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-md);
    font-size: 0.85rem;
  }

  .settings-info p {
    margin: 0;
  }

  .settings-info a {
    color: var(--color-primary);
    text-decoration: none;
  }

  .settings-info a:hover {
    text-decoration: underline;
  }
</style>
