<script lang="ts">
	import { brushStroke } from '#lib/ink.ts';

	// A horizontal ink stroke with a dry tail: under the active menu item, as a progress
	// bar, as a week bar laid on its side.
	let {
		width = 240,
		height = 44,
		seed = 1,
		color = 'var(--shu-500)',
		progress = 1,
		draw = false
	}: { width?: number; height?: number; seed?: number; color?: string; progress?: number; draw?: boolean } = $props();

	const stroke = $derived(brushStroke(width, height, seed));
	const clip = $derived(`inset(0 ${Math.round((1 - Math.min(1, Math.max(0, progress))) * 100)}% 0 0)`);
</script>

<svg class:draw viewBox="0 0 {width} {height}" {width} {height} aria-hidden="true" style="clip-path: {clip}">
	<path d={stroke.body} fill={color} />
	{#each stroke.hairs as hair (hair)}
		<path d={hair} stroke={color} stroke-width="1.2" stroke-linecap="round" fill="none" opacity="0.75" />
	{/each}
</svg>

<style>
	svg {
		display: block;
		overflow: visible;
		transition: clip-path 700ms var(--ease-ink);
	}

	.draw {
		animation: draw 620ms var(--ease-ink) both;
	}

	@keyframes draw {
		from {
			clip-path: inset(0 100% 0 0);
		}
	}
</style>
