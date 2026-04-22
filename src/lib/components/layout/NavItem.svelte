<script lang="ts">
  /**
   * NavItem - Navigation link component for sidebar
   */
  import { Icon } from '$lib/components/ui/primitives';

  interface Props {
    href: string;
    label: string;
    icon:
      | 'dashboard'
      | 'keyboard'
      | 'settings'
      | 'mic'
      | 'history'
      | 'terminal'
      | 'key'
      | 'wand'
      | 'edit'
      | 'send'
      | 'eye';
    active?: boolean;
    indent?: boolean;
  }

  let { href, label, icon, active = false, indent = false }: Props = $props();
</script>

<a class="nav-item" class:active class:indent {href}>
  <Icon name={icon} size={18} />
  <span class="nav-label">{label}</span>
</a>

<style>
  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    height: var(--nav-item-height);
    border-radius: var(--nav-item-radius);
    text-decoration: none;
    color: var(--color-text);
    transition: background 0.15s ease;
    margin: 0 var(--spacing-sm);
  }

  .nav-item.indent {
    padding-left: calc(var(--spacing-md) + var(--spacing-lg));
  }

  .nav-item:hover {
    background: var(--nav-item-hover-bg);
  }

  .nav-item.active {
    background: var(--nav-item-active-bg);
    color: var(--color-primary);
  }

  .nav-label {
    font-size: 14px;
    font-weight: 500;
  }
</style>
