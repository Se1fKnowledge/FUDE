<script lang="ts">
	import { t } from '#lib/i18n.svelte.ts';
	import { onDestroy, onMount, untrack } from 'svelte';
	import type { LessonCard, Shape } from '#lib/api.ts';
	import Icon from '../components/Icon.svelte';
	import LessonTile from '../components/LessonTile.svelte';
	import { GHOST_COLOR, styleOf, svgPath } from './geometry.ts';
	import { CHAPTERS, STEPS, columnPlace, rememberIndex, savedIndex, type TourPlace, type TourSnap } from './tour.ts';

	// The lesson-editor tour over the studio: everything but one control is dimmed and takes no
	// clicks; a card beside the control says what it is and sets a small task; when the task
	// changes something a student will see, a second panel shows it at once.
	let {
		dir,
		snap,
		card,
		brief,
		steps,
		stepIndex,
		hintText,
		shapes,
		canvas,
		checkMessage,
		finished = false,
		onplace,
		onexit
	}: {
		dir: string;
		snap: TourSnap;
		card: LessonCard | null;
		brief: string;
		steps: string[];
		stepIndex: number;
		hintText: string;
		shapes: Shape[];
		canvas: { w: number; h: number; background: string; image: string | null };
		checkMessage: string;
		/** The author is back from «Глазами ученика»: only the closing card is shown. */
		finished?: boolean;
		/** Opens the part of the studio a step works in. */
		onplace: (place: TourPlace) => void;
		onexit: () => void;
	} = $props();

	let index = $state(untrack(() => savedIndex(dir)));
	let start = $state<TourSnap>(untrack(() => ({ ...snap })));
	// A done step stays done while it is shown, even if the author changes the field again.
	let latched = $state(false);
	let hole = $state<{ x: number; y: number; w: number; h: number } | null>(null);
	let view = $state({ w: 1280, h: 800 });
	let colW = $state(360);
	let colH = $state(260);
	let toast = $state(false);
	let toastTimer: ReturnType<typeof setTimeout> | null = null;
	let frame = 0;

	const step = $derived(STEPS[index]);
	const last = $derived(index === STEPS.length - 1);
	const place = $derived(columnPlace(view, hole, { w: colW, h: colH }));

	$effect(() => {
		if (!latched && step.done(snap, start)) latched = true;
	});

	function go(to: number) {
		index = Math.min(Math.max(to, 0), STEPS.length - 1);
		start = { ...snap };
		latched = false;
		rememberIndex(dir, index);
		open();
	}

	// Bring the step's control on screen: open its part of the studio, then scroll to it.
	function open() {
		const place = STEPS[index].place;
		if (place) onplace(place);
		requestAnimationFrame(() => document.querySelector(STEPS[index].target)?.scrollIntoView({ block: 'nearest', inline: 'nearest' }));
	}

	function outside() {
		toast = true;
		if (toastTimer) clearTimeout(toastTimer);
		toastTimer = setTimeout(() => (toast = false), 2200);
	}

	// The lit control moves when panels open or the window resizes: follow it every frame.
	function track() {
		const el = document.querySelector(step.target);
		const r = el?.getBoundingClientRect();
		const next = r && r.width > 0 && r.height > 0 ? { x: Math.round(r.left) - 4, y: Math.round(r.top) - 4, w: Math.round(r.width) + 8, h: Math.round(r.height) + 8 } : null;
		if (JSON.stringify(next) !== JSON.stringify(hole)) hole = next;
		if (view.w !== window.innerWidth || view.h !== window.innerHeight) view = { w: window.innerWidth, h: window.innerHeight };
		frame = requestAnimationFrame(track);
	}

	// Alt+→ is «Дальше» (or «Пропустить»), Alt+← is «Назад»: the card can be driven without the mouse.
	function onkeydown(e: KeyboardEvent) {
		if (finished || !e.altKey || e.ctrlKey || e.metaKey) return;
		if (e.key === 'ArrowRight' && !last) {
			e.preventDefault();
			go(index + 1);
		} else if (e.key === 'ArrowLeft' && index > 0) {
			e.preventDefault();
			go(index - 1);
		}
	}

	onMount(() => {
		if (finished) return;
		rememberIndex(dir, index);
		open();
		frame = requestAnimationFrame(track);
	});
	onDestroy(() => {
		cancelAnimationFrame(frame);
		if (toastTimer) clearTimeout(toastTimer);
	});
