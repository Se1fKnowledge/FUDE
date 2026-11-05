<script lang="ts" module>
	import type { Point, Region, Shape } from '../api.ts';

	export type CanvasTool = 'select' | 'path' | 'line' | 'arrow' | 'ellipse' | 'rect' | 'region' | 'eraser' | 'brush';

	export interface RegionMark {
		key: string;
		label: string;
		region: Region;
		color: string;
		name: string;
	}

	export interface DemoStroke {
		points: number[][];
		size: number;
		color: string;
		opacity: number;
		/** How much of the stroke is drawn, 0–1. */
		progress: number;
	}
</script>

<script lang="ts">
	import { hit, moveRegion, moveShape, regionFromDrag, round3, shapeFromDrag, styleOf, svgPath, clamp01, type DrawTool } from './geometry.ts';

	// The task picture with everything the author puts on it, drawn in picture pixels.
	let {
		image = null,
		reference = null,
		referenceOpacity = 0,
		width,
		height,
		background = '#FFFFFF',
		tool = 'select',
		shapes = [],
		dimShapes = [],
		selected = -1,
		regions = [],
		selectedRegion = null,
		demo = [],
		caption = '',
		onshapes,
		onselect,
		onregion,
		onregionselect,
		onregiondraw,
		onstroke
	}: {
		image?: string | null;
		reference?: string | null;
		referenceOpacity?: number;
		width: number;
		height: number;
		background?: string;
		tool?: CanvasTool;
		shapes?: Shape[];
		dimShapes?: Shape[];
		selected?: number;
		regions?: RegionMark[];
		selectedRegion?: string | null;
		demo?: DemoStroke[];
		caption?: string;
		onshapes?: (shapes: Shape[]) => void;
		onselect?: (index: number) => void;
		onregion?: (key: string, region: Region) => void;
		onregionselect?: (key: string | null) => void;
		onregiondraw?: (region: Region) => void;
		onstroke?: (points: number[][]) => void;
	} = $props();

	let svg: SVGSVGElement | undefined = $state();
	type Drag =
		| { kind: 'draw'; start: Point; trail: Point[] }
		| { kind: 'move'; index: number; last: Point; original: Shape[] }
		| { kind: 'region-move'; key: string; last: Point; region: Region }
		| { kind: 'region-resize'; key: string; region: Region }
		| { kind: 'region-draw'; start: Point; now: Point }
		| { kind: 'brush'; points: number[][] };
	let drag = $state<Drag | null>(null);
	let preview = $state<Shape | null>(null);

	function at(e: PointerEvent): Point {
		const r = svg!.getBoundingClientRect();
		return [clamp01((e.clientX - r.left) / r.width), clamp01((e.clientY - r.top) / r.height)];
	}

	function down(e: PointerEvent) {
		if (e.button !== 0) return;
		const p = at(e);
		svg!.setPointerCapture(e.pointerId);
		if (tool === 'brush') {
			drag = { kind: 'brush', points: [[round3(p[0] * width), round3(p[1] * height), e.pressure || 1]] };
			return;
		}
		if (tool === 'region') {
			drag = { kind: 'region-draw', start: p, now: p };
			return;
		}
		if (tool === 'eraser') {
			const i = hit(shapes, p, tolerance());
			if (i >= 0) onshapes?.(shapes.filter((_, j) => j !== i));
			return;
		}
		if (tool === 'select') {
			const i = hit(shapes, p, tolerance());
			onselect?.(i);
			if (i >= 0) {
				onregionselect?.(null);
				drag = { kind: 'move', index: i, last: p, original: shapes };
			} else {
				onregionselect?.(null);
			}
			return;
		}
		drag = { kind: 'draw', start: p, trail: [p] };
	}

	function move(e: PointerEvent) {
		if (!drag) return;
		const p = at(e);
		if (drag.kind === 'brush') {
			drag.points.push([round3(p[0] * width), round3(p[1] * height), e.pressure || 1]);
		} else if (drag.kind === 'draw') {
			drag.trail.push(p);
			preview = shapeFromDrag(tool as DrawTool, drag.start, constrain(drag.start, p, e.shiftKey), drag.trail);
		} else if (drag.kind === 'move') {
			const [dx, dy] = [p[0] - drag.last[0], p[1] - drag.last[1]];
			drag.last = p;
			onshapes?.(shapes.map((s, j) => (j === (drag as { index: number }).index ? moveShape(s, dx, dy) : s)));
		} else if (drag.kind === 'region-move') {
			drag.region = moveRegion(drag.region, p[0] - drag.last[0], p[1] - drag.last[1]);
			drag.last = p;
			onregion?.(drag.key, drag.region);
		} else if (drag.kind === 'region-resize') {
			const [x, y] = drag.region;
			onregion?.(drag.key, regionFromDrag([x, y], p));
		} else if (drag.kind === 'region-draw') {
			drag.now = p;
		}
	}

	function up(e: PointerEvent) {
		if (!drag) return;
		const p = at(e);
		if (drag.kind === 'brush' && drag.points.length > 1) onstroke?.(drag.points);
		if (drag.kind === 'draw') {
			const shape = shapeFromDrag(tool as DrawTool, drag.start, constrain(drag.start, p, e.shiftKey), drag.trail);
			if (shape) {
				const style = e.altKey ? { dashed: true } : undefined;
				onshapes?.([...shapes, style ? { ...shape, style } : shape]);
				onselect?.(shapes.length);
			}
		}
		if (drag.kind === 'region-draw') {
			const r = regionFromDrag(drag.start, p, 0.02);
			if (Math.hypot(drag.start[0] - p[0], drag.start[1] - p[1]) > 0.01) onregiondraw?.(r);
		}
		drag = null;
		preview = null;
	}

	/** Shift keeps a line straight: horizontal, vertical or at 45°. */
	function constrain(a: Point, b: Point, on: boolean): Point {
		if (!on || (tool !== 'line' && tool !== 'arrow')) return b;
		const [dx, dy] = [b[0] - a[0], b[1] - a[1]];
		const angle = Math.round(Math.atan2(dy, dx) / (Math.PI / 4)) * (Math.PI / 4);
		const len = Math.hypot(dx, dy);
		return [clamp01(a[0] + Math.cos(angle) * len), clamp01(a[1] + Math.sin(angle) * len)];
	}

	const tolerance = () => {
		const r = svg?.getBoundingClientRect();
		return r ? 10 / Math.min(r.width, r.height) : 0.015;
	};

	function regionDown(e: PointerEvent, mark: RegionMark, resize = false) {
		// With the region tool a drag always draws a new area, even over an existing one.
		if (tool !== 'select') return;
		e.stopPropagation();
		svg!.setPointerCapture(e.pointerId);
		onregionselect?.(mark.key);
		onselect?.(-1);
		drag = resize ? { kind: 'region-resize', key: mark.key, region: mark.region } : { kind: 'region-move', key: mark.key, last: at(e), region: mark.region };
	}

	const firstPoint = (s: Shape): Point => (s.kind === 'line' || s.kind === 'arrow' ? s.from : s.kind === 'path' ? s.points[0] : [s.rect[0], s.rect[1]]);
	const scale = $derived(Math.max(width, height) / 1000);
	const dash = (s: Shape) => {
		const st = styleOf(s, width, height);
		return st.dashed ? `${st.width * 4} ${st.width * 3}` : undefined;
	};

	function demoPath(d: DemoStroke): string {
		const n = Math.max(2, Math.ceil(d.points.length * d.progress));
		return d.points
			.slice(0, n)
			.map(([x, y], i) => `${i ? 'L' : 'M'}${x.toFixed(1)} ${y.toFixed(1)}`)
			.join(' ');
	}
	const drawing = $derived(drag?.kind === 'region-draw' ? regionFromDrag(drag.start, drag.now, 0.001) : null);
	const brushTrail = $derived(drag?.kind === 'brush' ? drag.points.map(([x, y], i) => `${i ? 'L' : 'M'}${x} ${y}`).join(' ') : '');
