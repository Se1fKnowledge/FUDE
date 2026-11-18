<script lang="ts">
	import { t } from '#lib/i18n.svelte.ts';
	import { onMount } from 'svelte';
	import { api, errorText, type TourChapter, type TourOverview } from '#lib/api.ts';
	import Brush from '#lib/components/Brush.svelte';
	import Hanko from '#lib/components/Hanko.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import { count } from '#lib/format.ts';

	let overview = $state<TourOverview | null>(null);
	let error = $state('');
	let picked = $state<string | null>(null);

	const chapters = $derived(overview?.chapters ?? []);
	// The chapter shown below the route: the one the student picked, else the one to go on with.
	const chapter = $derived<TourChapter | null>(chapters.find((c) => c.id === (picked ?? overview?.current)) ?? chapters[0] ?? null);
	const currentIndex = $derived(Math.max(0, chapters.findIndex((c) => c.id === overview?.current)));
	// How far the ink line is drawn: up to the chapter to go on with, the whole way when all are done.
	const inked = $derived(!overview || chapters.length === 0 ? 0 : overview.current ? (currentIndex + 0.5) / chapters.length : 1);
	const firstOpen = $derived(chapter ? chapter.steps.findIndex((s) => s.state === 'pending') : -1);

	function stepsLabel(c: TourChapter) {
		return `${count(c.steps.length, ['шаг', 'шага', 'шагов'])} · ${t('{n} мин', { n: c.minutes })}`;
	}

	function action(c: TourChapter) {
		if (c.state === 'done') return t('Пройти заново');
		if (c.state === 'new') return t('Начать главу');
		return t('Продолжить с шага {n}', { n: firstOpen + 1 });
	}

	onMount(async () => {
		try {
			overview = await api.tourOverview();
		} catch (e) {
			error = errorText(e);
		}
	});
</script>