</script>

<svelte:window {onkeydown} />

{#if finished}
	<div class="tour">
		<div class="column end">
			<section class="coach">
				<header>
					<h2>{t('Урок собран')}</h2>
					<button class="x" onclick={onexit} aria-label={t('Закрыть')} title={t('Закрыть')}><Icon name="close" size={16} /></button>
				</header>
				<p class="body">{t('Вы прошли путь автора: карточка, теория, задание, подсказка и проверка — и увидели урок глазами ученика. Чего уроку ещё не хватает, показывает список «Готовность урока» слева.')}</p>
				<footer>
					<a class="plain" href="/studio/reference">{t('Справочник по редактору')}</a>
					<span class="grow"></span>
					<button class="next" onclick={onexit}>{t('Готово')}</button>
				</footer>
			</section>
		</div>
	</div>
{:else}
<div class="tour">
	{#if hole}
		<!-- Four shades around the lit control: they dim the studio and take the clicks. -->
		<div class="shade" style="left: 0; top: 0; width: 100%; height: {Math.max(0, hole.y)}px" onpointerdown={outside} role="presentation"></div>
		<div class="shade" style="left: 0; top: {hole.y + hole.h}px; width: 100%; bottom: 0" onpointerdown={outside} role="presentation"></div>
		<div class="shade" style="left: 0; top: {hole.y}px; width: {Math.max(0, hole.x)}px; height: {hole.h}px" onpointerdown={outside} role="presentation"></div>
		<div class="shade" style="left: {hole.x + hole.w}px; top: {hole.y}px; right: 0; height: {hole.h}px" onpointerdown={outside} role="presentation"></div>
		<div class="ring" style="left: {hole.x}px; top: {hole.y}px; width: {hole.w}px; height: {hole.h}px"></div>
	{/if}

	<div class="column" style="left: {place.x}px; top: {place.y}px" bind:clientWidth={colW} bind:clientHeight={colH}>
		<section class="coach" aria-live="polite">
			<header>
				<h2>{step.title}</h2>
				<button class="x" onclick={onexit} aria-label={t('Выйти из путеводителя')} title={t('Выйти из путеводителя')}><Icon name="close" size={16} /></button>
			</header>
			<p class="body">{step.body}</p>
			<div class="task" class:done={latched}>
				<span class="dot">{#if latched}<Icon name="check" size={12} stroke={2.8} />{/if}</span>
				<div>
					<p class="what">{step.task}</p>
					<p class="status">{latched ? t('Сделано') : hole ? t('Ждём действия') : t('Этого места сейчас не видно — шаг можно пропустить')}</p>
				</div>
			</div>
			<footer>
				<span class="where">{t('{n} из {of}', { n: index + 1, of: STEPS.length })} · {CHAPTERS[step.chapter]}</span>
				<span class="grow"></span>
				{#if index > 0}<button class="plain" onclick={() => go(index - 1)} title="Alt+←">{t('Назад')}</button>{/if}
				{#if latched && !last}
					<button class="next" onclick={() => go(index + 1)} title="Alt+→">{t('Дальше')}</button>
				{:else if !last}
					<button class="plain" onclick={() => go(index + 1)} title="Alt+→">{t('Пропустить')}</button>
				{/if}
			</footer>
		</section>

		{#if step.view !== 'none'}
			<section class="student" aria-label={t('Так увидит ученик')}>
				<p class="student-title"><Icon name="eye" size={15} /> {t('Так увидит ученик')}</p>
				{#if step.view === 'card' && card}
					<div class="tile" inert><LessonTile {card} /></div>
				{:else if step.view === 'row' && card}
					<div class="paper row">
						<span class="num">{String(card.number).padStart(2, '0')}</span>
						<div class="row-text">
							<p class="row-title">{card.title || t('Без названия')}</p>
							<p class="row-sum">{t('{n} мин', { n: card.minutes })} · {card.summary || t('Краткое описание появится здесь')}</p>
						</div>
					</div>
					<p class="note dark">{t('Строка урока в разделе «Уроки».')}</p>
				{:else if step.view === 'task'}
					<div class="panel">
						<p class="brief">{brief}</p>
						<ol>
							{#each steps as text, i (i)}
								<li class:current={i === stepIndex}><span class="mark"></span><span>{text || t('Новый шаг')}</span></li>
							{/each}
						</ol>
					</div>
				{:else if step.view === 'hint'}
					<div class="panel">
						<div class="hint">
							<span class="n">{stepIndex + 1}</span>
							<div>
								<p class="what">{steps[stepIndex] || t('Новый шаг')}</p>
								<p class="hint-text">{hintText || t('Текст подсказки появится здесь')}</p>
							</div>
						</div>
						<div class="sheet" style="aspect-ratio: {canvas.w} / {canvas.h}; background-color: {canvas.background}; {canvas.image ? `background-image: url('${canvas.image}')` : ''}">
							<svg viewBox="0 0 {canvas.w} {canvas.h}" preserveAspectRatio="none" aria-hidden="true">
								{#each shapes as s, i (i)}
									{@const st = styleOf(s, canvas.w, canvas.h)}
									<path d={svgPath(s, canvas.w, canvas.h)} fill="none" stroke={st.color || GHOST_COLOR} stroke-width={st.width} stroke-opacity={st.opacity} stroke-linecap="round" stroke-linejoin="round" stroke-dasharray={st.dashed ? `${st.width * 2.5} ${st.width * 2}` : undefined} />
								{/each}
							</svg>
						</div>
						<p class="note">{shapes.length ? t('Фигура появится на слое «FUDE · подсказка», когда ученик откроет подсказку.') : t('Нарисованные линии появятся на этом листе.')}</p>
					</div>
				{:else if step.view === 'check'}
					<div class="panel">
						<div class="almost">
							<p class="what">{t('Почти получилось')}</p>
							<p>{checkMessage || t('Пока область пуста, шаг не засчитан: ученику нужно закрасить её.')}</p>
						</div>
						<p class="note">{t('Это сообщение ученик видит, пока шаг не выполнен. Когда область закрашена, шаг отмечается сам.')}</p>
					</div>
				{/if}
			</section>
		{/if}
	</div>

	{#if toast}
		<p class="toast" role="status"><Icon name="lock" size={14} /> {t('Сейчас работает только подсвеченная область')}</p>
	{/if}
</div>
{/if}

<style>
	.tour {
		position: fixed;
		inset: 0;
		z-index: 60;
		pointer-events: none;
		font-family: var(--font-ui, inherit);
	}

	.shade {
		position: fixed;
		background: rgb(6 7 11 / 70%);
		pointer-events: auto;
	}

	.ring {
		position: fixed;
		border-radius: 10px;
		box-shadow:
			0 0 0 2px #8fb2d6,
			0 0 0 8px rgb(143 178 214 / 22%);
		transition:
			left 180ms var(--ease-out),
			top 180ms var(--ease-out),
			width 180ms var(--ease-out),
			height 180ms var(--ease-out);
	}

	.column {
		position: fixed;
		display: grid;
		gap: 12px;
		width: 360px;
		pointer-events: auto;
		transition:
			left 220ms var(--ease-out),
			top 220ms var(--ease-out);
	}

	.column.end {
		right: 24px;
		bottom: 24px;
	}

	.coach {
		display: grid;
		gap: 12px;
		padding: 18px 20px;
		border-radius: 16px;
		background: #1f4470;
		box-shadow: 0 12px 40px rgb(0 0 0 / 45%);
		color: #fff;
	}

	.coach header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
	}

	h2 {
		font-size: 19px;
		font-weight: 700;
		line-height: 1.3;
	}

	.x {
		display: grid;
		place-items: center;
		width: 26px;
		height: 26px;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: #dce7f1;
		cursor: pointer;
	}

	.x:hover {
		background: rgb(255 255 255 / 12%);
	}

	.body {
		font-size: 13px;
		line-height: 1.5;
		color: #dce7f1;
	}

	.task {
		display: grid;
		grid-template-columns: 18px minmax(0, 1fr);
		gap: 10px;
		padding: 12px 14px;
		border-radius: 12px;
		background: #16304f;
	}

	.dot {
		display: grid;
		place-items: center;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		border: 2px solid #e8893b;
		color: #16304f;
	}

	.task.done .dot {
		border-color: #8fb2d6;
		background: #8fb2d6;
	}

	.what {
		font-size: 13px;
		font-weight: 600;
		line-height: 1.4;
	}

	.status {
		margin-top: 2px;
		font-size: 12px;
		color: #e8893b;
	}

	.task.done .status {
		color: #8fb2d6;
	}

	footer {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.where {
		font-size: 12px;
		color: #8fb2d6;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.grow {
		flex: 1;
	}

	.plain {
		border: 0;
		background: transparent;
		text-decoration: none;
		font-size: 13px;
		font-weight: 600;
		color: #dce7f1;
		cursor: pointer;
	}

	.plain:hover {
		color: #fff;
	}

	.next {
		height: 36px;
		padding: 0 18px;
		border: 0;
		border-radius: 10px;
		background: #bc3823;
		font-size: 13px;
		font-weight: 600;
		color: #fff;
		cursor: pointer;
	}

	.student {
		display: grid;
		gap: 10px;
		padding: 14px 16px 16px;
		border-radius: 16px;
		background: #f3eee4;
		box-shadow: 0 12px 40px rgb(0 0 0 / 45%);
		color: #16161a;
	}

	.student-title {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 12px;
		font-weight: 700;
		color: #625e57;
	}

	.tile {
		border-radius: 16px;
		overflow: hidden;
	}

	.paper {
		padding: 14px 16px;
		border-radius: 12px;
		background: #fbf8f2;
	}

	.row {
		display: grid;
		grid-template-columns: 40px minmax(0, 1fr);
		align-items: center;
		gap: 14px;
	}

	.num {
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: 50%;
		box-shadow: inset 0 0 0 2px #16161a;
		font-family: var(--font-number, inherit);
		font-weight: 700;
		font-size: 17px;
	}

	.row-title {
		font-size: 16px;
		font-weight: 700;
		line-height: 1.3;
	}

	.row-sum {
		margin-top: 2px;
		font-size: 13px;
		line-height: 1.45;
		color: #625e57;
	}

	.note.dark {
		color: #625e57;
	}

	.panel {
		display: grid;
		gap: 10px;
		padding: 14px;
		border-radius: 12px;
		background: #1a1c21;
		color: #e8e4dc;
	}

	.brief {
		font-size: 14px;
		font-weight: 700;
		line-height: 1.35;
	}

	ol {
		display: grid;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	li {
		display: grid;
		grid-template-columns: 18px minmax(0, 1fr);
		gap: 10px;
		font-size: 13px;
		color: #a3a7af;
	}

	li.current {
		color: #fff;
		font-weight: 600;
	}

	.mark {
		width: 16px;
		height: 16px;
		margin-top: 1px;
		border-radius: 50%;
		border: 1.5px solid #4a4f5a;
	}

	li.current .mark {
		border: 4px solid #8fb2d6;
	}

	.hint {
		display: grid;
		grid-template-columns: 24px minmax(0, 1fr);
		gap: 10px;
		padding: 12px;
		border-radius: 12px;
		background: #1f4470;
		color: #fff;
	}

	.n {
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: #fff;
		font-size: 12px;
		font-weight: 700;
		color: #1f4470;
	}

	.hint-text {
		margin-top: 2px;
		font-size: 13px;
		color: #dce7f1;
	}

	.sheet {
		width: 100%;
		border-radius: 6px;
		background-size: 100% 100%;
		overflow: hidden;
	}

	.sheet svg {
		display: block;
		width: 100%;
		height: 100%;
	}

	.note {
		font-size: 12px;
		line-height: 1.45;
		color: #a3a7af;
	}

	.almost {
		display: grid;
		gap: 4px;
		padding: 12px 14px;
		border-radius: 12px;
		background: rgb(232 137 59 / 14%);
		box-shadow: inset 0 0 0 1px rgb(232 137 59 / 35%);
		font-size: 13px;
		color: #f3c9a0;
	}

	.almost .what {
		color: #e8893b;
	}

	.toast {
		position: fixed;
		left: 50%;
		bottom: 40px;
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 9px 14px;
		border-radius: 999px;
		border: 1px solid #3a3e48;
		background: #23262d;
		font-size: 12px;
		font-weight: 600;
		color: #e8e4dc;
	}
</style>
