<script lang="ts">
	import { pick, STROKES } from '#lib/ink.ts';

	// A horizontal ink stroke with a dry tail — a real sumi stroke filled with `color`: under the
	// active menu item and on lesson cards. `progress` shows only the start of it; `draw` paints
	// it left to right.
	let {
		width = 240,
		height = 44,
		seed = 1,
		color = 'var(--shu-500)',
		progress = 1,
		draw = false
	}: { width?: number; height?: number; seed?: number; color?: string; progress?: number; draw?: boolean } = $props();

	const src = $derived(pick(STROKES, seed));
	const clip = $derived(`inset(0 ${Math.round((1 - Math.min(1, Math.max(0, progress))) * 100)}% 0 0)`);
	const id = `stroke-${Math.random().toString(36).slice(2)}`;
</script>

<svg class:draw viewBox="0 0 {width} {height}" {width} {height} aria-hidden="true" style="clip-path: {clip}">
	<mask {id} maskUnits="userSpaceOnUse" x="0" y="0" {width} {height}>
		<image href={src} {width} {height} preserveAspectRatio="none" />
	</mask>
	<rect {width} {height} fill={color} mask="url(#{id})" />
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
