<script lang="ts">
	import { MIST, mistWindow } from '#lib/ink.ts';

	// Misty mountains in washes of indigo ink, nearest layer darkest: brush lines along the
	// ridges and washes fading into mist, from real sumi strokes. The seed picks which hills
	// show, so cards side by side don't repeat.
	let { width = 1440, height = 220, seed = 3 }: { width?: number; height?: number; seed?: number } = $props();

	const view = $derived(mistWindow(width, height, seed));
	const layers = $derived([
		{ src: MIST.far, color: 'var(--ai-300)', opacity: 0.4 },
		{ src: MIST.mid, color: 'var(--ai-300)', opacity: 0.6 },
		{ src: MIST.near, color: 'var(--ai-500)', opacity: 0.55 }
	]);
	const id = `mist-${Math.random().toString(36).slice(2)}`;
</script>

<svg viewBox="0 0 {width} {height}" preserveAspectRatio="none" aria-hidden="true">
	{#each layers as layer, i (i)}
		<mask id="{id}-{i}" maskUnits="userSpaceOnUse" x="0" y="0" {width} {height}>
			<image href={layer.src} x={view.x} width={view.width} {height} preserveAspectRatio="none" />
		</mask>
		<rect {width} {height} fill={layer.color} opacity={layer.opacity} mask="url(#{id}-{i})" />
	{/each}
</svg>

<style>
	svg {
		display: block;
		width: 100%;
		height: 100%;
		/* The wash dissolves at the left instead of ending on the edge of its box. */
		mask-image: linear-gradient(to right, transparent, #000 22%);
	}
</style>
