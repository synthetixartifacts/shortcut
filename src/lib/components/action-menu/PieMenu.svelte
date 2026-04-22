<script lang="ts">
	/**
	 * PieMenu — SVG container rendering all wedges in a radial layout.
	 *
	 * Renders a ring of PieWedge components with a center circle anchor.
	 */

	import type { MenuItem } from '$lib/features/action-menu/types';
	import { MENU_SIZE, OUTER_RADIUS, INNER_RADIUS } from '$lib/features/action-menu/constants';
	import type { ShortcutAction } from '$lib/types';
	import PieWedge from './PieWedge.svelte';

	interface Props {
		items: MenuItem[];
		onSelect: (action: ShortcutAction) => void;
	}

	let { items, onSelect }: Props = $props();

	let hoveredItem = $state<string | null>(null);

	const center = MENU_SIZE / 2;
</script>

<svg
	viewBox="0 0 {MENU_SIZE} {MENU_SIZE}"
	width={MENU_SIZE}
	height={MENU_SIZE}
	xmlns="http://www.w3.org/2000/svg"
>
	{#each items as item, i}
		<PieWedge
			{item}
			index={i}
			total={items.length}
			outerRadius={OUTER_RADIUS}
			innerRadius={INNER_RADIUS}
			centerX={center}
			centerY={center}
			isHovered={hoveredItem === item.id}
			onclick={() => onSelect(item.action)}
			onmouseenter={() => (hoveredItem = item.id)}
			onmouseleave={() => (hoveredItem = null)}
		/>
	{/each}

	<!-- Center circle (visual anchor) -->
	<circle
		cx={center}
		cy={center}
		r={INNER_RADIUS - 5}
		fill="rgba(0,0,0,0.6)"
		stroke="rgba(255,255,255,0.2)"
		stroke-width="1"
	/>
</svg>
