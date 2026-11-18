<script lang="ts">
	import { t } from '#lib/i18n.svelte.ts';
	import { onDestroy } from 'svelte';
	import { page } from '$app/state';
	import { api, errorText, type TourView } from '#lib/api.ts';
	import Hanko from '#lib/components/Hanko.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import { editorSlot } from '#lib/editor-slot.ts';
	import { count } from '#lib/format.ts';

	const chapter = $derived((page.params as Record<string, string | undefined>).chapter ?? '');

	let view = $state<TourView | null>(null);
	let phase = $state<'starting' | 'running' | 'finished' | 'failed'>('starting');
	let failure = $state('');
	let embedded = $state(true);
	let timer: ReturnType<typeof setInterval> | null = null;
	let busy = false;

	const skipped = $derived(view?.steps.filter((s) => s.state === 'skipped').length ?? 0);
	const spent = $derived(view ? t('{m} мин {s} с', { m: Math.floor(view.seconds / 60), s: view.seconds % 60 }) : '');

	function stop() {
		if (timer) clearInterval(timer);
		timer = null;
	}

	async function poll() {
		if (busy) return;
		busy = true;
		try {
			const next = await api.tourPoll();
			view = next;
			if (next.finished) {
				stop();
				phase = 'finished';
			}
		} catch (e) {
			stop();
			failure = errorText(e);
			phase = 'failed';
		} finally {
			busy = false;
		}
	}

	async function start(id: string) {
		stop();
		phase = 'starting';
		failure = '';
		view = null;
		try {
			// The lit place and the card are drawn by the editor itself, so the tour works in
			// either layout; inside the FUDE window is the default where the system allows it.
			const layout = await api.editorLayout(true);
			embedded = layout.supported && layout.embedded;
			view = await api.tourStart(id);
			phase = 'running';
			timer = setInterval(poll, 350);
		} catch (e) {
			failure = errorText(e);
			phase = 'failed';
		}
	}

	// A chapter starts when the page opens and again when «Следующая глава» changes the address.
	$effect(() => {
		if (chapter) start(chapter);
	});

	onDestroy(() => {
		stop();
		api.tourStop().catch(() => {});
	});
</script>

