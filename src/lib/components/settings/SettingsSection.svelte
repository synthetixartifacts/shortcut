<script lang="ts">
  /**
   * SettingsSection - Section wrapper using Card pattern
   *
   * Provides consistent styling for settings sections.
   */
  import type { Snippet } from 'svelte';
  import { Card } from '$lib/components/ui/patterns';

  interface Props {
    title: string;
    children: Snippet;
  }

  let { title, children }: Props = $props();
</script>

<div class="settings-section">
  <Card padding="lg">
    <h2 class="section-title">{title}</h2>
    {@render children()}
  </Card>
</div>

<style>
  .settings-section {
    margin-bottom: var(--spacing-lg);
  }

  .section-title {
    margin: 0 0 var(--spacing-lg);
    font-size: 1rem;
    font-weight: 600;
  }
</style>
