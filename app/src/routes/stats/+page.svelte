<script lang="ts">
	import { onMount } from 'svelte';
	import { api, errorText, type Stats } from '#lib/api.ts';
	import Hanko from '#lib/components/Hanko.svelte';
	import { percent, shortDate, thousands } from '#lib/format.ts';

	type Period = 'week' | 'quarter' | 'all';
	let period = $state<Period>('quarter');
	let data = $state<Stats | null>(null);
	let error = $state('');

	async function load() {
		try {
			data = await api.stats(period);
		} catch (e) {
			error = errorText(e);
		}
	}

	onMount(load);

	const PERIODS: [Period, string, string][] = [
		['week', 'Неделя', 'неделю назад'],
		['quarter', '3 месяца', '3 месяца назад'],
		['all', 'Всё время', 'в начале']
	];
	const ago = $derived(PERIODS.find((p) => p[0] === period)?.[2] ?? '');

	// Radar geometry: six axes around (170, 160).
	const R = 110;
	const point = (i: number, v: number, n: number) => {
		const a = -Math.PI / 2 + (i * 2 * Math.PI) / n;
		return [170 + Math.cos(a) * R * (v / 100), 160 + Math.sin(a) * R * (v / 100)] as const;
	};
	const poly = (values: number[]) => values.map((v, i) => point(i, Math.max(v, 2), values.length).join(',')).join(' ');

	// Weekly experience as a line.
	const chart = $derived.by(() => {
		if (!data) return null;
		const max = Math.max(100, ...data.weekly.map(([, xp]) => xp));
		const step = 420 / Math.max(1, data.weekly.length - 1);
		const pts = data.weekly.map(([, xp], i) => [20 + i * step, 170 - (xp / max) * 150] as const);
		return {
			max,
			line: pts.map((p) => p.join(',')).join(' '),
			area: `20,170 ${pts.map((p) => p.join(',')).join(' ')} ${20 + (pts.length - 1) * step},170`,
			last: pts[pts.length - 1],
			labels: data.weekly.map(([d], i) => ({ x: 20 + i * step, text: shortDate(d) })).filter((_, i, a) => i % Math.ceil(a.length / 6) === 0 || i === a.length - 1)
		};
	});

	const gauge = $derived(data?.independence ?? 0);
	const arc = (share: number) => {
		const a = Math.PI * (1 - share);
		return `M20 110 A90 90 0 0 1 ${110 + 90 * Math.cos(a)} ${110 - 90 * Math.sin(a)}`;
	};
</script>

