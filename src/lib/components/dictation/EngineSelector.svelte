<script lang="ts">
  /**
   * EngineSelector - Container for engine selection cards
   *
   * Renders an EngineCard for each available transcription engine.
   */
  import type { EngineInfo } from '$lib/types';
  import EngineCard from './EngineCard.svelte';

  interface Props {
    engines: EngineInfo[];
    pendingEngine?: string | null;
    onActivate: (id: string) => void;
    onDownloadComplete?: () => void;
    onDownloadCancel?: () => void;
    isSwitching?: boolean;
  }

  let { engines, pendingEngine = null, onActivate, onDownloadComplete, onDownloadCancel, isSwitching = false }: Props = $props();
</script>

<div class="engine-selector" class:switching={isSwitching}>
  {#each engines as engine (engine.id)}
    <EngineCard
      {engine}
      {onActivate}
      showDownload={pendingEngine === engine.id}
      {onDownloadComplete}
      {onDownloadCancel}
    />
  {/each}
</div>

<style>
  .engine-selector {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .engine-selector.switching {
    opacity: 0.6;
    pointer-events: none;
  }
</style>
