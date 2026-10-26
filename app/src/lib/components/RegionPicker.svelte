<script lang="ts">
	import type { Region } from '#lib/api.ts';

	// Drag a rectangle on the task picture to set a region as fractions of the canvas, so
	// authors never type coordinates.
	let {
		value = $bindable(),
		image = null,
		aspect = 1.5,
		others = [],
		color = 'var(--shu-500)',
		label = 'Область'
	}: { value: Region; image?: string | null; aspect?: number; others?: Region[]; color?: string; label?: string } = $props();

	let box: HTMLDivElement | undefined = $state();
	let start: [number, number] | null = null;

	const clamp = (v: number) => Math.min(1, Math.max(0, v));
	const round = (v: number) => Math.round(v * 1000) / 1000;

	function at(e: PointerEvent): [number, number] {
		const r = box!.getBoundingClientRect();
		return [clamp((e.clientX - r.left) / r.width), clamp((e.clientY - r.top) / r.height)];
	}

	function down(e: PointerEvent) {
		start = at(e);
		box!.setPointerCapture(e.pointerId);
	}

	function move(e: PointerEvent) {
		if (!start) return;
		const [x, y] = at(e);
		const x0 = Math.min(start[0], x);
		const y0 = Math.min(start[1], y);
		value = [round(x0), round(y0), round(Math.max(0.005, Math.abs(x - start[0]))), round(Math.max(0.005, Math.abs(y - start[1])))];
	}

	function up() {
		start = null;
	}

	const style = (r: Region) => `left:${r[0] * 100}%;top:${r[1] * 100}%;width:${r[2] * 100}%;height:${r[3] * 100}%`;
</script>

<div class="picker">
	<div
		class="box"
		bind:this={box}
		style="aspect-ratio: {aspect}"
		role="application"
		aria-label="{label}: потяните мышью, чтобы выделить прямоугольник"
		onpointerdown={down}
		onpointermove={move}
		onpointerup={up}
	>
		{#if image}<img src={image} alt="" draggable="false" />{/if}
		{#each others as o, i (i)}<span class="region other" style={style(o)}></span>{/each}
		<span class="region" style="{style(value)}; --c: {color}"></span>
	</div>
	<span class="mono coords">{label}: [{value.map((v) => v.toFixed(3)).join(', ')}]</span>
</div>

<style>
	.picker {
		display: grid;
		gap: 6px;
		max-width: 520px;
	}

	.box {
		position: relative;
		overflow: hidden;
		border-radius: 8px;
		background: #fff;
		box-shadow: inset 0 0 0 1px var(--paper-line);
		cursor: crosshair;
		touch-action: none;
		user-select: none;
	}

	img {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: fill;
		pointer-events: none;
	}

	.region {
		position: absolute;
		border: 2px solid var(--c, var(--shu-500));
		background: color-mix(in srgb, var(--c, var(--shu-500)) 14%, transparent);
		border-radius: 3px;
		pointer-events: none;
	}

	.other {
		--c: var(--ai-500);
		border-style: dashed;
	}

	.coords {
		color: var(--ink-500);
	}
</style>
