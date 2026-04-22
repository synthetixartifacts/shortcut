<script lang="ts">
  /**
   * Sidebar - Main navigation sidebar component
   */
  import { page } from '$app/state';
  import NavItem from './NavItem.svelte';
  import { t } from '$lib/i18n';

  type NavIcon = 'dashboard' | 'history' | 'settings';

  interface NavItemConfig {
    href: string;
    label: string;
    icon: NavIcon;
  }

  const mainNavItems: NavItemConfig[] = $derived([
    { href: '/', label: t('nav.dashboard'), icon: 'dashboard' },
    { href: '/history', label: t('nav.dictation_history'), icon: 'history' },
    { href: '/settings', label: t('nav.settings'), icon: 'settings' },
  ]);

  function isActive(href: string, pathname: string): boolean {
    if (href === '/') return pathname === '/';
    return pathname === href || pathname.startsWith(href + '/');
  }
</script>

<aside class="sidebar">
  <a href="/" class="sidebar-header">
    <img src="/icon.png" alt="ShortCut" class="app-logo" />
    <div class="app-title">
      <span>Short</span>
      <span>Cut</span>
    </div>
  </a>

  <nav class="sidebar-nav">
    {#each mainNavItems as item}
      <NavItem
        href={item.href}
        label={item.label}
        icon={item.icon}
        active={isActive(item.href, page.url.pathname)}
      />
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    height: 100vh;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--sidebar-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-lg);
    border-bottom: 1px solid var(--sidebar-border);
    text-decoration: none;
    color: inherit;
  }

  .sidebar-header:hover {
    opacity: 0.8;
  }

  .app-logo {
    height: 48px;
    width: auto;
  }

  .app-title {
    display: flex;
    flex-direction: column;
    font-size: 1.1rem;
    font-weight: 700;
    line-height: 1.15;
    color: var(--color-text);
    letter-spacing: 0.01em;
  }

  .sidebar-nav {
    flex: 1;
    padding: var(--spacing-md) 0;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }
</style>
