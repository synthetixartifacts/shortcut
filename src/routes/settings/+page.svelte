<script lang="ts">
  /**
   * Settings Hub — single landing page for all settings categories.
   *
   * Replaces the former sidebar "Settings" collapsible section. Groups every
   * settings area into Actions / System / Preferences, each rendered with the
   * reusable SettingsLinkCard.
   */
  import { onMount } from 'svelte';
  import { getCurrentPlatform } from '$lib/features/shortcuts';
  import { PageHeader, SettingsLinkCard } from '$lib/components/ui/patterns';
  import ActionsShortcutGrid from '$lib/components/actions/ActionsShortcutGrid.svelte';
  import { appSettingsState } from '$lib/state/app-settings.svelte';
  import { t } from '$lib/i18n';

  let platform = $state<'macos' | 'windows' | 'linux'>('windows');

  onMount(async () => {
    platform = await getCurrentPlatform();
  });
</script>

<div class="page-settings-hub">
  <PageHeader
    title={t('settings_hub.title')}
    subtitle={t('settings_hub.subtitle')}
  />

  <section class="group">
    <h2 class="group-title">{t('settings_hub.section_actions')}</h2>
    <ActionsShortcutGrid {platform} />
  </section>

  <section class="group">
    <h2 class="group-title">{t('settings_hub.section_system')}</h2>
    <div class="group-items">
      <SettingsLinkCard
        href="/settings/providers"
        icon="🔑"
        title={t('settings_hub.providers_title')}
        description={t('settings_hub.providers_desc')}
      />
      <SettingsLinkCard
        href="/shortcuts"
        icon="⌨️"
        title={t('shortcuts.title')}
        description={t('settings_hub.shortcuts_desc')}
      />
    </div>
  </section>

  <section class="group">
    <h2 class="group-title">{t('settings_hub.section_preferences')}</h2>
    <div class="group-items">
      <SettingsLinkCard
        href="/app-settings"
        icon="⚙️"
        title={t('app_settings.title')}
        description={t('app_settings.subtitle')}
      />
      {#if appSettingsState.debugEnabled}
        <SettingsLinkCard
          href="/debug"
          icon="🛠️"
          title={t('debug.title')}
          description={t('settings_hub.debug_desc')}
        />
      {/if}
    </div>
  </section>
</div>

<style>
  .page-settings-hub {
    max-width: var(--page-max-width);
  }

  .group {
    margin-bottom: var(--spacing-xl);
  }

  .group-title {
    margin: 0 0 var(--spacing-sm);
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-text-muted);
  }

  .group-items {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }
</style>