<div class="page reveal">
	<header class="page-head">
		<div>
			<h1 class="display-l">{t('Редактор')}</h1>
			<p>{t('Путеводитель проведёт по окну PhotoCraft шаг за шагом: одна подсвеченная кнопка, одна маленькая задача.')}</p>
		</div>
	</header>

	{#if error}
		<p class="muted">{error}</p>
	{:else if overview && chapter}
		<section class="route card" aria-label={t('Маршрут путеводителя')}>
			<div class="route-head">
				<h2 class="h2">
					{t('Маршрут: {chapters}, {steps}, около {n} минут', {
						chapters: count(chapters.length, ['глава', 'главы', 'глав']),
						steps: count(overview.steps, ['шаг', 'шага', 'шагов']),
						n: overview.minutes
					})}
				</h2>
				<span class="body-s muted">{t('Пройдено глав: {n} из {of}', { n: overview.done, of: chapters.length })}</span>
			</div>
			<div class="line" style="--n: {chapters.length}">
				<div class="ink faded"><Brush width={1000} height={18} seed={4} color="var(--ink-900)" /></div>
				<div class="ink" style="width: {inked * 100}%"><Brush width={1000} height={18} seed={4} color="var(--ink-900)" /></div>
				<ol>
					{#each chapters as c, i (c.id)}
						<li>
							<button class="stop" class:shown={c.id === chapter.id} class:current={c.id === overview.current} aria-pressed={c.id === chapter.id} onclick={() => (picked = c.id)}>
								<span class="mark">
									{#if c.state === 'done'}
										<Hanko kanji="済" size={30} tilt={-4} />
									{:else}
										<span class="num">{i + 1}</span>
									{/if}
								</span>
								<span class="name">{c.short}</span>
								<span class="body-s muted">{stepsLabel(c)}</span>
							</button>
						</li>
					{/each}
				</ol>
			</div>
		</section>

		<div class="detail">
			<section class="chapter card">
				<div class="chapter-head">
					<div>
						<h2 class="h1">{chapter.title}</h2>
						<p class="body-s muted">{t('Глава {n} из {of}', { n: chapters.indexOf(chapter) + 1, of: chapters.length })} · {stepsLabel(chapter)}</p>
					</div>
					<a class="btn btn-accent" href="/editor/tour/{chapter.id}">{action(chapter)}</a>
				</div>
				<p class="summary">{chapter.summary}</p>
				<ol class="steps">
					{#each chapter.steps as s, i (s.id)}
						<li class={s.state} class:now={chapter.state !== 'done' && i === firstOpen}>
							<span class="dot">
								{#if s.state === 'done'}<Icon name="check" size={18} stroke={2.2} />{:else if s.state === 'skipped'}<Icon name="skip" size={16} />{/if}
							</span>
							<span class="title">{s.title}</span>
							{#if s.key}<span class="kbd">{s.key}</span>{/if}
							<span class="state body-s">
								{#if s.state === 'done'}{t('пройден')}{:else if s.state === 'skipped'}{t('пропущен')}{:else if chapter.state !== 'done' && i === firstOpen}{t('сейчас')}{/if}
							</span>
						</li>
					{/each}
				</ol>
			</section>

			<aside class="rules">
				<div class="rule">
					<Icon name="target" size={20} />
					<div>
						<p class="label">{t('Подсвечена одна кнопка')}</p>
						<p class="body-s muted">{t('Пока шаг не сделан, в редакторе нажимается только она.')}</p>
					</div>
				</div>
				<div class="rule">
					<Icon name="task" size={20} />
					<div>
						<p class="label">{t('Маленькая задача на каждом шаге')}</p>
						<p class="body-s muted">{t('Приложение видит, что вы её сделали, и открывает следующий шаг.')}</p>
					</div>
				</div>
				<div class="rule">
					<Icon name="skip" size={20} />
					<div>
						<p class="label">{t('Умеете? Пропустите')}</p>
						<p class="body-s muted">{t('Любой шаг можно пропустить. Опыт за путеводитель не начисляется.')}</p>
					</div>
				</div>
			</aside>
		</div>
	{/if}
</div>

<style>
	.route {
		display: grid;
		gap: 22px;
		padding: 26px 28px;
		margin-bottom: 26px;
	}

	.route-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 16px;
	}

	/* One brush line with a stop per chapter: the corridor the tour walks. */
	.line {
		position: relative;
	}

	.ink {
		position: absolute;
		top: 8px;
		left: 0;
		width: 100%;
		height: 18px;
		overflow: hidden;
		transition: width 700ms var(--ease-ink);
	}

	.ink :global(svg) {
		width: 1000px;
		max-width: none;
	}

	.ink.faded {
		opacity: 0.14;
	}

	ol {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.line ol {
		position: relative;
		display: grid;
		grid-template-columns: repeat(var(--n), minmax(0, 1fr));
	}

	.stop {
		display: grid;
		justify-items: center;
		gap: 4px;
		width: 100%;
		padding: 0 4px 6px;
		border: 0;
		border-radius: 12px;
		background: transparent;
		cursor: pointer;
	}

	.stop:hover .name {
		color: var(--shu-700);
	}

	.mark {
		display: grid;
		place-items: center;
		width: 36px;
		height: 36px;
	}

	.num {
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border-radius: 50%;
		border: 1.5px solid var(--ink-300);
		background: var(--paper-base);
		font-size: 13px;
		font-weight: 600;
		color: var(--ink-500);
	}

	.current .num {
		width: 36px;
		height: 36px;
		border: 3px solid var(--shu-500);
		background: var(--paper-raised);
		color: var(--shu-700);
	}

	.name {
		font-size: 13px;
		font-weight: 600;
		color: var(--ink-900);
	}

	.current .name {
		color: var(--shu-700);
	}

	.shown .name {
		text-decoration: underline;
		text-decoration-thickness: 2px;
		text-underline-offset: 5px;
	}

	.detail {
		display: grid;
		grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
		gap: 24px;
		align-items: start;
	}

	.chapter {
		display: grid;
		gap: 16px;
		padding: 26px 28px;
	}

	.chapter-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
	}

	.summary {
		color: var(--ink-700);
		max-width: 62ch;
	}

	.steps li {
		display: grid;
		grid-template-columns: 20px minmax(0, 1fr) auto 76px;
		align-items: center;
		gap: 12px;
		padding: 9px 0;
		border-top: 1px solid var(--paper-line);
		color: var(--ink-700);
	}

	.dot {
		display: grid;
		place-items: center;
		width: 20px;
		height: 20px;
		color: var(--ai-500);
	}

	.steps li.pending .dot::before {
		content: '';
		width: 13px;
		height: 13px;
		border-radius: 50%;
		border: 1.5px solid var(--ink-300);
	}

	.steps li.now .dot::before {
		border-color: var(--shu-500);
		background: var(--shu-500);
	}

	.steps li.skipped .dot {
		color: var(--ink-300);
	}

	.steps li.done,
	.steps li.skipped {
		color: var(--ink-500);
	}

	.steps li.now .title {
		font-weight: 600;
		color: var(--ink-900);
	}

	.steps .kbd {
		grid-column: 3;
	}

	.state {
		grid-column: 4;
		text-align: right;
		color: var(--ink-500);
	}

	.now .state {
		color: var(--shu-700);
	}

	.rules {
		display: grid;
		gap: 18px;
		padding-top: 6px;
	}

	.rule {
		display: grid;
		grid-template-columns: 20px minmax(0, 1fr);
		gap: 14px;
		color: var(--ai-500);
	}

	.rule .label {
		color: var(--ink-900);
	}
</style>
