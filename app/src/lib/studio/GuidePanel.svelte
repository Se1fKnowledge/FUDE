<script lang="ts">
	import type { GuideScript, Lesson, Shape } from '../api.ts';
	import Icon from '../components/Icon.svelte';
	import { strokeToShape } from './geometry.ts';
	import { describeCall, isStroke, pauseOf, strokePoints, targetLabel, type BrushSettings } from './timeline.ts';

	// The inspector of the selected guide step, or the brush for drawing new strokes.
	let {
		script = $bindable(),
		brush = $bindable(),
		selected,
		lesson,
		width,
		height,
		onmap,
		onghost,
		onmove
	}: {
		script: GuideScript;
		brush: BrushSettings;
		selected: number;
		lesson: Lesson;
		width: number;
		height: number;
		onmap: () => void;
		onghost: (taskStep: number, shapes: Shape[]) => void;
		onmove: (by: number) => void;
	} = $props();

	const step = $derived(script.steps[selected] ?? null);
	const strokes = $derived(step ? step.calls.filter(isStroke) : []);
	let ghostTarget = $state(0);

	type Inner = { size?: number; hardness?: number; opacity?: number; color?: string };
	const innerOf = (i: number) => (strokes[i]?.params.params ?? {}) as Inner;

	function setStroke(key: keyof Inner, value: number | string) {
		if (!step) return;
		for (const c of script.steps[selected].calls) {
			if (isStroke(c)) (c.params.params as Inner)[key] = value as never;
		}
	}

	function removeCall(i: number) {
		script.steps[selected].calls.splice(i, 1);
	}

	function toGhost() {
		const shapes = strokes.map((c) => strokeToShape(strokePoints(c), width, height, (c.params.params as Inner).size)).filter((s): s is Shape => s !== null);
		if (shapes.length) onghost(ghostTarget, shapes);
	}
</script>

