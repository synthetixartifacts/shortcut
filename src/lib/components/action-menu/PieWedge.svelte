<script lang="ts">
	/**
	 * PieWedge — SVG arc wedge for the radial pie menu.
	 *
	 * Renders a single wedge with icon and label, positioned
	 * as a sector of a ring (outer arc + inner arc).
	 */

	import type { MenuItem } from '$lib/features/action-menu/types';
	import { t } from '$lib/i18n';

	interface Props {
		item: MenuItem;
		index: number;
		total: number;
		outerRadius: number;
		innerRadius: number;
		centerX: number;
		centerY: number;
		isHovered: boolean;
		onclick: () => void;
		onmouseenter: () => void;
		onmouseleave: () => void;
	}

	let {
		item,
		index,
		total,
		outerRadius,
		innerRadius,
		centerX,
		centerY,
		isHovered,
		onclick,
		onmouseenter,
		onmouseleave
	}: Props = $props();

	const GAP_DEGREES = 2;

	/** Convert degrees to radians */
	function toRad(deg: number): number {
		return (deg * Math.PI) / 180;
	}

	/** Get point on circle at given angle and radius */
	function pointOnCircle(cx: number, cy: number, r: number, angleDeg: number) {
		const rad = toRad(angleDeg);
		return { x: cx + r * Math.cos(rad), y: cy + r * Math.sin(rad) };
	}

	/** Build SVG path for a ring sector (wedge) */
	function describeArc(
		cx: number,
		cy: number,
		outerR: number,
		innerR: number,
		startDeg: number,
		endDeg: number
	): string {
		const outerStart = pointOnCircle(cx, cy, outerR, startDeg);
		const outerEnd = pointOnCircle(cx, cy, outerR, endDeg);
		const innerStart = pointOnCircle(cx, cy, innerR, startDeg);
		const innerEnd = pointOnCircle(cx, cy, innerR, endDeg);
		const largeArc = endDeg - startDeg > 180 ? 1 : 0;

		return [
			`M ${outerStart.x} ${outerStart.y}`,
			`A ${outerR} ${outerR} 0 ${largeArc} 1 ${outerEnd.x} ${outerEnd.y}`,
			`L ${innerEnd.x} ${innerEnd.y}`,
			`A ${innerR} ${innerR} 0 ${largeArc} 0 ${innerStart.x} ${innerStart.y}`,
			'Z'
		].join(' ');
	}

	/** Derived geometry for this wedge */
	let wedgeAngle = $derived(360 / total);
	let startAngle = $derived(index * wedgeAngle - 90 + GAP_DEGREES / 2);
	let endAngle = $derived(startAngle + wedgeAngle - GAP_DEGREES);
	let midAngle = $derived((startAngle + endAngle) / 2);
	let path = $derived(
		describeArc(centerX, centerY, outerRadius, innerRadius, startAngle, endAngle)
	);

	/** Label/icon placement at midpoint of the wedge arc */
	let labelRadius = $derived((innerRadius + outerRadius) / 2);
	let labelPos = $derived(pointOnCircle(centerX, centerY, labelRadius, midAngle));

	/** Fill opacity changes on hover */
	let fillOpacity = $derived(isHovered ? 0.95 : 0.75);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<g
	class="wedge"
	{onclick}
	{onmouseenter}
	{onmouseleave}
	role="button"
	tabindex="-1"
>
	<path
		d={path}
		fill={item.color}
		fill-opacity={fillOpacity}
		stroke="rgba(255,255,255,0.15)"
		stroke-width="1"
		class="wedge-path"
	/>
	<text
		x={labelPos.x}
		y={labelPos.y - 6}
		text-anchor="middle"
		dominant-baseline="central"
		class="wedge-icon"
	>
		{item.icon}
	</text>
	<text
		x={labelPos.x}
		y={labelPos.y + 14}
		text-anchor="middle"
		dominant-baseline="central"
		class="wedge-label"
	>
		{t(item.label)}
	</text>
</g>

<style>
	.wedge {
		cursor: pointer;
	}

	.wedge-path {
		transition: fill-opacity 0.15s ease;
	}

	.wedge-icon {
		font-size: 20px;
		fill: white;
		pointer-events: none;
		user-select: none;
		filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.4));
	}

	.wedge-label {
		font-size: 10px;
		fill: white;
		pointer-events: none;
		user-select: none;
		font-weight: 500;
		filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.5));
	}
</style>
