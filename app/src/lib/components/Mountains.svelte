<script lang="ts">
	import { ridge } from '#lib/ink.ts';

	// Misty mountains in washes of indigo ink, nearest layer darkest.
	let { width = 1440, height = 220, seed = 3 }: { width?: number; height?: number; seed?: number } = $props();
	const layers = $derived([
		{ d: ridge(width, height, 0.38, 0.09, seed), color: 'var(--ai-300)', opacity: 0.35, blur: 3 },
		{ d: ridge(width, height, 0.55, 0.08, seed + 1), color: 'var(--ai-300)', opacity: 0.55, blur: 1.5 },
		{ d: ridge(width, height, 0.72, 0.06, seed + 2), color: 'var(--ai-500)', opacity: 0.45, blur: 0.6 }
	]);
	const id = `mist-${Math.random().toString(36).slice(2)}`;
</script>

<svg viewBox="0 0 {width} {height}" preserveAspectRatio="none" aria-hidden="true">
	<defs>
		{#each layers as layer, i (i)}
			<filter id="{id}-{i}" x="-5%" y="-20%" width="110%" height="140%">
				<feGaussianBlur stdDeviation={layer.blur} />
			</filter>
		{/each}
	</defs>
	{#each layers as layer, i (i)}
		<path d={layer.d} fill={layer.color} opacity={layer.opacity} filter="url(#{id}-{i})" />
	{/each}
</svg>

<style>
	svg {
		display: block;
		width: 100%;
		height: 100%;
	}
</style>
