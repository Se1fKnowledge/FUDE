<script lang="ts">
	import { onMount } from 'svelte';
	import { api, errorText, type Home } from '#lib/api.ts';
	import Enso from '#lib/components/Enso.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import LessonTile from '#lib/components/LessonTile.svelte';
	import Mountains from '#lib/components/Mountains.svelte';
	import { session } from '#lib/session.svelte.ts';
	import { count, timeOf } from '#lib/format.ts';

	let home = $state<Home | null>(null);
	let error = $state('');

	onMount(async () => {
		try {
			home = await api.home();
		} catch (e) {
			error = errorText(e);
		}
	});

	// Reload once the first-run name is saved.
	$effect(() => {
		if (session.boot && !session.boot.firstRun && home && !home.name) api.home().then((h) => (home = h));
	});

	const STAGES: Record<string, string> = { guide: 'видеогайд', theory: 'теория', practice: 'практика в редакторе' };
	const stageIndex = (s: string | null) => (s === 'practice' ? 3 : s === 'theory' ? 2 : 1);
	const left = $derived(home ? Math.max(0, home.goal - home.doneToday) : 0);
</script>

<div class="page reveal">
	{#if error}
		<p class="muted">{error}</p>
	{:else if home}
		<header class="page-head">
			<div>
				<h1 class="display-l">{home.greeting}{home.name ? `, ${home.name}` : ''}</h1>
				<p>
					{home.date} ·
					{#if left === 0}цель дня выполнена{:else if home.firstDay}начнём с первого урока{:else}до цели дня {left === 1 ? 'осталось одно задание' : `осталось ${count(left, ['задание', 'задания', 'заданий'])}`}{/if}
				</p>
			</div>
		</header>

		<section class="top">
			{#if home.current}
				{@const c = home.current}
				<article class="hero card">
					<div class="text">
						<h2 class="display-l title">{c.title}</h2>
						<p class="muted body-s">{c.categoryTitle} · урок {c.number} из {c.of}</p>
						<p class="summary">
							{#if home.firstDay}Первый урок: посмотрите гайд, прочитайте теорию и попробуйте в редакторе.{:else}{c.summary}{/if}
						</p>
						<div class="stage">
							<div class="track"><i style="width: {(stageIndex(c.stage) / 3) * 100}%"></i></div>
							<span class="body-s muted">Этап {stageIndex(c.stage)} из 3 · {STAGES[c.stage ?? 'guide']}</span>
						</div>
						<div class="actions">
							{#if c.stage === 'practice'}
								<a class="btn btn-accent" href="/practice/{c.id}">Продолжить практику</a>
							{:else}
								<a class="btn btn-accent" href="/lesson/{c.id}">{c.stage ? 'Продолжить урок' : 'Начать урок'}</a>
							{/if}
							<a class="btn btn-quiet" href="/lesson/{c.id}?tab=theory">Теория</a>
						</div>
					</div>
					<div class="art" aria-hidden="true">
						<div class="mist"><Mountains width={520} height={150} seed={c.number + 2} /></div>
						<div class="ring"><Enso size={230} width={20} seed={c.number + 4} /></div>
						<span class="sphere"></span>
						<span class="sun"></span>
					</div>
				</article>
			{:else}
				<article class="hero card done">
					<div class="text">
						<h2 class="display-l title">Все уроки пройдены</h2>
						<p class="summary">Создайте свой урок в мастерской или импортируйте чужой из файла.</p>
						<div class="actions"><a class="btn btn-accent" href="/studio">В мастерскую</a></div>
					</div>
				</article>
			{/if}

			<article class="today card">
				<header>
					<h2 class="h1">Сегодня</h2>
					<span class="body-s muted">{Math.min(home.doneToday, home.goal)} из {home.goal} · цель дня</span>
				</header>
				{#if home.today.length === 0}
					<p class="empty muted">Пока ничего. Первое задание дня — самое важное: оно запускает серию.</p>
				{:else}
					<ul>
						{#each home.today.slice(0, 4) as item (item.at + item.title)}
							<li>
								<span class="dot" class:ok={item.kind === 'pass'}>
									{#if item.kind === 'pass'}<Icon name="check" size={14} stroke={2.4} />{:else if item.kind === 'achievement'}<Icon name="sparkle" size={14} />{/if}
								</span>
								<span>
									<span class="item">{item.title}</span>
									<span class="body-s muted">{timeOf(item.at)}{item.detail ? ` · ${item.detail}` : ''}</span>
								</span>
							</li>
						{/each}
					</ul>
				{/if}
				<div class="goal">
					<div class="track"><i style="width: {Math.min(100, (home.doneToday / home.goal) * 100)}%"></i></div>
					<span class="body-s">{Math.round(Math.min(1, home.doneToday / home.goal) * 100)}%</span>
				</div>
			</article>
		</section>

		{#if home.recommended.length}
			<section class="next">
				<header>
					<h2 class="h1">Рекомендуем дальше</h2>
					<span class="body-s muted">следующие открытые уроки в каждой категории</span>
					<a class="all" href="/lessons">Все уроки <Icon name="arrow" size={18} /></a>
				</header>
				<div class="tiles">
					{#each home.recommended as card (card.id)}
						<LessonTile {card} />
					{/each}
				</div>
			</section>
		{/if}
	{/if}
</div>

<style>
	.top {
		display: grid;
		grid-template-columns: minmax(0, 1.75fr) minmax(300px, 1fr);
		gap: 20px;
	}

	.hero {
		position: relative;
		display: grid;
		grid-template-columns: minmax(0, 1fr) 300px;
		min-height: 380px;
		overflow: hidden;
	}

	.text {
		position: relative;
		z-index: 1;
		display: grid;
		align-content: start;
		gap: 12px;
		padding: 40px;
	}

	.title {
		max-width: 12ch;
	}

	.summary {
		max-width: 44ch;
		margin-top: 8px;
	}

	.stage {
		display: flex;
		align-items: center;
		gap: 14px;
		margin-top: 10px;
		white-space: nowrap;
	}

	.stage .track {
		flex: 0 1 220px;
	}

	.actions {
		display: flex;
		gap: 10px;
		margin-top: 18px;
	}

	.art {
		position: relative;
	}

	.mist {
		position: absolute;
		right: 0;
		bottom: 0;
		width: 520px;
		height: 150px;
	}

	.ring {
		position: absolute;
		right: 40px;
		top: 60px;
	}

	.sphere {
		position: absolute;
		right: 92px;
		top: 112px;
		width: 128px;
		height: 128px;
		border-radius: 50%;
		background: radial-gradient(circle at 34% 30%, #dce7f1 0, #8fb2d6 22%, #2f5d8c 60%, #16304f 100%);
		box-shadow: 30px 40px 30px -18px rgb(22 22 26 / 30%);
	}

	.sun {
		position: absolute;
		right: 250px;
		top: 70px;
		width: 30px;
		height: 30px;
		border-radius: 50%;
		background: var(--shu-500);
	}

	.today {
		display: grid;
		grid-template-rows: auto 1fr auto;
		gap: 18px;
		padding: 28px;
	}

	.today header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
	}

	ul {
		display: grid;
		align-content: start;
		gap: 16px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	li {
		display: flex;
		gap: 12px;
	}

	li > span:last-child {
		display: grid;
	}

	.item {
		font-weight: 600;
		font-size: 14px;
	}

	.dot {
		display: grid;
		place-items: center;
		flex: none;
		width: 22px;
		height: 22px;
		margin-top: 1px;
		border-radius: 50%;
		box-shadow: inset 0 0 0 1.5px var(--ink-300);
		color: var(--kin-700);
	}

	.dot.ok {
		background: var(--ai-500);
		box-shadow: none;
		color: #fff;
	}

	.empty {
		font-size: 14px;
	}

	.goal {
		display: grid;
		grid-template-columns: 1fr auto;
		align-items: center;
		gap: 12px;
	}

	.next {
		margin-top: 44px;
	}

	.next header {
		display: flex;
		align-items: baseline;
		gap: 14px;
		margin-bottom: 18px;
	}

	.all {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-left: auto;
		color: var(--ink-900);
		font-weight: 600;
		font-size: 14px;
	}

	.tiles {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 20px;
	}

	@media (max-width: 1280px) {
		.top {
			grid-template-columns: 1fr;
		}
	}
</style>
