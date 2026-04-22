<script lang="ts">
  /**
   * HistoryList - Container for history entries
   *
   * Renders a list of HistoryItem components or empty state.
   */
  import type { HistoryEntry } from '$lib/types';
  import HistoryItem from './HistoryItem.svelte';
  import EmptyHistory from './EmptyHistory.svelte';

  interface Props {
    entries: HistoryEntry[];
    onCopy: (text: string) => void;
    onDelete?: (id: string) => void;
  }

  let { entries, onCopy, onDelete }: Props = $props();
</script>

{#if entries.length === 0}
  <EmptyHistory />
{:else}
  <div class="history-list">
    {#each entries as entry (entry.id)}
      <HistoryItem {entry} {onCopy} {onDelete} />
    {/each}
  </div>
{/if}

<style>
  .history-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }
</style>