<div class="panel">
	{#if step}
		<div class="head">
			<h2>{step.caption || (strokes.length ? 'Штрих' : step.highlight ? targetLabel(step.highlight) : 'Шаг гайда')}</h2>
			<p class="where">Шаг гайда {selected + 1} из {script.steps.length}</p>
		</div>
		<label class="field">Подпись внизу экрана
			<input type="text" bind:value={script.steps[selected].caption} placeholder="Пусто — остаётся прежняя подпись" />
		</label>
		<div class="field">
			<span>Подсветка в PhotoCraft</span>
			<div class="row">
				<button class="target" onclick={onmap}><Icon name="target" size={16} /> {step.highlight ? targetLabel(step.highlight) : 'Нет'}</button>
				{#if step.highlight}<button class="quiet" onclick={() => delete script.steps[selected].highlight}>Убрать</button>{/if}
			</div>
		</div>
		{#if lesson.task.steps.length}
			<label class="field">Показывает шаг задания
				<select value={step.task_step ?? ''} onchange={(e) => (e.currentTarget.value ? (script.steps[selected].task_step = e.currentTarget.value) : delete script.steps[selected].task_step)}>
					<option value="">— не связан —</option>
					{#each lesson.task.steps as st, i (st.id)}<option value={st.id}>Шаг {i + 1}: {st.text || st.id}</option>{/each}
				</select>
			</label>
		{/if}
		<label class="field">Пауза после шага, мс
			<input type="number" min="0" max="15000" step="100" value={pauseOf(step)} onchange={(e) => (script.steps[selected].pause = Math.max(0, Math.min(15000, Number(e.currentTarget.value))))} />
		</label>
		{#if strokes.length}
			{@const b = innerOf(0)}
			<div class="box">
				<p class="box-title">Кисть {strokes.length > 1 ? `для ${strokes.length} штрихов` : 'штриха'}</p>
				<div class="grid">
					<label>Размер, px <input type="number" min="1" max="500" value={b.size ?? 30} onchange={(e) => setStroke('size', Number(e.currentTarget.value))} /></label>
					<label>Жёсткость, % <input type="number" min="0" max="100" value={Math.round((b.hardness ?? 1) * 100)} onchange={(e) => setStroke('hardness', Number(e.currentTarget.value) / 100)} /></label>
					<label>Непрозрачность, % <input type="number" min="1" max="100" value={Math.round((b.opacity ?? 1) * 100)} onchange={(e) => setStroke('opacity', Number(e.currentTarget.value) / 100)} /></label>
					<label>Цвет <input type="color" value={b.color ?? '#000000'} onchange={(e) => setStroke('color', e.currentTarget.value.toUpperCase())} /></label>
				</div>
			</div>
			{#if lesson.task.steps.length}
				<div class="box">
					<p class="box-title">Использовать в задании</p>
					<p class="note">Штрих станет линией-призраком: ученик увидит его полупрозрачным, когда попросит подсказку.</p>
					<div class="row">
						<select bind:value={ghostTarget} aria-label="Шаг задания">
							{#each lesson.task.steps as st, i (st.id)}<option value={i}>Шаг {i + 1}: {st.text || st.id}</option>{/each}
						</select>
						<button class="quiet" onclick={toGhost}>В подсказку</button>
					</div>
				</div>
			{/if}
		{/if}
		<div class="row">
			<button class="quiet" onclick={() => onmove(-1)} disabled={selected === 0}>← Раньше</button>
			<button class="quiet" onclick={() => onmove(1)} disabled={selected === script.steps.length - 1}>Позже →</button>
		</div>
		<div class="field">
			<span>Что делает шаг</span>
			{#each step.calls as c, i (i)}
				<div class="call">
					<span class="grow">{describeCall(c)}</span>
					<button class="icon" onclick={() => removeCall(i)} aria-label="Убрать действие"><Icon name="close" size={13} /></button>
				</div>
			{:else}
				<p class="note">Только подпись и пауза.</p>
			{/each}
		</div>
	{:else}
		<div class="head">
			<h2>Кисть показа</h2>
			<p class="where">Ничего не выбрано</p>
		</div>
		<p class="note">Возьмите «Кисть показа» и рисуйте на холсте — каждый штрих станет клипом на дорожке «Штрихи». Или запишите урок прямо в PhotoCraft.</p>
		<div class="box">
			<div class="grid">
				<label>Размер, px <input type="number" min="1" max="500" bind:value={brush.size} /></label>
				<label>Жёсткость, % <input type="number" min="0" max="100" value={Math.round(brush.hardness * 100)} onchange={(e) => (brush.hardness = Number(e.currentTarget.value) / 100)} /></label>
				<label>Непрозрачность, % <input type="number" min="1" max="100" value={Math.round(brush.opacity * 100)} onchange={(e) => (brush.opacity = Number(e.currentTarget.value) / 100)} /></label>
				<label>Цвет <input type="color" bind:value={brush.color} /></label>
			</div>
		</div>
	{/if}
	<div class="safe">
		<p class="box-title"><Icon name="lock" size={14} /> Безопасно для учеников</p>
		<p class="note">Гайд только рисует, выбирает инструменты и подсвечивает. Файлы, сеть и горячие клавиши ему недоступны.</p>
	</div>
</div>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	h2 {
		margin: 0;
		font: 700 18px/1.3 var(--font-ui);
		color: var(--sumi-text);
	}

	.where,
	.note {
		margin: 0;
		color: var(--sumi-muted);
		font: 12px/1.5 var(--font-ui);
	}

	.field,
	label {
		display: flex;
		flex-direction: column;
		gap: 6px;
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
	}

	input[type='text'],
	input[type='number'],
	select {
		min-width: 0;
		padding: 8px 10px;
		border: 1px solid var(--sumi-line);
		border-radius: 8px;
		background: var(--sumi-bg);
		color: var(--sumi-text);
		font: 13px var(--font-ui);
	}

	input[type='color'] {
		width: 100%;
		height: 34px;
		border: 1px solid var(--sumi-line);
		border-radius: 8px;
		background: none;
	}

	.row {
		display: flex;
		gap: 8px;
	}

	.row select {
		flex: 1;
	}

	.grow {
		flex: 1;
	}

	.quiet:disabled {
		opacity: 0.45;
		cursor: default;
	}

	.target,
	.quiet {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border: 1px solid var(--sumi-line);
		border-radius: 8px;
		background: var(--sumi-raised);
		color: var(--sumi-text);
		font: 600 13px var(--font-ui);
		cursor: pointer;
	}

	.target {
		flex: 1;
		background: var(--sumi-bg);
	}

	.box,
	.safe {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px;
		border: 1px solid var(--sumi-line);
		border-radius: 8px;
		background: var(--sumi-bg);
	}

	.safe {
		margin-top: auto;
		border-color: transparent;
		background: var(--sumi-raised);
	}

	.box-title {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 0;
		color: var(--sumi-text);
		font: 600 13px var(--font-ui);
	}

	.grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
	}

	.call {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 8px 6px 10px;
		border-radius: 6px;
		background: var(--sumi-bg);
		color: var(--sumi-text);
		font: 500 13px var(--font-ui);
	}

	.icon {
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--sumi-muted);
		cursor: pointer;
	}
</style>
