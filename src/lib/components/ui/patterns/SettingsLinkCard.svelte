<script lang="ts">
  interface Props {
    href: string;
    icon: string;
    title: string;
    description: string;
    hint?: string;
    active?: boolean;
  }

  let { href, icon, title, description, hint, active = false }: Props = $props();
</script>

<a {href} class="link-card" class:active class:has-hint={!!hint}>
  {#if active}
    <span class="active-indicator"></span>
  {/if}
  <div class="info">
    {#if !hint}
      <span class="icon">{icon}</span>
    {/if}
    <div>
      <h3>{title}</h3>
      <p>{description}</p>
    </div>
  </div>
  {#if hint}
    <div class="hint-stack">
      <span class="icon stack-icon">{icon}</span>
      <span class="shortcut-hint">{hint}</span>
    </div>
  {:else}
    <span class="arrow">&rarr;</span>
  {/if}
</a>

<style>
  .link-card {
    position: relative;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm) var(--spacing-lg);
    background: var(--color-card-bg);
    border-radius: var(--border-radius-lg);
    text-decoration: none;
    color: inherit;
    transition: all 0.15s ease;
  }

  .link-card:hover {
    background: var(--color-primary-light);
  }

  .link-card.active {
    background: var(--color-primary-light);
  }

  .active-indicator {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-danger);
    animation: pulse-indicator 1.5s infinite;
  }

  @keyframes pulse-indicator {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.6; transform: scale(1.2); }
  }

  .info {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    min-width: 0;
    flex: 1;
  }

  .icon {
    font-size: calc(1.5rem + 5px);
    line-height: 1;
  }

  .info h3 {
    margin: 0;
    font-size: 1rem;
  }

  .info p {
    margin: var(--spacing-xs) 0 0;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .arrow {
    color: var(--color-primary);
  }

  .hint-stack {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-xs);
    flex-shrink: 0;
    margin-left: var(--spacing-md);
  }

  .stack-icon {
    font-size: calc(1.25rem + 5px);
  }

  .shortcut-hint {
    font-size: 0.8rem;
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--color-kbd-bg);
    border-radius: var(--border-radius-sm);
    font-family: monospace;
  }
</style>
