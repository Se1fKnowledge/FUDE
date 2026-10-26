<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { api, errorText, type GuideEvent, type LessonView } from '#lib/api.ts';
	import Icon from '#lib/components/Icon.svelte';
	import LessonHeader from '#lib/components/LessonHeader.svelte';
	import Markdown from '#lib/components/Markdown.svelte';
	import { editorSlot, preferEmbedded } from '#lib/editor-slot.ts';
	import { clock, duration } from '#lib/format.ts';

	let view = $state<LessonView | null>(null);
	let error = $state('');
	let tab = $state<'theory' | 'task' | 'quiz'>('theory');
	let mode = $state<'editor' | 'video'>('editor');
	let guide = $state<GuideEvent | null>(null);
	let videoError = $state(false);
	let answers = $state<Record<number, number>>({});
	let unlisten: UnlistenFn | null = null;
	// The guide plays in the editor shown right in the player, unless the student prefers it
	// in its own window.
	let inside = $state(false);

	const id = $derived(page.params.id ?? '');

	onMount(async () => {
		const t = page.url.searchParams.get('tab');
		if (t === 'theory' || t === 'task' || t === 'quiz') tab = t;
		unlisten = await listen<GuideEvent>('guide', (e) => {
			if (e.payload.lesson === id) guide = e.payload;
		});
		api.editorLayout(preferEmbedded()).then((l) => (inside = l.supported && l.embedded)).catch(() => {});
		try {
			view = await api.lesson(id);
			if (!view.guide.length && view.video) mode = 'video';
			api.lessonStage(id, view.card.stage === 'practice' ? 'practice' : 'guide');
		} catch (e) {
			error = errorText(e);
		}
	});

	onDestroy(() => {
		unlisten?.();
		if (playing) api.guideStop();
	});

	const playing = $derived(guide?.state === 'starting' || guide?.state === 'step');
	const editorShown = $derived(inside && !!guide && guide.state !== 'stopped' && guide.state !== 'error');
	const xp = $derived(view ? Math.floor((40 + 20 * view.lesson.difficulty) * 1.5) : 0);
	const elapsed = $derived(view && guide && guide.state !== 'starting' ? view.guide.slice(0, guide.index).reduce((s, g) => s + g.pause, 0) : 0);

	async function play() {
		error = '';
		guide = { lesson: id, state: 'starting', index: 0, total: view?.guide.length ?? 0, caption: 'Открываем редактор…', highlight: null, message: null };
		try {
			await api.guidePlay(id);
		} catch (e) {
			guide = null;
			error = errorText(e);
		}
	}

	async function toPractice() {
		await api.lessonStage(id, 'practice');
		goto(`/practice/${id}`);
	}

	function openTab(t: typeof tab) {
		tab = t;
		if (t === 'theory') api.lessonStage(id, 'theory');
	}
</script>

