<script lang="ts">
  /**
   * TextTransformHistoryList - Container for text-transform history entries.
   *
   * Renders a list of TextTransformHistoryItem components. Empty state is
   * the page's responsibility: the page chooses when to render this list
   * vs. EmptyTextTransformHistory (with its `filtered` prop) to differentiate
   * "no entries yet" from "search/filter has zero matches".
   */
  import type { TextTransformHistoryEntry } from '$lib/types';
  import TextTransformHistoryItem from './TextTransformHistoryItem.svelte';

  interface Props {
    entries: TextTransformHistoryEntry[];
    onCopy: (text: string) => void;
    onDelete?: (id: string) => void;
  }

  let { entries, onCopy, onDelete }: Props = $props();
</script>

<div class="history-list">
  {#each entries as entry (entry.id)}
    <TextTransformHistoryItem {entry} {onCopy} {onDelete} />
  {/each}
</div>

<style>
  .history-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }
</style>
