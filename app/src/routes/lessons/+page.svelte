<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, errorText, type Catalog, type CategoryView, type LessonCard } from '#lib/api.ts';
	import Difficulty from '#lib/components/Difficulty.svelte';
	import Hanko from '#lib/components/Hanko.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import { count, duration, two } from '#lib/format.ts';

	let catalog = $state<Catalog | null>(null);
	let error = $state('');
	let selected = $state('');
	let filter = $state<'all' | 'progress' | 'open' | 'passed'>('all');

	onMount(async () => {
		try {
			catalog = await api.catalog();
			const current = catalog.categories.find((c) => c.lessons.some((l) => l.status === 'progress' || l.status === 'open'));
			selected = page.url.searchParams.get('c') ?? current?.id ?? catalog.categories[0]?.id ?? '';
		} catch (e) {
			error = errorText(e);
		}
	});

	const query = $derived((page.url.searchParams.get('q') ?? '').trim().toLowerCase());
	const category = $derived(catalog?.categories.find((c) => c.id === selected) ?? null);
	const lessons = $derived.by(() => {
		if (!catalog) return [];
		const base: LessonCard[] = query
			? catalog.categories.flatMap((c) => c.lessons).filter((l) => `${l.title} ${l.summary} ${l.categoryTitle}`.toLowerCase().includes(query))
			: (category?.lessons ?? []);
		return base.filter((l) => filter === 'all' || l.status === filter || (filter === 'open' && l.status === 'locked'));
	});

	const tint = (c: CategoryView | LessonCard) => ({ ai: 'var(--ai-100)', shu: 'var(--shu-100)', kaki: 'var(--kaki-100)', kin: 'var(--kin-100)' })[c.accent];
	const ink = (c: CategoryView | LessonCard) => ({ ai: 'var(--ai-500)', shu: 'var(--shu-500)', kaki: 'var(--kaki-500)', kin: 'var(--kin-500)' })[c.accent];
	const afterNumber = (l: LessonCard) => {
		const all = catalog?.categories.flatMap((c) => c.lessons) ?? [];
		const req = all.find((x) => x.title === l.after);
		return req ? (req.category === l.category ? two(req.number) : `«${req.title}»`) : '';
	};
	const LABELS = ['', 'Первый шаг', 'Лёгкий', 'Средний', 'Сложный', 'Мастер'];
	const filters = [
		['all', 'Все'],
		['progress', 'В процессе'],
		['open', 'Не начатые'],
		['passed', 'Пройденные']
	] as const;
</script>