</script>

<div class="frame" style:aspect-ratio="{width} / {height}">
	{#if image}
		<img src={image} alt="Картинка задания" draggable="false" />
	{:else}
		<div class="blank" style:background></div>
	{/if}
	{#if reference && referenceOpacity > 0}
		<img class="reference" src={reference} alt="Эталон поверх задания" style:opacity={referenceOpacity} draggable="false" />
	{/if}
	<svg
		bind:this={svg}
		viewBox="0 0 {width} {height}"
		class="tool-{tool}"
		role="application"
		aria-label="Холст урока: рисуйте линии-призраки и области проверок"
		onpointerdown={down}
		onpointermove={move}
		onpointerup={up}
		onpointercancel={() => ((drag = null), (preview = null))}
	>
		{#each dimShapes as s, i (i)}
			<path d={svgPath(s, width, height)} class="dim" stroke-width={styleOf(s, width, height).width} />
		{/each}

		{#each demo as d, i (i)}
			<path d={demoPath(d)} class="demo" stroke={d.color} stroke-width={d.size} stroke-opacity={d.opacity} />
		{/each}
		{#if brushTrail}<path d={brushTrail} class="demo live" stroke-width={12 * scale} />{/if}

		{#each regions as m (m.key)}
			{@const [x, y, w, h] = m.region}
			<g class="region" class:on={selectedRegion === m.key} style:--c={m.color}>
				<rect x={x * width} y={y * height} width={w * width} height={h * height} onpointerdown={(e) => regionDown(e, m)} role="presentation" />
				<g transform="translate({x * width} {y * height - 26 * scale}) scale({scale})">
					<rect class="tag" width={30 + m.name.length * 7.2} height="22" rx="6" />
					<text x="8" y="15" class="tag-key">{m.label}</text>
					<text x="24" y="15" class="tag-name">{m.name}</text>
				</g>
				{#if selectedRegion === m.key}
					<rect class="handle" x={(x + w) * width - 7 * scale} y={(y + h) * height - 7 * scale} width={14 * scale} height={14 * scale} onpointerdown={(e) => regionDown(e, m, true)} role="presentation" />
				{/if}
			</g>
		{/each}
		{#if drawing}
			<rect class="drawing" x={drawing[0] * width} y={drawing[1] * height} width={drawing[2] * width} height={drawing[3] * height} />
		{/if}

		{#each shapes as s, i (i)}
			{@const st = styleOf(s, width, height)}
			<path d={svgPath(s, width, height)} class="ghost" class:on={selected === i} stroke={st.color} stroke-opacity={Math.max(st.opacity, 0.2)} stroke-width={st.width} stroke-dasharray={dash(s)} />
		{/each}
		{#if preview}
			{@const st = styleOf(preview, width, height)}
			<path d={svgPath(preview, width, height)} class="ghost" stroke={st.color} stroke-opacity="0.5" stroke-width={st.width} />
		{/if}
		{#each shapes as s, i (i)}
			{@const [px, py] = firstPoint(s)}
			<g class="seal" class:on={selected === i} transform="translate({px * width} {py * height}) scale({scale})">
				<circle r="12" />
				<text y="4.5">{i + 1}</text>
			</g>
		{/each}
	</svg>
	{#if caption}<p class="caption">{caption}</p>{/if}
</div>

<style>
	.frame {
		position: relative;
		max-width: 100%;
		max-height: 100%;
		box-shadow: 0 24px 60px rgb(0 0 0 / 45%);
		user-select: none;
	}

	img,
	.blank,
	svg {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}

	img {
		object-fit: fill;
	}

	svg {
		touch-action: none;
		cursor: crosshair;
		overflow: visible;
	}

	svg.tool-select {
		cursor: default;
	}

	svg.tool-eraser {
		cursor: not-allowed;
	}

	.ghost,
	.dim,
	.demo {
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.ghost.on {
		filter: drop-shadow(0 0 4px rgb(143 178 214 / 90%));
	}

	.dim {
		stroke: var(--shu-500);
		stroke-opacity: 0.16;
	}

	.demo.live {
		stroke: var(--kaki-500);
		stroke-opacity: 0.8;
	}

	.region rect:first-child {
		fill: color-mix(in srgb, var(--c) 14%, transparent);
		stroke: var(--c);
		stroke-width: calc(2px * 1);
		stroke-dasharray: 8 6;
		vector-effect: non-scaling-stroke;
		cursor: move;
	}

	.region.on rect:first-child {
		stroke-dasharray: none;
	}

	.tag {
		fill: var(--c);
	}

	.tag-key,
	.tag-name {
		font: 600 12px var(--font-ui);
		fill: #fff;
	}

	.handle {
		fill: #fff;
		stroke: var(--c);
		stroke-width: 2;
		vector-effect: non-scaling-stroke;
		cursor: nwse-resize;
	}

	.drawing {
		fill: rgb(47 93 140 / 14%);
		stroke: var(--ai-300);
		stroke-dasharray: 6 4;
		vector-effect: non-scaling-stroke;
	}

	.seal circle {
		fill: var(--shu-500);
		stroke: #fff;
		stroke-width: 2;
	}

	.seal.on circle {
		fill: var(--ai-500);
	}

	.seal text {
		font: 700 12px var(--font-ui);
		fill: #fff;
		text-anchor: middle;
		pointer-events: none;
	}

	.caption {
		position: absolute;
		left: 50%;
		bottom: 16px;
		transform: translateX(-50%);
		margin: 0;
		padding: 8px 16px;
		border-radius: 999px;
		background: rgb(17 18 22 / 85%);
		color: var(--sumi-text);
		font: 600 13px var(--font-ui);
		white-space: nowrap;
		pointer-events: none;
	}
</style>
