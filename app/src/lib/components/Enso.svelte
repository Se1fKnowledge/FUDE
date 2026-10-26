<script lang="ts">
	import { enso } from '#lib/ink.ts';

	// An ensō, one circular stroke. With `draw` the brush paints it around once.
	let {
		size = 200,
		width = 16,
		seed = 7,
		color = 'var(--ink-900)',
		draw = false,
		duration = 880,
		delay = 0
	}: { size?: number; width?: number; seed?: number; color?: string; draw?: boolean; duration?: number; delay?: number } = $props();

	const c = $derived(size / 2);
	const r = $derived(size / 2 - width);
	const d = $derived(enso(c, c, r, width, seed));
	const id = `enso-${Math.random().toString(36).slice(2)}`;
</script>

<svg viewBox="0 0 {size} {size}" width={size} height={size} aria-hidden="true">
	{#if draw}
		<defs>
			<mask {id}>
				<circle
					cx={c}
					cy={c}
					r={r}
					fill="none"
					stroke="#fff"
					stroke-width={width * 2.4}
					pathLength="100"
					stroke-dasharray="100"
					transform="rotate(-70 {c} {c})"
					style="animation: enso-draw {duration}ms var(--ease-ink) {delay}ms both"
				/>
			</mask>
		</defs>
		<path {d} fill={color} mask="url(#{id})" />
	{:else}
		<path {d} fill={color} />
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