<div class="page reveal">
	<header class="page-head">
		<div>
			<h1 class="display-l">Статистика</h1>
			<p>Как растёт ваш навык</p>
		</div>
		<div class="segmented" role="group" aria-label="Период">
			{#each PERIODS as [key, label] (key)}
				<button aria-pressed={period === key} onclick={() => ((period = key), load())}>{label}</button>
			{/each}
		</div>
	</header>

	{#if error}
		<p class="muted">{error}</p>
	{:else if data}
		<section class="row">
			<article class="card radar">
				<header>
					<h2 class="h2">Профиль навыка</h2>
					<span class="legend body-s muted"><i class="now"></i> Сейчас <i class="then"></i> {ago}</span>
				</header>
				<svg viewBox="0 0 340 330" role="img" aria-label="Профиль навыка: {data.skills.map((s) => `${s.label} ${Math.round(s.score)}`).join(', ')}">
					{#each [25, 50, 75, 100] as ring (ring)}
						<polygon points={poly(data.skills.map(() => ring))} fill="none" stroke="var(--paper-line)" />
					{/each}
					{#each data.skills as s, i (s.skill)}
						{@const [x, y] = point(i, 100, data.skills.length)}
						<line x1="170" y1="160" x2={x} y2={y} stroke="var(--paper-line)" />
						{@const [lx, ly] = point(i, 128, data.skills.length)}
						<text x={lx} y={ly - 4} text-anchor="middle" class="axis">{s.label}</text>
						<text x={lx} y={ly + 11} text-anchor="middle" class="axis-n">{Math.round(s.score)}</text>
					{/each}
					<polygon points={poly(data.skills.map((s) => s.before))} fill="rgb(173 167 156 / 25%)" stroke="var(--ink-300)" stroke-dasharray="4 3" />
					<polygon points={poly(data.skills.map((s) => s.score))} fill="rgb(47 93 140 / 14%)" stroke="var(--ai-500)" stroke-width="2" />
					{#each data.skills as s, i (s.skill)}
						{@const [x, y] = point(i, Math.max(s.score, 2), data.skills.length)}
						<circle cx={x} cy={y} r="4" fill="var(--paper-raised)" stroke="var(--ai-500)" stroke-width="2" />
					{/each}
				</svg>
			</article>

			<article class="card growth">
				<header>
					<h2 class="h2">Рост по направлениям</h2>
					<span class="body-s muted">{PERIODS.find((p) => p[0] === period)?.[1].toLowerCase()}</span>
				</header>
				{#each [...data.skills].sort((a, b) => b.score - b.before - (a.score - a.before)) as s (s.skill)}
					<div class="skill">
						<span>{s.label}</span>
						<span class="mono muted">ур. {s.level}</span>
						<span class="bar">
							<i class="then" style="width: {s.before}%"></i>
							<i class="now" style="width: {s.score}%"></i>
						</span>
						<span class="pill pill-ai">+{Math.round(s.score - s.before)}</span>
					</div>
				{/each}
				<p class="note body-s">Светлая часть шкалы — уровень {ago}, насыщенная — сейчас. 100 — всё, что дают уроки библиотеки.</p>
			</article>
		</section>

		<section class="row three">
			<article class="card xp">
				<header>
					<h2 class="h2">Опыт по неделям</h2>
					<span class="total number">{thousands(data.totalXp)} XP</span>
				</header>
				{#if chart}
					<svg viewBox="0 0 460 200" role="img" aria-label="Опыт по неделям">
						{#each [0.33, 0.66, 1] as g (g)}
							<line x1="20" x2="440" y1={170 - g * 150} y2={170 - g * 150} stroke="var(--paper-line)" stroke-dasharray="3 4" />
						{/each}
						<polygon points={chart.area} fill="rgb(47 93 140 / 14%)" />
						<polyline points={chart.line} fill="none" stroke="var(--ai-500)" stroke-width="2" />
						<circle cx={chart.last[0]} cy={chart.last[1]} r="5" fill="var(--shu-500)" />
						{#each chart.labels as l (l.x)}
							<text x={l.x} y="192" text-anchor="middle" class="axis-n">{l.text}</text>
						{/each}
					</svg>
				{/if}
				<p class="body-s muted">За период: +{thousands(data.periodXp)} XP · пройдено уроков: {data.passed}</p>
			</article>

			<article class="card gauge">
				<h2 class="h2">Самостоятельность</h2>
				<svg viewBox="0 0 220 130" aria-hidden="true">
					<path d={arc(1)} fill="none" stroke="var(--paper-sunken)" stroke-width="14" stroke-linecap="round" />
					{#if gauge > 0}<path d={arc(gauge)} fill="none" stroke="var(--ai-500)" stroke-width="14" stroke-linecap="round" />{/if}
				</svg>
				<span class="big number">{data.independence === null ? '—' : percent(data.independence)}</span>
				<p class="body-s muted">{data.independence === null ? 'появится после первого задания' : 'заданий выполнено без подсказок'}</p>
				<span class="pill pill-ai">Подсказок за период: {data.hints}</span>
			</article>

			<article class="sensei">
				<header><Hanko kanji="師" size={34} /> <h2 class="h2">Совет сэнсэя</h2></header>
				{#if data.advice}
					<p class="lead">{data.advice.text}</p>
					<p class="body-s">{data.advice.detail}</p>
					{#if data.advice.lesson}<a class="btn btn-accent" href="/lesson/{data.advice.lesson}">Открыть урок</a>{/if}
				{:else}
					<p class="lead">Начните с первого урока.</p>
					<p class="body-s">Советы появятся, когда будет из чего их составить.</p>
					<a class="btn btn-accent" href="/lessons">К урокам</a>
				{/if}
			</article>
		</section>
	{/if}
</div>

<style>
	.row {
		display: grid;
		grid-template-columns: 1fr 1.25fr;
		gap: 20px;
	}

	.row.three {
		grid-template-columns: 1.5fr 1fr 1fr;
		margin-top: 20px;
	}

	.card {
		padding: 26px 28px;
	}

	header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 12px;
	}

	.legend {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.legend i {
		width: 14px;
		height: 3px;
		border-radius: 2px;
		background: var(--ai-500);
	}

	.legend i.then {
		margin-left: 8px;
		background: var(--ink-300);
	}

	.axis {
		font-size: 12px;
		font-weight: 600;
		fill: var(--ink-700);
	}

	.axis-n {
		font-size: 11px;
		fill: var(--ink-500);
	}

	.skill {
		display: grid;
		grid-template-columns: 130px 44px 1fr 48px;
		align-items: center;
		gap: 12px;
		padding: 7px 0;
		font-size: 14px;
	}

	.bar {
		position: relative;
		height: 10px;
		border-radius: 99px;
		background: var(--paper-sunken);
	}

	.bar i {
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		border-radius: inherit;
		transition: width 700ms var(--ease-out);
	}

	.bar .then {
		background: var(--ai-300);
		opacity: 0.6;
	}

	.bar .now {
		background: var(--ai-500);
		mix-blend-mode: multiply;
	}

	.pill {
		justify-content: center;
	}

	.note {
		margin-top: 14px;
		padding: 12px 14px;
		border-radius: 10px;
		background: var(--paper-sunken);
		color: var(--ink-700);
	}

	.total {
		font-size: 34px;
	}

	.xp svg {
		width: 100%;
	}

	.gauge {
		display: grid;
		justify-items: center;
		align-content: start;
		gap: 4px;
		text-align: center;
	}

	.gauge svg {
		width: 220px;
	}

	.big {
		margin-top: -60px;
		font-size: 52px;
		line-height: 1;
	}

	.sensei {
		display: grid;
		align-content: start;
		gap: 14px;
		padding: 26px 28px;
		border-radius: var(--radius-card);
		background: var(--ink-900);
		color: var(--sumi-text);
	}

	.sensei header {
		justify-content: flex-start;
		align-items: center;
		margin: 0;
	}

	.lead {
		font-size: 17px;
	}

	.sensei .body-s {
		color: var(--sumi-muted);
	}

	.sensei .btn {
		justify-self: start;
		margin-top: 10px;
	}
</style>