{#if view}
	<LessonHeader
		title={view.card.title}
		subtitle="{view.card.categoryTitle} · урок {view.card.number} из {view.card.of}"
		stage={1}
		back="/lessons?c={view.card.category}"
		{xp}
	/>

	<div class="lesson reveal">
		<section class="left">
			{#if view.guide.length && view.video}
				<div class="segmented modes">
					<button aria-pressed={mode === 'editor'} onclick={() => (mode = 'editor')}>Гайд в редакторе</button>
					<button aria-pressed={mode === 'video'} onclick={() => (mode = 'video')}>Видео</button>
				</div>
			{/if}

			{#if mode === 'video' && view.video}
				<div class="player">
					{#if videoError}
						<div class="fallback">
							<Icon name="alert" size={28} />
							<p class="h3">Видео не загрузилось</p>
							<p class="body-s">Файл повреждён или в формате, который не умеет встроенный плеер. Теория и задание доступны и без него.</p>
							<button class="btn btn-quiet btn-s" onclick={() => (videoError = false)}><Icon name="refresh" size={16} /> Ещё раз</button>
						</div>
					{:else}
						<!-- svelte-ignore a11y_media_has_caption -->
						<video src={convertFileSrc(view.video)} poster={view.poster ?? undefined} controls onerror={() => (videoError = true)}></video>
					{/if}
				</div>
			{:else if view.guide.length}
				<div class="player stage">
					{#if editorShown}
						<div class="view slot" use:editorSlot aria-label="Редактор PhotoCraft с гайдом">
							<p class="body-s waiting">Открываем редактор…</p>
						</div>
						<div class="caption-row" aria-live="polite">
							{#if guide?.state === 'step'}<span class="kbd">{guide.index + 1}</span>{/if}
							{guide?.caption}
						</div>
					{:else}
					<div class="view">
						{#if view.taskImage}
							<img src={view.taskImage} alt="Холст задания" />
						{:else}
							<div class="blank" style="aspect-ratio: {view.lesson.task.canvas?.width ?? 3} / {view.lesson.task.canvas?.height ?? 2}"></div>
						{/if}
						<div class="overlay">
							{#if guide && guide.state !== 'stopped'}
								<div class="caption" class:error={guide.state === 'error'}>
									{#if guide.state === 'error'}
										<Icon name="alert" size={18} /> {guide.message}
									{:else}
										{#if guide.state === 'step'}<span class="kbd">{guide.index + 1}</span>{/if}
										{guide.caption}
									{/if}
								</div>
							{:else}
								<div class="intro">
									<p class="h2">Гайд рисует в настоящем редакторе</p>
									<p class="body-s">Откроется окно PhotoCraft: смотрите, как кисть выполняет приём, а подписи появятся здесь.</p>
								</div>
							{/if}
						</div>
					</div>
					{/if}
					<div class="controls">
						{#if playing}
							<button class="btn btn-quiet btn-s" onclick={() => api.guideStop()}><Icon name="stop" size={16} /> Стоп</button>
						{:else}
							<button class="btn btn-primary btn-s play" onclick={play}><Icon name="play" size={16} /> {guide?.state === 'done' ? 'Ещё раз' : 'Смотреть гайд'}</button>
						{/if}
						<div class="timeline">
							{#each view.guide as g, i (i)}
								<span class="seg" class:on={guide && (guide.state === 'done' || (guide.state === 'step' && i <= guide.index))} style="flex: {Math.max(1, g.pause)}"></span>
							{/each}
						</div>
						<span class="mono">{clock(elapsed / 1000)} / {clock(view.guideMs / 1000)}</span>
					</div>
				</div>
			{:else}
				<div class="player"><div class="fallback"><p class="h3">У этого урока нет гайда</p><p class="body-s">Начните с теории справа.</p></div></div>
			{/if}

			{#if view.guide.length && mode === 'editor'}
				<ol class="chapters">
					{#each view.guide.slice(0, 8) as g, i (i)}
						<li class:current={guide?.state === 'step' && guide.index === i} class:seen={guide && (guide.state === 'done' || guide.index > i)}>
							<span class="body-s muted">Шаг {i + 1}</span>
							<span class="label">{g.caption}</span>
						</li>
					{/each}
				</ol>
			{/if}

			<article class="summary card">
				<h2 class="h2">Задание</h2>
				<p>{view.lesson.task.brief}</p>
				<ol>
					{#each view.lesson.task.steps as s (s.id)}<li>{s.text}</li>{/each}
				</ol>
			</article>
		</section>

		<aside class="right card">
			<div class="segmented tabs" role="tablist">
				<button role="tab" aria-selected={tab === 'theory'} onclick={() => openTab('theory')}>Теория</button>
				<button role="tab" aria-selected={tab === 'task'} onclick={() => openTab('task')}>Эталон</button>
				{#if view.lesson.quiz?.length}
					<button role="tab" aria-selected={tab === 'quiz'} onclick={() => openTab('quiz')}>Вопросы</button>
				{/if}
			</div>

			<div class="scroll">
				{#if tab === 'theory'}
					<Markdown text={view.theory} />
				{:else if tab === 'task'}
					<h2 class="h1">Что получится</h2>
					<p class="muted gap">Так может выглядеть выполненное задание. Ваш вариант не обязан совпадать — проверка смотрит на главное: свет, тень, форму.</p>
					{#if view.referenceImage}
						<img class="ref" src={view.referenceImage} alt="Эталон задания" />
					{:else if view.taskImage}
						<img class="ref" src={view.taskImage} alt="Картинка задания" />
					{:else}
						<p class="muted">Задание начинается с чистого холста {view.lesson.task.canvas?.width}×{view.lesson.task.canvas?.height}.</p>
					{/if}
					<p class="body-s muted gap">Примерно {duration(view.lesson.minutes)} · {view.card.difficultyLabel}</p>
				{:else}
					<h2 class="h1">Проверьте себя</h2>
					{#each view.lesson.quiz ?? [] as q, qi (qi)}
						<div class="question">
							<p class="label">{q.question}</p>
							<div class="options">
								{#each q.options as o, oi (oi)}
									{@const chosen = answers[qi] === oi}
									<button
										class="option"
										class:right={chosen && oi === q.answer}
										class:wrong={chosen && oi !== q.answer}
										onclick={() => (answers[qi] = oi)}
										aria-pressed={chosen}>{o}</button
									>
								{/each}
							</div>
							{#if answers[qi] !== undefined}
								<p class="body-s verdict" class:ok={answers[qi] === q.answer}>
									{answers[qi] === q.answer ? 'Верно.' : 'Не совсем.'}
									{q.explain ?? ''}
								</p>
							{/if}
						</div>
					{/each}
				{/if}
			</div>

			<button class="btn btn-accent go" onclick={toPractice}>К практике в редакторе</button>
		</aside>
	</div>
{:else if error}
	<div class="page"><p class="muted">{error}</p><a href="/lessons">К урокам</a></div>
{/if}

{#if error && view}
	<div class="toast" role="alert"><Icon name="alert" size={18} /> {error}</div>
{/if}

<style>
	.lesson {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 460px;
		gap: 24px;
		padding: 0 40px 32px;
	}

	.left {
		display: grid;
		align-content: start;
		gap: 16px;
		min-width: 0;
	}

	.modes {
		justify-self: start;
	}

	.player {
		position: relative;
		display: grid;
		border-radius: var(--radius-card);
		background: var(--sumi-bg);
		overflow: hidden;
		aspect-ratio: 16 / 10;
		color: var(--sumi-text);
	}

	video {
		width: 100%;
		height: 100%;
		background: #000;
	}

	.stage {
		grid-template-rows: minmax(0, 1fr) auto;
	}

	.stage:has(.slot) {
		grid-template-rows: minmax(0, 1fr) auto auto;
		aspect-ratio: auto;
		height: calc(100vh - 140px);
	}

	.view {
		position: relative;
		display: grid;
		place-items: center;
		min-height: 0;
		padding: 28px 40px 8px;
	}

	.view img,
	.view .blank {
		max-width: 100%;
		max-height: 100%;
		min-height: 0;
		border-radius: 4px;
		object-fit: contain;
		background: #fff;
	}

	.view .blank {
		width: 100%;
	}

	.overlay {
		position: absolute;
		inset: 28px 40px 8px;
		display: grid;
		place-items: end center;
		padding-bottom: 18px;
		pointer-events: none;
	}

	.caption {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		max-width: 80%;
		padding: 10px 16px;
		border-radius: 12px;
		background: rgb(17 18 22 / 88%);
		color: var(--sumi-text);
		font-weight: 600;
		box-shadow: var(--shadow-float);
		animation: reveal 300ms var(--ease-out);
	}

	.caption .kbd {
		background: var(--shu-500);
		color: #fff;
		box-shadow: none;
	}

	.caption.error {
		background: var(--kaki-100);
		color: var(--kaki-700);
	}

	.intro {
		place-self: center;
		max-width: 420px;
		padding: 20px 24px;
		border-radius: 14px;
		background: rgb(17 18 22 / 82%);
		text-align: center;
	}

	.intro .body-s {
		margin-top: 6px;
		color: var(--sumi-muted);
	}

	.slot {
		background: var(--sumi-bg);
	}

	.waiting {
		color: var(--sumi-muted);
	}

	.caption-row {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 40px;
		padding: 8px 24px 0;
		font-weight: 600;
	}

	.caption-row .kbd {
		background: var(--shu-500);
		color: #fff;
		box-shadow: none;
	}

	.controls {
		display: grid;
		padding: 12px 24px 16px;
		grid-template-columns: auto 1fr auto;
		align-items: center;
		gap: 16px;
	}

	.play {
		background: var(--paper-raised);
		color: var(--ink-900);
	}

	.timeline {
		display: flex;
		gap: 4px;
	}

	.seg {
		height: 4px;
		border-radius: 99px;
		background: var(--sumi-line);
		transition: background 300ms;
	}

	.seg.on {
		background: var(--shu-500);
	}

	.fallback {
		display: grid;
		gap: 8px;
		place-content: center;
		justify-items: center;
		padding: 40px;
		text-align: center;
		color: var(--sumi-muted);
	}

	.fallback .h3 {
		color: var(--sumi-text);
	}

	.chapters {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 10px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.chapters li {
		display: grid;
		gap: 2px;
		padding: 12px 14px;
		border-radius: 12px;
		background: var(--paper-raised);
		box-shadow: var(--shadow-card);
	}

	.chapters li.current {
		background: var(--ink-900);
		color: var(--paper-raised);
	}

	.chapters li.current .muted {
		color: var(--kaki-500);
	}

	.chapters li.seen .muted::after {
		content: ' ✓';
		color: var(--ai-500);
	}

	.summary {
		padding: 22px 26px;
	}

	.summary p {
		margin: 8px 0 10px;
		color: var(--ink-700);
	}

	.summary ol {
		margin: 0;
		padding-left: 20px;
		color: var(--ink-700);
	}

	.summary li::marker {
		color: var(--ai-500);
		font-weight: 700;
	}

	.right {
		position: sticky;
		top: 16px;
		display: grid;
		grid-template-rows: auto 1fr auto;
		gap: 18px;
		height: calc(100vh - 120px);
		padding: 24px;
	}

	.tabs {
		justify-self: start;
	}

	.scroll {
		overflow: auto;
		padding-right: 8px;
	}

	.gap {
		margin: 10px 0 16px;
	}

	.ref {
		width: 100%;
		border-radius: 12px;
		box-shadow: var(--shadow-card);
	}

	.question {
		margin-top: 18px;
		padding: 16px;
		border-radius: 14px;
		background: var(--ai-100);
	}

	.options {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 10px;
	}

	.option {
		padding: 7px 14px;
		border: 0;
		border-radius: 99px;
		background: var(--paper-raised);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
	}

	.option.right {
		background: var(--ai-500);
		color: #fff;
	}

	.option.wrong {
		background: var(--kaki-100);
		color: var(--kaki-700);
	}

	.verdict {
		margin-top: 10px;
		color: var(--kaki-700);
	}

	.verdict.ok {
		color: var(--ai-700);
	}

	.go {
		width: 100%;
		height: 50px;
	}

	.toast {
		position: fixed;
		right: 24px;
		bottom: 24px;
		display: flex;
		align-items: center;
		gap: 10px;
		max-width: 460px;
		padding: 14px 18px;
		border-radius: 12px;
		background: var(--kaki-100);
		color: var(--kaki-700);
		box-shadow: var(--shadow-float);
	}
</style>
