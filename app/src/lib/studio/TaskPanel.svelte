<script lang="ts" module>
	export type TaskTab = 'text' | 'hint' | 'checks';
</script>

<script lang="ts">
	import type { Check, CheckKind, CheckProof, Lesson, Shape, ShapeStyle } from '../api.ts';
	import { CHECK_KINDS, defaultCheck } from '../checks.ts';
	import Icon from '../components/Icon.svelte';
	import CheckCard from './CheckCard.svelte';
	import { cleanStyle, SHAPE_LABELS, styleOf } from './geometry.ts';
	import { targetLabel } from './timeline.ts';

	// The inspector of the selected task step: its text, its hint with the ghost lines drawn on
	// the canvas, and its checks with what the live check found.
	let {
		lesson = $bindable(),
		step,
		tab = $bindable(),
		selectedShape = -1,
		selectedRegion = null,
		width,
		height,
		proofs = [],
		letters,
		onselectshape,
		onfocusregion,
		onmap,
		onhintpreview,
		onremovestep,
		onmovestep,
		taskImage = null,
		referenceImage = null,
		marks = [],
		background = '#FFFFFF'
	}: {
		lesson: Lesson;
		step: number;
		tab: TaskTab;
		selectedShape?: number;
		selectedRegion?: string | null;
		width: number;
		height: number;
		proofs?: CheckProof[];
		letters: (check: number) => { letters: string[]; colors: string[]; keys: string[] };
		onselectshape: (i: number) => void;
		onfocusregion: (key: string) => void;
		onmap: () => void;
		onhintpreview: () => void;
		onremovestep: () => void;
		onmovestep: (by: number) => void;
		taskImage?: string | null;
		referenceImage?: string | null;
		marks?: { key: string; label: string; region: [number, number, number, number]; color: string }[];
		background?: string;
	} = $props();

	const s = $derived(lesson.task.steps[step]);
	const shapes = $derived(s?.hint?.shapes ?? []);
	const shape = $derived(shapes[selectedShape] ?? null);
	const style = $derived(shape ? styleOf(shape, width, height) : null);
	const stepProofs = $derived(proofs.filter((p) => p.step === s?.id));
	const proofFor = (i: number) => stepProofs.find((p) => p.index === i) ?? null;
	const badChecks = $derived(stepProofs.filter((p) => p.verdict === 'passesUntouched' || p.verdict === 'failsReference').length);

	function setHintText(text: string) {
		const st = lesson.task.steps[step];
		if (!text.trim() && !st.hint?.shapes?.length && !st.hint?.target) st.hint = undefined;
		else st.hint = { ...(st.hint ?? { text: '' }), text };
	}

	function setStyle(patch: ShapeStyle) {
		const st = lesson.task.steps[step];
		const list = st.hint?.shapes;
		if (!list || !list[selectedShape]) return;
		const current = list[selectedShape];
		const merged = cleanStyle({ ...styleOf(current, width, height), ...patch }, width, height);
		const next = { ...current } as Shape;
		if (merged) next.style = merged;
		else delete next.style;
		list[selectedShape] = next;
	}

	function removeShape(i: number) {
		lesson.task.steps[step].hint?.shapes?.splice(i, 1);
		onselectshape(-1);
	}

	function setCheck(i: number, check: Check) {
		lesson.task.steps[step].checks[i] = check;
	}

	function addCheck(kind: CheckKind['kind']) {
		lesson.task.steps[step].checks.push(defaultCheck(kind));
	}

	const COLORS = ['#BC3823', '#2F5D8C', '#E8893B', '#16161A'];
</script>

