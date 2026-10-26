<script lang="ts">
	import { onMount } from 'svelte';
	import { api, errorText, type Activity, type Day } from '#lib/api.ts';
	import Icon from '#lib/components/Icon.svelte';
	import { count, duration, longDate, monthShort, timeOf } from '#lib/format.ts';

	let data = $state<Activity | null>(null);
	let error = $state('');
	let hover = $state<Day | null>(null);

	onMount(async () => {
		try {
			data = await api.activity();
		} catch (e) {
			error = errorText(e);
		}
	});

	// The calendar: 53 columns of weeks, Monday on top.
	const weeks = $derived(data ? Array.from({ length: 53 }, (_, w) => data!.days.slice(w * 7, w * 7 + 7)) : []);
	// A month name over the first week of each month, skipped when it would crowd the last one.
	const months = $derived.by(() => {
		let last = -10;
		return weeks.map((w, i) => {
			const first = w[0]?.date ?? '';
			const prev = i > 0 ? (weeks[i - 1][0]?.date ?? '') : '';
			const starts = i === 0 || monthShort(first) !== monthShort(prev);
			if (!starts || i - last < 3) return '';
			last = i;
			return monthShort(first);
		});
	});
	const level = (d: Day) => (d.tasks === 0 ? 0 : d.tasks === 1 ? 1 : d.tasks === 2 ? 2 : d.tasks <= 4 ? 3 : 4);
	const future = (d: Day) => !!data && d.date > data.today;
	const maxWeek = $derived(data ? Math.max(data.goalMinutes, ...data.week.map((d) => d.minutes), 1) : 1);
	const DAYS = ['Пн', 'Вт', 'Ср', 'Чт', 'Пт', 'Сб', 'Вс'];
	const KIND: Record<string, 'check' | 'sparkle' | 'bulb' | 'play' | 'download' | 'pencil'> = {
		pass: 'check',
		achievement: 'sparkle',
		start: 'play',
		import: 'download',
		author: 'pencil'
	};
</script>

