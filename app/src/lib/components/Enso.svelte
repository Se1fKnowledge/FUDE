<script lang="ts">
	import { ENSO, ENSO_RADIUS, ENSO_START, pick } from '#lib/ink.ts';

	// An ensō — a real sumi stroke bent into a circle and filled with `color`. With `draw` the
	// brush paints it around once, from where the stroke starts.
	let {
		size = 200,
		seed = 7,
		color = 'var(--ink-900)',
		draw = false,
		duration = 880,
		delay = 0
	}: { size?: number; seed?: number; color?: string; draw?: boolean; duration?: number; delay?: number } = $props();

	const src = $derived(pick(ENSO, seed));
	const c = $derived(size / 2);
	const id = `enso-${Math.random().toString(36).slice(2)}`;
</script>

<svg viewBox="0 0 {size} {size}" width={size} height={size} aria-hidden="true">
	<mask id="{id}-ink" maskUnits="userSpaceOnUse" x="0" y="0" width={size} height={size}>
		<image href={src} width={size} height={size} />
	</mask>
	{#if draw}
		<mask id="{id}-draw" maskUnits="userSpaceOnUse" x="0" y="0" width={size} height={size}>
			<circle
				cx={c}
				cy={c}
				r={size * ENSO_RADIUS}
				fill="none"
				stroke="#fff"
				stroke-width={size * 0.24}
				pathLength="100"
				stroke-dasharray="100"
				transform="rotate({ENSO_START} {c} {c})"
				style="animation: enso-draw {duration}ms var(--ease-ink) {delay}ms both"
			/>
		</mask>
		<g mask="url(#{id}-draw)">
			<rect width={size} height={size} fill={color} mask="url(#{id}-ink)" />
		</g>
	{:else}
		<rect width={size} height={size} fill={color} mask="url(#{id}-ink)" />
	{/if}
</svg>

<style>
	svg {
		display: block;
		overflow: visible;
	}

	:global {
		@keyframes enso-draw {
			from {
				stroke-dashoffset: 100;
			}
			to {
				stroke-dashoffset: 0;
			}
		}
	}
</style>