{#if s}
	<div class="panel">
		<div class="head">
			<h2>{s.text || 'Новый шаг'}</h2>
			<p class="where">Шаг {step + 1} из {lesson.task.steps.length}</p>
		</div>
		<div class="tabs" role="tablist">
			<button role="tab" aria-selected={tab === 'text'} onclick={() => (tab = 'text')}>Текст</button>
			<button role="tab" aria-selected={tab === 'hint'} onclick={() => (tab = 'hint')}>Подсказка</button>
			<button role="tab" aria-selected={tab === 'checks'} onclick={() => (tab = 'checks')}>
				Проверки {s.checks.length}{#if badChecks}<span class="warn-dot" aria-label="есть проблемы"></span>{/if}
			</button>
		</div>

		{#if tab === 'text'}
			<label class="field">Что сделать ученику <textarea rows="3" bind:value={lesson.task.steps[step].text} placeholder="Создайте новый слой для тени"></textarea></label>
			<p class="note">Шаг без проверок засчитывается, когда ученик нажмёт «Готово».</p>
			<div class="row">
				<button class="quiet" onclick={() => onmovestep(-1)} disabled={step === 0}>Выше</button>
				<button class="quiet" onclick={() => onmovestep(1)} disabled={step === lesson.task.steps.length - 1}>Ниже</button>
				<span class="grow"></span>
				<button class="quiet danger" onclick={onremovestep}><Icon name="trash" size={14} /> Удалить шаг</button>
			</div>
		{:else if tab === 'hint'}
			<label class="field">Текст подсказки
				<textarea rows="2" value={s.hint?.text ?? ''} oninput={(e) => setHintText(e.currentTarget.value)} placeholder="Мягкой кистью проведите тень по линиям"></textarea>
			</label>
			<div class="field">
				<span>Куда нажать</span>
				<div class="row">
					<button class="target" onclick={onmap}>
						<Icon name="target" size={16} />
						{s.hint?.target ? targetLabel(s.hint.target) : 'Не выбрано'}
					</button>
					<button class="quiet" onclick={onmap}>На карте</button>
				</div>
			</div>
			<div class="field">
				<span class="row-between">Линии на холсте ученика <b>{shapes.length}</b></span>
				{#each shapes as sh, i (i)}
					<div class="ghost-row" class:on={selectedShape === i} role="button" tabindex="0" onclick={() => onselectshape(i)} onkeydown={(e) => e.key === 'Enter' && onselectshape(i)}>
						<span class="seal" class:blue={selectedShape === i}>{i + 1}</span>
						<span class="grow">
							<span class="name">{SHAPE_LABELS[sh.kind]}</span>
							<span class="meta">{sh.style?.dashed ? 'пунктир' : 'сплошная'}{sh.kind === 'path' ? ` · ${sh.points.length} точек` : ''}</span>
						</span>
						<button class="icon" onclick={(e) => (e.stopPropagation(), removeShape(i))} aria-label="Удалить линию {i + 1}"><Icon name="trash" size={14} /></button>
					</div>
				{:else}
					<p class="note">Нарисуйте на холсте: линия от руки, прямая, стрелка, овал или прямоугольник. Ученик увидит их полупрозрачными на слое «FUDE · подсказка».</p>
				{/each}
			</div>
			{#if shape && style}
				<div class="box">
					<p class="box-title">Линия {selectedShape + 1} — как увидит ученик</p>
					<div class="prop">
						<span>Цвет</span>
						<span class="swatches">
							{#each COLORS as c (c)}
								<button class="swatch" class:on={style.color.toUpperCase() === c} style:background={c} onclick={() => setStyle({ color: c })} aria-label="Цвет {c}"></button>
							{/each}
						</span>
					</div>
					<label class="prop">Непрозрачность
						<span class="slider"><input type="range" min="10" max="100" step="5" value={Math.round(style.opacity * 100)} oninput={(e) => setStyle({ opacity: Number(e.currentTarget.value) / 100 })} /><b>{Math.round(style.opacity * 100)}%</b></span>
					</label>
					<label class="prop">Толщина
						<span class="slider"><input type="range" min="1" max="40" step="1" value={style.width} oninput={(e) => setStyle({ width: Number(e.currentTarget.value) })} /><b>{style.width} px</b></span>
					</label>
					<label class="prop inline"><input type="checkbox" checked={style.dashed} onchange={(e) => setStyle({ dashed: e.currentTarget.checked, ...(e.currentTarget.checked ? { reveal: false } : {}) })} /> Пунктир</label>
					<label class="prop inline"><input type="checkbox" checked={style.reveal} disabled={style.dashed} onchange={(e) => setStyle({ reveal: e.currentTarget.checked })} /> Прорисовкой — линия появляется постепенно, как от руки</label>
				</div>
			{/if}
			<button class="wide" onclick={onhintpreview} disabled={!s.hint}><Icon name="play" size={14} /> Показать подсказку в PhotoCraft</button>
		{:else}
			{#if stepProofs.some((p) => p.canvas)}
				{@const works = stepProofs.filter((p) => p.verdict === 'works').length}
				<div class="live">
					<p class="box-title row-between">Живая проверка <span class="meta">{works} из {stepProofs.filter((p) => p.canvas).length} работают</span></p>
					<div class="thumbs">
						{#each [['Эталон', referenceImage, 'onReference'], ['Пустое задание', taskImage, 'onTask']] as [title, img, side] (title)}
							<figure>
								<div class="thumb" style:aspect-ratio="{width} / {height}" style:background>
									{#if img}<img src={img as string} alt={title as string} />{:else if title === 'Эталон'}<span class="noref">нет эталона</span>{/if}
									{#each marks as m (m.key)}
										<span class="mark" style:left="{m.region[0] * 100}%" style:top="{m.region[1] * 100}%" style:width="{m.region[2] * 100}%" style:height="{m.region[3] * 100}%" style:border-color={m.color}></span>
									{/each}
								</div>
								<figcaption>
									{title}
									{#each stepProofs.filter((p) => p.canvas) as p (p.index)}
										{@const o = side === 'onReference' ? p.onReference : p.onTask}
										{#if o}
											{@const good = side === 'onReference' ? o.passed : !o.passed}
											<span class:good class:bad={!good} aria-label={good ? 'как надо' : 'не так'}><Icon name={good ? 'check' : 'alert'} size={10} /></span>
										{/if}
									{/each}
								</figcaption>
							</figure>
						{/each}
					</div>
					<p class="note">Хорошая проверка проходит на эталоне и не проходит на нетронутом задании.</p>
				</div>
			{/if}
			{#each s.checks as check, i (i)}
				{@const l = letters(i)}
				<CheckCard
					{check}
					letters={l.letters}
					colors={l.colors}
					proof={proofFor(i)}
					selected={l.keys.includes(selectedRegion ?? '')}
					onchange={(c) => setCheck(i, c)}
					onremove={() => lesson.task.steps[step].checks.splice(i, 1)}
					onfocus={() => l.keys[0] && onfocusregion(l.keys[0])}
				/>
			{:else}
				<p class="note">Без проверок шаг засчитывается кнопкой «Готово». Растяните область на холсте инструментом «Область проверки» — появится проверка «Закрашена область».</p>
			{/each}
			<label class="field">Добавить проверку
				<select value="" onchange={(e) => (e.currentTarget.value && addCheck(e.currentTarget.value as CheckKind['kind']), (e.currentTarget.value = ''))}>
					<option value="" disabled>Выберите вид…</option>
					{#each CHECK_KINDS as k (k.kind)}<option value={k.kind}>{k.label}</option>{/each}
				</select>
			</label>
		{/if}
	</div>
{/if}

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.where {
		margin: 0;
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
	}

	h2 {
		margin: 2px 0 0;
		font: 700 18px/1.3 var(--font-ui);
		color: var(--sumi-text);
		text-wrap: balance;
	}

	.tabs {
		display: flex;
		gap: 2px;
		padding: 3px;
		border-radius: 8px;
		background: var(--sumi-bg);
	}

	.tabs button {
		flex: 1;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 6px 8px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--sumi-muted);
		font: 600 13px var(--font-ui);
		cursor: pointer;
	}

	.tabs button[aria-selected='true'] {
		background: var(--sumi-line);
		color: var(--sumi-text);
	}

	.warn-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--kaki-500);
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
	}

	textarea,
	select {
		padding: 10px 12px;
		border: 1px solid var(--sumi-line);
		border-radius: 8px;
		background: var(--sumi-bg);
		color: var(--sumi-text);
		font: 13px/1.5 var(--font-ui);
		resize: vertical;
	}

	.note {
		margin: 0;
		color: var(--sumi-muted);
		font: 12px/1.5 var(--font-ui);
	}

	.row {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.row-between {
		display: flex;
		justify-content: space-between;
	}

	.grow {
		flex: 1;
		min-width: 0;
	}

	.quiet,
	.target,
	.wide {
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

	.quiet:disabled,
	.wide:disabled {
		opacity: 0.45;
		cursor: default;
	}

	.danger {
		color: var(--shu-100);
	}

	.target {
		flex: 1;
		background: var(--sumi-bg);
	}

	.wide {
		justify-content: center;
		background: transparent;
	}

	.ghost-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 8px;
		border: 1px solid transparent;
		border-radius: 8px;
		cursor: pointer;
	}

	.ghost-row:hover {
		background: var(--sumi-raised);
	}

	.ghost-row.on {
		border-color: var(--ai-300);
		background: var(--sumi-raised);
	}

	.seal {
		display: grid;
		place-items: center;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--shu-500);
		color: #fff;
		font: 700 11px var(--font-ui);
	}

	.seal.blue {
		background: var(--ai-500);
	}

	.name {
		display: block;
		color: var(--sumi-text);
		font: 600 13px var(--font-ui);
	}

	.meta {
		display: block;
		color: var(--sumi-muted);
		font: 500 12px var(--font-ui);
	}

	.icon {
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--sumi-muted);
		cursor: pointer;
	}

	.box {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px;
		border: 1px solid var(--sumi-line);
		border-radius: 8px;
		background: var(--sumi-bg);
	}

	.box-title {
		margin: 0;
		color: var(--sumi-text);
		font: 600 13px var(--font-ui);
	}

	.prop {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
	}

	.prop.inline {
		justify-content: flex-start;
		color: var(--sumi-text);
	}

	.swatches {
		display: flex;
		gap: 6px;
	}

	.swatch {
		width: 18px;
		height: 18px;
		border-radius: 50%;
		border: 2px solid transparent;
		cursor: pointer;
	}

	.swatch.on {
		border-color: #fff;
	}

	.live {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px;
		border: 1px solid var(--sumi-line);
		border-radius: 8px;
		background: var(--sumi-bg);
	}

	.thumbs {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
	}

	figure {
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.thumb {
		position: relative;
		overflow: hidden;
		border-radius: 6px;
	}

	.thumb img {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}

	.noref {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		color: var(--ink-500);
		font: 600 11px var(--font-ui);
	}

	.mark {
		position: absolute;
		border: 1.5px dashed;
	}

	figcaption {
		display: flex;
		gap: 4px;
		align-items: center;
		color: var(--sumi-text);
		font: 600 12px var(--font-ui);
	}

	figcaption span {
		display: grid;
		place-items: center;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		font-size: 10px;
	}

	figcaption .good {
		background: var(--ai-900);
		color: var(--ai-100);
	}

	figcaption .bad {
		background: var(--kaki-500);
		color: var(--ink-900);
	}

	.slider {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.slider input {
		width: 110px;
		accent-color: var(--shu-500);
	}

	.slider b {
		min-width: 40px;
		color: var(--sumi-text);
		font: 500 12px var(--font-mono);
		text-align: right;
	}
</style>