<div class="page reveal">
	<header class="page-head">
		<div>
			<h1 class="display-l">Активность</h1>
			{#if data}<p>{count(data.tasks, ['задание', 'задания', 'заданий'])} за последний год</p>{/if}
		</div>
	</header>

	{#if error}
		<p class="muted">{error}</p>
	{:else if data}
		<section class="year card">
			<header>
				<h2 class="h2">Год в мазках туши</h2>
				<span class="body-s muted">каждый квадрат — день, цвет — сколько заданий вы выполнили</span>
				<span class="legend body-s muted">
					Меньше
					{#each [0, 1, 2, 3, 4] as l (l)}<span class="cell l{l}"></span>{/each}
					Больше
					<span class="sep"></span>
					<span class="gold"></span> без подсказок
				</span>
			</header>
			<div class="heat">
				<div class="labels body-s muted">
					<span></span>
					{#each DAYS as d, i (d)}<span>{i % 2 === 0 ? d : ''}</span>{/each}
				</div>
				{#each weeks as w, i (i)}
					<div class="col">
						<span class="month body-s muted">{months[i]}</span>
						{#each w as d (d.date)}
							<span
								class="cell l{level(d)}"
								class:future={future(d)}
								class:today={d.date === data.today}
								role="img"
								aria-label="{longDate(d.date)}: {count(d.tasks, ['задание', 'задания', 'заданий'])}"
								onmouseenter={() => (hover = d)}
								onmouseleave={() => (hover = null)}
							>
								{#if d.tasks > 0 && d.noHints}<i></i>{/if}
							</span>
						{/each}
					</div>
				{/each}
			</div>
			<p class="tip body-s" aria-live="polite">
				{#if hover}
					{longDate(hover.date)} · {count(hover.tasks, ['задание', 'задания', 'заданий'])}{hover.minutes ? ` · ${duration(hover.minutes)}` : ''}
				{:else}
					Наведите на день, чтобы увидеть подробности
				{/if}
			</p>
		</section>

		<section class="stats">
			<div class="stat card"><span class="ico kaki"><Icon name="flame" /></span><span><span class="n number">{data.streak.current}</span> <span class="unit">{count(data.streak.current, ['день', 'дня', 'дней']).replace(/^\d+ /, '')}</span><span class="body-s muted">Текущая серия</span></span></div>
			<div class="stat card"><span class="ico shu"><Icon name="trophy" /></span><span><span class="n number">{data.streak.best}</span> <span class="unit">{count(data.streak.best, ['день', 'дня', 'дней']).replace(/^\d+ /, '')}</span><span class="body-s muted">Лучшая серия{data.streak.bestEnd ? ` · ${monthShort(data.streak.bestEnd)}` : ''}</span></span></div>
			<div class="stat card"><span class="ico ai"><Icon name="calendar" /></span><span><span class="n number">{data.activeDays}</span> <span class="unit">{count(data.activeDays, ['день', 'дня', 'дней']).replace(/^\d+ /, '')}</span><span class="body-s muted">Активных дней за год</span></span></div>
			<div class="stat card"><span class="ico"><Icon name="clock" /></span><span>{#if data.minutes < 60}<span class="n number">{data.minutes}</span> <span class="unit">мин</span>{:else}<span class="n number">{Math.floor(data.minutes / 60)}</span> <span class="unit">ч {data.minutes % 60} мин</span>{/if}<span class="body-s muted">Практики в редакторе</span></span></div>
		</section>

		<section class="lower">
			<article class="week card">
				<header>
					<h2 class="h2">Эта неделя</h2>
					<span class="body-s muted">{duration(data.week.reduce((s, d) => s + d.minutes, 0))} · цель {data.goalMinutes} мин в день</span>
				</header>
				<div class="bars">
					<span class="goal" style="bottom: {(data.goalMinutes / maxWeek) * 180 + 26}px"></span>
					{#each data.week as d, i (d.date)}
						<div class="bar">
							<span class="v body-s">{d.minutes ? `${d.minutes} мин` : ''}</span>
							<span class="ink" class:today={d.date === data.today} style="height: {Math.max(d.minutes ? 8 : 2, (d.minutes / maxWeek) * 180)}px"></span>
							<span class="body-s">{DAYS[i]}</span>
						</div>
					{/each}
				</div>
			</article>

			<article class="journal card">
				<header><h2 class="h2">Журнал</h2></header>
				{#if data.journal.length === 0}
					<p class="muted body-s">Здесь появятся пройденные уроки, достижения и импорт уроков.</p>
				{:else}
					<ul>
						{#each data.journal.slice(0, 6) as j (j.at + j.title)}
							<li>
								<span class="jico {j.kind}"><Icon name={KIND[j.kind] ?? 'sparkle'} size={18} /></span>
								<span class="what">
									<span class="label">{j.title}</span>
									<span class="body-s muted">{j.detail}</span>
								</span>
								<span class="mono muted">{longDate(j.at)}, {timeOf(j.at)}</span>
							</li>
						{/each}
					</ul>
				{/if}
			</article>
		</section>
	{/if}
</div>

<style>
	.year {
		padding: 26px 28px 18px;
	}

	.year header {
		display: flex;
		align-items: baseline;
		gap: 12px;
		margin-bottom: 18px;
	}

	.legend {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-left: auto;
	}

	.sep {
		width: 1px;
		height: 14px;
		margin: 0 6px;
		background: var(--paper-line);
	}

	.gold {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--kin-500);
	}

	.heat {
		display: flex;
		gap: 3px;
		overflow-x: auto;
		padding-bottom: 4px;
	}

	.labels,
	.col {
		display: grid;
		grid-template-rows: 20px repeat(7, 15px);
		gap: 3px;
	}

	.labels {
		width: 26px;
		line-height: 15px;
	}

	.month {
		white-space: nowrap;
		width: 0;
		overflow: visible;
	}

	.cell {
		position: relative;
		display: grid;
		place-items: center;
		width: 15px;
		height: 15px;
		border-radius: 3px;
		background: var(--paper-sunken);
	}

	.cell i {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--kin-500);
	}

	.l1 {
		background: var(--ai-100);
	}

	.l2 {
		background: var(--ai-300);
	}

	.l3 {
		background: var(--ai-500);
	}

	.l4 {
		background: var(--ai-900);
	}

	.future {
		opacity: 0.35;
	}

	.today {
		outline: 2px solid var(--ink-900);
		outline-offset: 1px;
	}

	.tip {
		margin-top: 10px;
		color: var(--ink-500);
	}

	.stats {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 18px;
		margin: 20px 0;
	}

	.stat {
		display: flex;
		gap: 16px;
		padding: 20px 22px;
	}

	.stat > span:last-child {
		display: block;
	}

	.n {
		font-size: 36px;
		line-height: 1;
	}

	.unit {
		font-size: 14px;
		color: var(--ink-700);
	}

	.stat .body-s {
		display: block;
		margin-top: 6px;
	}

	.ico {
		display: grid;
		place-items: center;
		flex: none;
		width: 44px;
		height: 44px;
		border-radius: 12px;
		background: var(--paper-sunken);
	}

	.ico.kaki {
		background: var(--kaki-100);
		color: var(--kaki-500);
	}

	.ico.shu {
		background: var(--shu-100);
		color: var(--shu-500);
	}

	.ico.ai {
		background: var(--ai-100);
		color: var(--ai-500);
	}

	.lower {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}

	.week,
	.journal {
		padding: 26px 28px;
	}

	.week header,
	.journal header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin-bottom: 20px;
	}

	.bars {
		position: relative;
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		align-items: end;
		height: 240px;
	}

	.goal {
		position: absolute;
		left: 0;
		right: 0;
		border-top: 1px dashed var(--ink-300);
	}

	.bar {
		display: grid;
		justify-items: center;
		gap: 6px;
	}

	.v {
		color: var(--ink-500);
	}

	/* A vertical brush stroke: rounded loaded top, dry ragged foot. */
	.ink {
		width: 26px;
		border-radius: 13px 13px 6px 6px / 14px 14px 4px 4px;
		background: linear-gradient(var(--ai-500), var(--ai-500) 85%, rgb(47 93 140 / 60%));
		mask: linear-gradient(90deg, #000 70%, rgb(0 0 0 / 70%));
		transition: height 700ms var(--ease-out);
	}

	.ink.today {
		background: var(--ink-900);
	}

	ul {
		display: grid;
		gap: 16px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	li {
		display: grid;
		grid-template-columns: 40px 1fr auto;
		align-items: center;
		gap: 14px;
	}

	.what {
		display: grid;
		min-width: 0;
	}

	.jico {
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: 50%;
		background: var(--paper-sunken);
		color: var(--ink-700);
	}

	.jico.pass {
		background: var(--ai-100);
		color: var(--ai-500);
	}

	.jico.achievement {
		background: var(--kin-100);
		color: var(--kin-700);
	}
</style>
