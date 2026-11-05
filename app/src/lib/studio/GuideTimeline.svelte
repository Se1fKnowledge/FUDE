<script lang="ts">
	import type { GuideStep } from '../api.ts';
	import Icon from '../components/Icon.svelte';
	import { count } from '../format.ts';
	import { clips, clock, pauseOf, setPause, times, TRACKS, type Track } from './timeline.ts';

	// The guide laid out on tracks under the canvas: captions, highlights, actions, strokes.
	let {
		steps,
		selected = -1,
		playhead = 0,
		playing = false,
		recording = false,
		recordLabel = '',
		onselect,
		onseek,
		onsteps,
		onplay,
		onrecord,
		onaddcaption,
		onremove
	}: {
		steps: GuideStep[];
		selected?: number;
		playhead?: number;
		playing?: boolean;
		recording?: boolean;
		recordLabel?: string;
		onselect: (i: number) => void;
		onseek: (ms: number) => void;
		onsteps: (steps: GuideStep[]) => void;
		onplay: () => void;
		onrecord: () => void;
		onaddcaption: () => void;
		onremove: () => void;
	} = $props();

	const LABEL = 116;
	let lanes: HTMLDivElement | undefined = $state();
	let laneWidth = $state(600);

	const layout = $derived(times(steps));
	const all = $derived(clips(steps));
	const total = $derived(Math.max(layout.total, 4000));
	const pxPerMs = $derived(Math.max(laneWidth / total, 0.03));
	const width = $derived(total * pxPerMs);
	const ticks = $derived(Array.from({ length: Math.floor(total / 1000) + 1 }, (_, i) => i));
	const byTrack = (t: Track) => all.filter((c) => c.track === t);

	let resizing: { step: number; startX: number; pause: number } | null = null;

	function seekFrom(e: PointerEvent) {
		const r = lanes!.getBoundingClientRect();
		onseek(Math.max(0, Math.min(layout.total, (e.clientX - r.left + lanes!.scrollLeft) / pxPerMs)));
	}

	function startResize(e: PointerEvent, step: number) {
		e.stopPropagation();
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		resizing = { step, startX: e.clientX, pause: pauseOf(steps[step]) };
	}

	function resizeMove(e: PointerEvent) {
		if (!resizing) return;
		onsteps(setPause(steps, resizing.step, resizing.pause + (e.clientX - resizing.startX) / pxPerMs));
	}

	function onkey(e: KeyboardEvent, step: number) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			onselect(step);
		}
		if (e.key === 'ArrowRight' && e.altKey) onsteps(setPause(steps, step, pauseOf(steps[step]) + 100));
		if (e.key === 'ArrowLeft' && e.altKey) onsteps(setPause(steps, step, pauseOf(steps[step]) - 100));
		if (e.key === 'Delete') onremove();
	}

	const ICON: Record<Track, 'pencil' | 'target' | 'sparkle' | 'brush'> = { caption: 'pencil', highlight: 'target', action: 'sparkle', stroke: 'brush' };
</script>

