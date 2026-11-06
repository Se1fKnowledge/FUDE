<script lang="ts">
	import { t } from '#lib/i18n.svelte.ts';
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
			<h2>{step.caption || (strokes.length ? t('Штрих') : step.highlight ? targetLabel(step.highlight) : t('Шаг гайда'))}</h2>
			<p class="where">{t('Шаг гайда {n} из {of}', { n: selected + 1, of: script.steps.length })}</p>
		</div>
		<label class="field">{t('Подпись внизу экрана')}
			<input type="text" bind:value={script.steps[selected].caption} placeholder={t('Пусто — остаётся прежняя подпись')} />
		</label>
		<div class="field">
			<span>{t('Подсветка в PhotoCraft')}</span>
			<div class="row">
				<button class="target" onclick={onmap}><Icon name="target" size={16} /> {step.highlight ? targetLabel(step.highlight) : t('Нет')}</button>
				{#if step.highlight}<button class="quiet" onclick={() => delete script.steps[selected].highlight}>{t('Убрать')}</button>{/if}
			</div>
		</div>
		{#if lesson.task.steps.length}
			<label class="field">{t('Показывает шаг задания')}
				<select value={step.task_step ?? ''} onchange={(e) => (e.currentTarget.value ? (script.steps[selected].task_step = e.currentTarget.value) : delete script.steps[selected].task_step)}>
					<option value="">{t('— не связан —')}</option>
					{#each lesson.task.steps as st, i (st.id)}<option value={st.id}>{t('Шаг {n}: {text}', { n: i + 1, text: st.text || st.id })}</option>{/each}
				</select>
			</label>
		{/if}
		<label class="field">{t('Пауза после шага, мс')}
			<input type="number" min="0" max="15000" step="100" value={pauseOf(step)} onchange={(e) => (script.steps[selected].pause = Math.max(0, Math.min(15000, Number(e.currentTarget.value))))} />
		</label>
		{#if strokes.length}
			{@const b = innerOf(0)}
			<div class="box">
				<p class="box-title">{strokes.length > 1 ? t('Кисть для {n} штрихов', { n: strokes.length }) : t('Кисть штриха')}</p>
				<div class="grid">
					<label>{t('Размер, px')} <input type="number" min="1" max="500" value={b.size ?? 30} onchange={(e) => setStroke('size', Number(e.currentTarget.value))} /></label>
					<label>{t('Жёсткость, %')} <input type="number" min="0" max="100" value={Math.round((b.hardness ?? 1) * 100)} onchange={(e) => setStroke('hardness', Number(e.currentTarget.value) / 100)} /></label>
					<label>{t('Непрозрачность, %')} <input type="number" min="1" max="100" value={Math.round((b.opacity ?? 1) * 100)} onchange={(e) => setStroke('opacity', Number(e.currentTarget.value) / 100)} /></label>
					<label>{t('Цвет')} <input type="color" value={b.color ?? '#000000'} onchange={(e) => setStroke('color', e.currentTarget.value.toUpperCase())} /></label>
				</div>
			</div>
			{#if lesson.task.steps.length}
				<div class="box">
					<p class="box-title">{t('Использовать в задании')}</p>
					<p class="note">{t('Штрих станет линией-призраком: ученик увидит его полупрозрачным, когда попросит подсказку.')}</p>
					<div class="row">
						<select bind:value={ghostTarget} aria-label={t('Шаг задания')}>
							{#each lesson.task.steps as st, i (st.id)}<option value={i}>{t('Шаг {n}: {text}', { n: i + 1, text: st.text || st.id })}</option>{/each}
						</select>
						<button class="quiet" onclick={toGhost}>{t('В подсказку')}</button>
					</div>
				</div>
			{/if}
		{/if}
		<div class="row">
			<button class="quiet" onclick={() => onmove(-1)} disabled={selected === 0}>← {t('Раньше')}</button>
			<button class="quiet" onclick={() => onmove(1)} disabled={selected === script.steps.length - 1}>{t('Позже')} →</button>
		</div>
		<div class="field">
			<span>{t('Что делает шаг')}</span>
			{#each step.calls as c, i (i)}
				<div class="call">
					<span class="grow">{describeCall(c)}</span>
					<button class="icon" onclick={() => removeCall(i)} aria-label={t('Убрать действие')}><Icon name="close" size={13} /></button>
				</div>
			{:else}
				<p class="note">{t('Только подпись и пауза.')}</p>
			{/each}
		</div>
	{:else}
		<div class="head">
			<h2>{t('Кисть показа')}</h2>
			<p class="where">{t('Ничего не выбрано')}</p>
		</div>
		<p class="note">{t('Возьмите «Кисть показа» и рисуйте на холсте — каждый штрих станет клипом на дорожке «Штрихи». Или запишите урок прямо в PhotoCraft.')}</p>
		<div class="box">
			<div class="grid">
				<label>{t('Размер, px')} <input type="number" min="1" max="500" bind:value={brush.size} /></label>
				<label>{t('Жёсткость, %')} <input type="number" min="0" max="100" value={Math.round(brush.hardness * 100)} onchange={(e) => (brush.hardness = Number(e.currentTarget.value) / 100)} /></label>
				<label>{t('Непрозрачность, %')} <input type="number" min="1" max="100" value={Math.round(brush.opacity * 100)} onchange={(e) => (brush.opacity = Number(e.currentTarget.value) / 100)} /></label>
				<label>{t('Цвет')} <input type="color" bind:value={brush.color} /></label>
			</div>
		</div>
	{/if}
	<div class="safe">
		<p class="box-title"><Icon name="lock" size={14} /> {t('Безопасно для учеников')}</p>
		<p class="note">{t('Гайд только рисует, выбирает инструменты и подсвечивает. Файлы, сеть и горячие клавиши ему недоступны.')}</p>
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