<div class="tour sumi-scope">
	<header>
		<a class="round" href="/editor" aria-label={t('К списку глав')}><Icon name="back" /></a>
		<Hanko kanji="筆" size={30} />
		<div class="title">
			<h1>{t('Путеводитель по редактору')}</h1>
			<p>{view?.title ?? ''}</p>
		</div>
		{#if view}
			<ol class="progress" aria-label={t('Шаги главы')}>
				{#each view.steps as s, i (s.id)}
					<li class={s.state} class:now={phase === 'running' && i === view.index} title={s.title}></li>
				{/each}
			</ol>
			<span class="where body-s">{phase === 'finished' ? t('Глава пройдена') : t('Шаг {n} из {of}', { n: view.index + 1, of: view.total })}</span>
		{/if}
		<a class="exit" href="/editor">{t('Выйти из путеводителя')}</a>
	</header>

	{#if phase === 'failed'}
		<section class="center reveal">
			<div class="box">
				<Icon name="alert" size={30} />
				<h2 class="h1">{t('Редактор не запустился')}</h2>
				<p class="muted">{failure}</p>
				<div class="row">
					<button class="btn btn-accent" onclick={() => start(chapter)}><Icon name="refresh" size={18} /> {t('Попробовать снова')}</button>
					<a class="btn btn-quiet" href="/editor">{t('К списку глав')}</a>
				</div>
			</div>
		</section>
	{:else if phase === 'finished' && view}
		<section class="center done reveal">
			{#if view.picture}<img class="sheet" src={view.picture} alt="" />{/if}
			<div class="summary">
				<div class="summary-head">
					<Hanko kanji="済" size={56} stamp />
					<div>
						<h2 class="h1">{t('Глава «{title}» пройдена', { title: view.title })}</h2>
						<p class="body-s meta">
							{count(view.total, ['шаг', 'шага', 'шагов'])}{#if skipped}, {t('пропущено: {n}', { n: skipped })}{/if} · {spent}
						</p>
					</div>
				</div>
				<h3 class="label">{t('Теперь вы умеете')}</h3>
				<ul class="learned">
					{#each view.steps as s (s.id)}
						<li class={s.state}>
							<Icon name={s.state === 'done' ? 'check' : 'skip'} size={18} stroke={s.state === 'done' ? 2.2 : 1.8} />
							<span>{s.learned}{#if s.state !== 'done'} <i>{t('— шаг пропущен')}</i>{/if}</span>
							{#if s.key && s.state === 'done'}<span class="kbd">{s.key}</span>{/if}
						</li>
					{/each}
				</ul>
				<p class="body-s meta">{t('Опыт за путеводитель не начисляется: это знакомство с редактором, а не урок.')}</p>
				<div class="row">
					{#if view.next}
						<a class="btn btn-accent" href="/editor/tour/{view.next.id}">{t('Следующая глава: {title}', { title: view.next.title })}</a>
						<a class="btn btn-quiet" href="/editor">{t('К списку глав')}</a>
					{:else}
						<a class="btn btn-accent" href="/editor">{t('К списку глав')}</a>
					{/if}
				</div>
			</div>
		</section>
	{:else if embedded}
		<section class="slot" use:editorSlot aria-label={t('Редактор PhotoCraft')}>
			{#if phase === 'starting'}<p class="loading">{t('Открываем PhotoCraft и готовим главу…')}</p>{/if}
		</section>
	{:else}
		<section class="center">
			<div class="box">
				<Icon name="layers" size={26} />
				<p>{phase === 'starting' ? t('Открываем PhotoCraft и готовим главу…') : t('Редактор открыт отдельным окном. Подсветка и карточка шага — там.')}</p>
			</div>
		</section>
	{/if}
</div>

<style>
	.tour {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		height: 100vh;
		background: var(--sumi-bg);
		color: var(--sumi-text);
	}

	header {
		display: grid;
		grid-template-columns: auto auto minmax(0, 1fr) auto auto auto;
		align-items: center;
		gap: 14px;
		height: 60px;
		padding: 0 18px;
		border-bottom: 1px solid var(--sumi-line);
	}

	.round {
		display: grid;
		place-items: center;
		width: 34px;
		height: 34px;
		border-radius: 10px;
		background: var(--sumi-raised);
		color: var(--sumi-text);
	}

	h1 {
		font-size: 15px;
		font-weight: 600;
		line-height: 1.3;
	}

	.title p {
		font-size: 13px;
		color: var(--sumi-muted);
	}

	.progress {
		display: flex;
		gap: 6px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.progress li {
		width: 34px;
		height: 5px;
		border-radius: 3px;
		background: var(--sumi-line);
	}

	.progress li.done {
		background: var(--ai-500);
	}

	.progress li.skipped {
		background: color-mix(in srgb, var(--ai-500) 40%, var(--sumi-line));
	}

	.progress li.now {
		background: var(--ai-300);
	}

	.where {
		color: var(--sumi-muted);
		min-width: 96px;
	}

	.exit {
		display: inline-flex;
		align-items: center;
		height: 38px;
		padding: 0 18px;
		border: 1px solid var(--sumi-line);
		border-radius: 12px;
		font-size: 13px;
		font-weight: 600;
		color: var(--sumi-text);
	}

	.exit:hover {
		background: var(--sumi-raised);
	}

	.slot {
		position: relative;
		min-height: 0;
	}

	.loading {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		color: var(--sumi-muted);
	}

	.center {
		position: relative;
		display: grid;
		place-items: center;
		min-height: 0;
		padding: 24px;
		overflow: auto;
	}

	.box {
		display: grid;
		justify-items: center;
		gap: 12px;
		max-width: 520px;
		text-align: center;
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
	}

	/* The sheet the student left, dim behind the chapter's summary. */
	.sheet {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: contain;
		padding: 40px;
		opacity: 0.22;
	}

	.summary {
		position: relative;
		display: grid;
		gap: 16px;
		width: min(620px, 100%);
		padding: 32px 36px;
		border-radius: 20px;
		background: var(--paper-raised);
		box-shadow: var(--shadow-float);
		color: var(--ink-900);
	}

	.summary-head {
		display: flex;
		align-items: center;
		gap: 18px;
	}

	.meta {
		color: var(--ink-500);
	}

	.learned {
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.learned li {
		display: grid;
		grid-template-columns: 18px minmax(0, 1fr) auto;
		align-items: center;
		gap: 12px;
		padding: 8px 0;
		border-top: 1px solid var(--paper-line);
		color: var(--ink-900);
	}

	.learned li :global(svg) {
		color: var(--ai-500);
	}

	.learned li.skipped,
	.learned li.pending {
		color: var(--ink-500);
	}

	.learned li.skipped :global(svg),
	.learned li.pending :global(svg) {
		color: var(--ink-300);
	}

	.learned i {
		font-style: normal;
	}
</style>