<section class="timeline" aria-label="Дорожки гайда">
	<div class="transport">
		<button class="play" onclick={onplay} aria-label={playing ? 'Пауза' : 'Проиграть на холсте'} disabled={!steps.length || recording}>
			<Icon name={playing ? 'stop' : 'play'} size={14} />
		</button>
		<span class="time mono">{clock(playhead)} / {clock(layout.total)}</span>
		<span class="count">{steps.length ? count(steps.length, ['шаг', 'шага', 'шагов']) : 'Гайд пуст'}</span>
		<span class="grow"></span>
		<button class="tbtn" onclick={onaddcaption} disabled={recording}><Icon name="plus" size={14} /> Подпись</button>
		<button class="tbtn" onclick={onremove} disabled={selected < 0 || recording} aria-label="Удалить выбранный шаг"><Icon name="trash" size={14} /></button>
		<button class="rec" class:on={recording} onclick={onrecord}>
			<span class="dot"></span>{recording ? recordLabel || 'Остановить запись' : 'Записать в PhotoCraft'}
		</button>
	</div>

	<div class="body">
		<div class="labels" style:width="{LABEL}px">
			<div class="ruler-pad"></div>
			{#each TRACKS as t (t.id)}
				<div class="label-row"><Icon name={ICON[t.id]} size={14} /> {t.label}</div>
			{/each}
		</div>
		<div class="lanes" bind:this={lanes} bind:clientWidth={laneWidth} onpointermove={resizeMove} onpointerup={() => (resizing = null)} role="presentation">
			<div class="inner" style:width="{width}px">
				<div class="ruler" onpointerdown={seekFrom} role="slider" aria-label="Время гайда" aria-valuemin={0} aria-valuemax={layout.total} aria-valuenow={Math.round(playhead)} tabindex="0">
					{#each ticks as s (s)}
						<span class="tick" class:major={s % 2 === 0} style:left="{s * 1000 * pxPerMs}px">{#if s % 2 === 0}{s} с{/if}</span>
					{/each}
				</div>
				{#each TRACKS as t (t.id)}
					<div class="lane" onpointerdown={seekFrom} role="presentation">
						{#each byTrack(t.id) as c (c.track + c.step)}
							<div
								class="clip {c.track}"
								class:on={c.step === selected}
								style:left="{c.start * pxPerMs}px"
								style:width="{Math.max(10, (c.end - c.start) * pxPerMs - 2)}px"
								onpointerdown={(e) => (e.stopPropagation(), onselect(c.step))}
								onkeydown={(e) => onkey(e, c.step)}
								role="button"
								tabindex="0"
								title={c.label}
							>
								<span>{c.label}</span>
								{#if c.track !== 'caption' && c.step === selected}
									<span class="grip" onpointerdown={(e) => startResize(e, c.step)} role="presentation" title="Потяните, чтобы изменить паузу"></span>
								{/if}
							</div>
						{/each}
					</div>
				{/each}
				<div class="playhead" style:left="{playhead * pxPerMs}px"></div>
			</div>
		</div>
	</div>
</section>

<style>
	.timeline {
		display: flex;
		flex-direction: column;
		background: var(--sumi-panel);
		border-top: 1px solid var(--sumi-line);
		color: var(--sumi-text);
		font: 500 12px var(--font-ui);
	}

	.transport {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--sumi-line);
	}

	.grow {
		flex: 1;
	}

	.play {
		display: grid;
		place-items: center;
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 0;
		background: var(--sumi-text);
		color: var(--ink-900);
		cursor: pointer;
	}

	.play:disabled {
		opacity: 0.4;
		cursor: default;
	}

	.time {
		color: var(--sumi-text);
		font-variant-numeric: tabular-nums;
	}

	.count {
		color: var(--sumi-muted);
	}

	.tbtn,
	.rec {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 12px;
		border-radius: 8px;
		border: 1px solid var(--sumi-line);
		background: transparent;
		color: var(--sumi-text);
		font: 600 12px var(--font-ui);
		cursor: pointer;
	}

	.tbtn:disabled {
		opacity: 0.4;
		cursor: default;
	}

	.rec {
		border-color: transparent;
		background: var(--shu-500);
		color: #fff;
	}

	.rec.on {
		background: var(--shu-700);
	}

	.rec .dot {
		width: 9px;
		height: 9px;
		border-radius: 50%;
		background: #fff;
	}

	.rec.on .dot {
		animation: blink 1s steps(2) infinite;
		border-radius: 2px;
	}

	@keyframes blink {
		50% {
			opacity: 0.3;
		}
	}

	.body {
		display: flex;
		min-height: 0;
	}

	.labels {
		flex: none;
		color: var(--sumi-muted);
	}

	.ruler-pad {
		height: 24px;
	}

	.label-row {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 38px;
		padding-left: 12px;
	}

	.lanes {
		flex: 1;
		overflow-x: auto;
		position: relative;
		scrollbar-color: var(--sumi-line) transparent;
	}

	.inner {
		position: relative;
		min-width: 100%;
		padding-right: 24px;
	}

	.ruler {
		position: relative;
		height: 24px;
		cursor: pointer;
	}

	.tick {
		position: absolute;
		top: 4px;
		padding-left: 4px;
		border-left: 1px solid var(--sumi-line);
		height: 16px;
		color: var(--sumi-muted);
		font-size: 11px;
		font-variant-numeric: tabular-nums;
	}

	.tick:not(.major) {
		top: 12px;
		height: 8px;
	}

	.lane {
		position: relative;
		height: 34px;
		margin-bottom: 4px;
		border-radius: 6px;
		background: var(--sumi-bg);
	}

	.clip {
		position: absolute;
		top: 2px;
		height: 30px;
		display: flex;
		align-items: center;
		padding: 0 8px;
		border-radius: 6px;
		overflow: hidden;
		white-space: nowrap;
		cursor: pointer;
		color: var(--sumi-text);
		outline: none;
	}

	.clip span:first-child {
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.clip.caption {
		background: var(--ai-700);
	}

	.clip.highlight {
		background: var(--sumi-line);
	}

	.clip.action {
		background: var(--kaki-700);
	}

	.clip.stroke {
		background: var(--shu-700);
	}

	.clip.on,
	.clip:focus-visible {
		box-shadow: inset 0 0 0 2px var(--sumi-text);
	}

	.grip {
		position: absolute;
		right: 0;
		top: 0;
		bottom: 0;
		width: 8px;
		cursor: ew-resize;
		background: rgb(255 255 255 / 25%);
	}

	.playhead {
		position: absolute;
		top: 4px;
		bottom: 0;
		width: 2px;
		margin-left: -1px;
		background: var(--shu-500);
		pointer-events: none;
	}

	.playhead::before {
		content: '';
		position: absolute;
		top: -2px;
		left: -5px;
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: var(--shu-500);
	}
</style>
