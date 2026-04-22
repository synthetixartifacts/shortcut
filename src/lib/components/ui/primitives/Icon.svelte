<script lang="ts">
  /**
   * Icon Primitive
   *
   * SVG icons with consistent sizing and styling.
   */

  type IconName =
    | 'edit'
    | 'close'
    | 'check'
    | 'warning'
    | 'refresh'
    | 'send'
    | 'dashboard'
    | 'keyboard'
    | 'settings'
    | 'mic'
    | 'history'
    | 'terminal'
    | 'copy'
    | 'trash'
    | 'eye'
    | 'eye-off'
    | 'key'
    | 'wand'
    | 'chevron-right';

  interface Props {
    name: IconName;
    size?: number;
    class?: string;
  }

  let { name, size = 16, class: className = '' }: Props = $props();

  // SVG path data for each icon (24x24 viewBox)
  const icons: Record<IconName, string> = {
    edit: 'M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z M15 5l4 4',
    close: 'M18 6L6 18 M6 6l12 12',
    check: 'M20 6L9 17l-5-5',
    warning: 'M12 9v4 M12 17h.01 M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z',
    refresh: 'M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8 M21 3v5h-5 M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16 M3 21v-5h5',
    send: 'M22 2L11 13 M22 2l-7 20-4-9-9-4z',
    // Navigation icons
    dashboard: 'M3 3h7v7H3V3z M14 3h7v7h-7V3z M14 14h7v7h-7v-7z M3 14h7v7H3v-7z',
    keyboard: 'M2 6a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z M6 10h.01 M10 10h.01 M14 10h.01 M18 10h.01 M8 14h8',
    settings: 'M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z',
    mic: 'M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3z M19 10v2a7 7 0 0 1-14 0v-2 M12 19v3 M8 22h8',
    history: 'M12 8v4l3 3 M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8 M3 3v5h5',
    terminal: 'M4 17l6-6-6-6 M12 19h8',
    copy: 'M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2 M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1z',
    trash: 'M3 6h18 M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6 M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2',
    eye: 'M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z',
    'eye-off': 'M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94 M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19 M14.12 14.12a3 3 0 1 1-4.24-4.24 M1 1l22 22',
    key: 'M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4',
    wand: 'M15 4V2 M15 16v-2 M8 9h2 M20 9h2 M17.8 11.8l1.4 1.4 M17.8 6.2l1.4-1.4 M12.2 6.2l-1.4-1.4 M12.2 11.8l-1.4 1.4 M12 15l-8.5 8.5a2.12 2.12 0 0 1-3-3L9 12',
    'chevron-right': 'M9 18l6-6-6-6'
  };

  // Split paths by space-separated commands starting with M
  const paths = $derived(icons[name].split(/(?=M)/));
</script>

<svg
  class="icon {className}"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#each paths as path}
    <path d={path.trim()} />
  {/each}
</svg>

<style>
  .icon {
    display: inline-block;
    vertical-align: middle;
    flex-shrink: 0;
  }
</style>