<div class="page reveal">
	<header class="page-head">
		<div>
			<h1 class="display-l">Уроки</h1>
			{#if catalog}
				<p>
					{count(catalog.total, ['урок', 'урока', 'уроков'])} в {count(catalog.categories.length, ['категории', 'категориях', 'категориях'])} · пройдено {catalog.passed}
				</p>
			{/if}
		</div>
	</header>

	{#if error}
		<p class="muted">{error}</p>
	{:else if catalog}
		<div class="layout">
			<nav class="categories card" aria-label="Категории">
				<div class="head">
					<span class="caps muted">Категории</span>
					<span class="body-s muted">Пройдено {catalog.passed} из {catalog.total}</span>
				</div>
				{#each catalog.categories as c (c.id)}
					<button class="cat" class:active={c.id === selected && !query} onclick={() => (selected = c.id)} aria-pressed={c.id === selected}>
						<span class="glyph kanji" style="background: {tint(c)}; color: {ink(c)}">{c.kanji ?? '筆'}</span>
						<span class="info">
							<span class="label">{c.title}{#if c.user}<span class="mine">своя</span>{/if}</span>
							<span class="body-s muted">{count(c.lessons.length, ['урок', 'урока', 'уроков'])} · {duration(c.minutes)}</span>
						</span>
						<span class="ratio">
							<span class="body-s">{c.passed} / {c.lessons.length}</span>
							<span class="track"><i style="width: {(c.passed / Math.max(1, c.lessons.length)) * 100}%; background: {ink(c)}"></i></span>
						</span>
					</button>
				{/each}
			</nav>

			<section class="list">
				{#if query}
					<div class="banner search">
						<h2 class="h1">Поиск: «{page.url.searchParams.get('q')}»</h2>
						<p>{count(lessons.length, ['урок найден', 'урока найдено', 'уроков найдено'])}</p>
					</div>
				{:else if category}
					<div class="banner">
						<span class="big kanji" aria-hidden="true">{category.kanji ?? '筆'}</span>
						<h2 class="display-l">{category.title}</h2>
						<p>{category.description}</p>
						<div class="facts">
							<span><Icon name="book" size={17} /> {count(category.lessons.length, ['урок', 'урока', 'уроков'])}</span>
							<span><Icon name="clock" size={17} /> {duration(category.minutes)}</span>
							<span><Icon name="chart" size={17} /> {LABELS[category.difficulty[0]]} → {LABELS[category.difficulty[1]]}</span>
							<span class="progress">
								<span class="bar"><i style="width: {(category.passed / Math.max(1, category.lessons.length)) * 100}%"></i></span>
								{category.passed} из {category.lessons.length}
							</span>
						</div>
					</div>
				{/if}

				<div class="filters" role="group" aria-label="Фильтр уроков">
					{#each filters as [key, label] (key)}
						<button class="chip" aria-pressed={filter === key} onclick={() => (filter = key)}>{label}</button>
					{/each}
				</div>

				<ol class="lessons card">
					{#each lessons as l (l.id)}
						<li class:current={l.status === 'progress'} class:locked={l.status === 'locked'}>
							<span class="num" class:done={l.status === 'passed'}>{two(l.number)}</span>
							<a class="what" href={l.status === 'locked' ? undefined : `/lesson/${l.id}`} aria-disabled={l.status === 'locked'}>
								<span class="h3 title">{l.title}</span>
								<span class="body-s muted">{l.minutes} мин · {l.summary}</span>
							</a>
							<Difficulty level={l.difficulty} label={l.difficultyLabel} />
							<span class="act">
								{#if l.status === 'passed'}
									<a class="passed" href="/lesson/{l.id}">Пройден <Hanko kanji="済" size={26} /></a>
								{:else if l.status === 'progress'}
									<a class="btn btn-accent btn-s" href={l.stage === 'practice' ? `/practice/${l.id}` : `/lesson/${l.id}`}>Продолжить</a>
								{:else if l.status === 'open'}
									<a class="btn btn-quiet btn-s" href="/lesson/{l.id}">Начать</a>
								{:else}
									<span class="lock body-s" title="Сначала пройдите «{l.after}»"><Icon name="lock" size={16} /> После урока {afterNumber(l)}</span>
								{/if}
							</span>
						</li>
					{:else}
						<li class="empty muted">Здесь пока пусто.</li>
					{/each}
				</ol>
			</section>
		</div>
	{/if}
</div>

<style>
	.layout {
		display: grid;
		grid-template-columns: 340px minmax(0, 1fr);
		gap: 24px;
		align-items: start;
	}

	.categories {
		display: grid;
		gap: 2px;
		padding: 18px 12px;
	}

	.head {
		display: flex;
		justify-content: space-between;
		padding: 4px 12px 12px;
	}

	.cat {
		display: grid;
		grid-template-columns: 46px 1fr 60px;
		align-items: center;
		gap: 14px;
		padding: 10px 12px;
		border: 0;
		border-radius: 12px;
		background: transparent;
		text-align: left;
		cursor: pointer;
	}

	.cat:hover {
		background: rgb(22 22 26 / 4%);
	}

	.cat.active {
		background: var(--paper-sunken);
		box-shadow: inset 3px 0 0 var(--shu-500);
	}

	.glyph {
		display: grid;
		place-items: center;
		width: 46px;
		height: 46px;
		border-radius: 10px;
		font-size: 26px;
	}

	.info {
		display: grid;
		min-width: 0;
	}

	.mine {
		margin-left: 8px;
		padding: 1px 7px;
		border-radius: 99px;
		background: var(--kin-100);
		color: var(--kin-700);
		font-size: 11px;
	}

	.ratio {
		display: grid;
		gap: 6px;
		justify-items: end;
	}

	.ratio .track {
		width: 100%;
		height: 4px;
	}

	.banner {
		position: relative;
		overflow: hidden;
		padding: 34px 36px 28px;
		border-radius: var(--radius-card);
		background: var(--ai-900);
		color: var(--paper-raised);
	}

	.banner.search {
		background: var(--ink-900);
	}

	.banner p {
		max-width: 56ch;
		margin-top: 12px;
		color: rgb(251 248 242 / 82%);
	}

	.big {
		position: absolute;
		right: 24px;
		top: -14px;
		font-size: 220px;
		line-height: 1;
		color: rgb(143 178 214 / 22%);
		pointer-events: none;
	}

	.facts {
		position: relative;
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 26px;
		margin-top: 26px;
		font-size: 13px;
	}

	.facts span {
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}

	.progress .bar {
		width: 60px;
		height: 5px;
		border-radius: 99px;
		background: rgb(255 255 255 / 20%);
		overflow: hidden;
	}

	.progress .bar i {
		display: block;
		height: 100%;
		background: var(--kaki-500);
	}

	.filters {
		display: flex;
		gap: 10px;
		margin: 22px 0 16px;
	}

	.lessons {
		margin: 0;
		padding: 10px;
		list-style: none;
	}

	li {
		display: grid;
		grid-template-columns: 44px minmax(0, 1fr) auto 150px;
		align-items: center;
		gap: 20px;
		padding: 16px 18px;
		border-radius: 12px;
	}

	li.current {
		background: rgb(47 93 140 / 10%);
	}

	li.locked .what,
	li.locked .num {
		opacity: 0.6;
	}

	.num {
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: 50%;
		box-shadow: inset 0 0 0 2px var(--ink-900);
		font-family: var(--font-number);
		font-weight: 700;
		font-size: 17px;
	}

	.num.done {
		box-shadow: inset 0 0 0 2px var(--ai-500);
		color: var(--ai-700);
	}

	.what {
		display: grid;
		gap: 2px;
		min-width: 0;
		color: inherit;
	}

	.what .body-s {
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.what:not([aria-disabled='true']):hover .title {
		text-decoration: underline;
		text-underline-offset: 3px;
	}

	.act {
		display: flex;
		justify-content: flex-end;
	}

	.passed {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		color: var(--ai-500);
		font-weight: 600;
		font-size: 14px;
	}

	.lock {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--ink-500);
		text-align: right;
	}

	.empty {
		display: block;
		padding: 30px;
	}
</style>
