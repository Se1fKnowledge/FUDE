<script lang="ts">
	import { onMount } from 'svelte';
	import { api, errorText, type Achievements } from '#lib/api.ts';
	import Enso from '#lib/components/Enso.svelte';
	import Hanko from '#lib/components/Hanko.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import { longDate, timeOf } from '#lib/format.ts';

	let data = $state<Achievements | null>(null);
	let error = $state('');
	let group = $state('Все');

	onMount(async () => {
		try {
			data = await api.achievements();
		} catch (e) {
			error = errorText(e);
		}
	});

	const groups = $derived(data ? ['Все', ...new Set(data.list.map((a) => a.group))] : []);
	const shown = $derived(data ? data.list.filter((a) => group === 'Все' || a.group === group) : []);
	const latest = $derived(data?.list.filter((a) => a.unlocked).sort((a, b) => (b.unlocked ?? '').localeCompare(a.unlocked ?? ''))[0] ?? null);
	const ranks = ['10 кю', '9 кю', '8 кю', '7 кю', '6 кю', '5 кю', '4 кю', '3 кю', '2 кю', '1 кю', '1 дан'];
	const RANK_KANJI = ['十', '九', '八', '七', '六', '五', '四', '三', '二', '一', '段'];
</script>

<div class="page reveal">
	<header class="page-head">
		<div>
			<h1 class="display-l">Достижения</h1>
			{#if data}<p>Получено {data.unlocked} из {data.list.length}</p>{/if}
		</div>
	</header>

	{#if error}
		<p class="muted">{error}</p>
	{:else if data}
		<section class="top">
			<article class="ranks card">
				<header>
					<h2 class="h1">{data.rank.title} — {data.rank.name}</h2>
					{#if data.rank.next !== null}
						<span class="to-next">
							<span class="body-s">До {ranks[data.rank.index + 1]} — {data.rank.next - data.rank.xp} XP</span>
							<span class="track"><i style="width: {((data.rank.xp - data.rank.floor) / (data.rank.next - data.rank.floor)) * 100}%; background: var(--kaki-500)"></i></span>
						</span>
					{/if}
				</header>
				<ol class="path">
					{#each ranks as r, i (r)}
						<li class:passed={i < data.rank.index} class:now={i === data.rank.index}>
							{#if i === data.rank.index}
								<Hanko kanji={RANK_KANJI[i]} size={34} tilt={-4} />
							{:else}
								<span class="dot"></span>
							{/if}
							<span class="body-s">{r}</span>
						</li>
					{/each}
				</ol>
			</article>

			<article class="latest">
				{#if latest}
					<div>
						<h2 class="h1">{latest.title}</h2>
						<p class="body-s">Получено {longDate(latest.unlocked ?? '')} в {timeOf(latest.unlocked ?? '')}: {latest.description.toLowerCase()}.</p>
					</div>
					<div class="seal"><Enso size={130} width={12} color="var(--kin-500)" seed={4} /><span><Hanko kanji={latest.kanji} size={58} /></span></div>
				{:else}
					<div>
						<h2 class="h1">Первая печать впереди</h2>
						<p class="body-s">Пройдите первый урок — и здесь появится «Первый мазок».</p>
					</div>
					<div class="seal"><Enso size={130} width={12} color="var(--sumi-line)" seed={4} /><span><Hanko kanji="一" size={58} tone="faded" /></span></div>
				{/if}
			</article>
		</section>

		<div class="filters" role="group" aria-label="Группы достижений">
			{#each groups as g (g)}
				<button class="chip" aria-pressed={group === g} onclick={() => (group = g)}>
					{g} · {g === 'Все' ? data.list.length : data.list.filter((a) => a.group === g).length}
				</button>
			{/each}
		</div>

		<section class="grid">
			{#each shown as a (a.id)}
				<article class="ach card" class:locked={!a.unlocked}>
					<div class="badge">
						<Enso size={84} width={7} color={a.unlocked ? 'var(--kin-500)' : 'var(--ink-300)'} seed={a.kanji.charCodeAt(0)} />
						<span class="h"><Hanko kanji={a.kanji} size={46} tone={a.unlocked ? 'shu' : 'faded'} /></span>
						{#if !a.unlocked}<span class="lock"><Icon name="lock" size={14} stroke={2.2} /></span>{/if}
					</div>
					<h3 class="h3">{a.title}</h3>
					<p class="body-s muted">{a.description}</p>
					{#if a.unlocked}
						<span class="pill pill-kin">{longDate(a.unlocked)}</span>
					{:else}
						<span class="progress">
							<span class="track"><i style="width: {(a.current / a.target) * 100}%"></i></span>
							<span class="body-s">{a.current} / {a.target}</span>
						</span>
					{/if}
				</article>
			{/each}
		</section>
	{/if}
</div>

<style>
	.top {
		display: grid;
		grid-template-columns: minmax(0, 1.7fr) minmax(320px, 1fr);
		gap: 20px;
	}

	.ranks {
		padding: 30px;
	}

	.ranks header {
		display: flex;
		justify-content: space-between;
		gap: 20px;
	}

	.to-next {
		display: grid;
		gap: 8px;
		width: 180px;
		text-align: right;
	}

	.path {
		position: relative;
		display: grid;
		grid-template-columns: repeat(11, 1fr);
		margin: 34px 0 0;
		padding: 0;
		list-style: none;
	}

	.path::before {
		content: '';
		position: absolute;
		left: 4%;
		right: 4%;
		top: 16px;
		height: 2px;
		background: var(--paper-line);
	}

	.path li {
		position: relative;
		display: grid;
		justify-items: center;
		align-content: start;
		gap: 12px;
		color: var(--ink-500);
	}

	.path li.passed::before {
		content: '';
		position: absolute;
		left: 0;
		right: 0;
		top: 13px;
		height: 8px;
		border-radius: 4px;
		background: var(--ink-900);
	}

	.dot {
		position: relative;
		width: 14px;
		height: 14px;
		margin: 10px 0 10px;
		border-radius: 50%;
		background: var(--paper-raised);
		box-shadow: inset 0 0 0 2px var(--ink-300);
	}

	.passed .dot {
		background: var(--paper-raised);
		box-shadow: inset 0 0 0 3px var(--ink-900);
	}

	.now {
		color: var(--shu-500) !important;
		font-weight: 700;
	}

	.latest {
		display: grid;
		grid-template-columns: 1fr auto;
		align-items: center;
		gap: 16px;
		padding: 28px;
		border-radius: var(--radius-card);
		background: var(--ink-900);
		color: var(--sumi-text);
	}

	.latest .body-s {
		margin-top: 10px;
		color: var(--sumi-muted);
	}

	.seal {
		position: relative;
		display: grid;
		place-items: center;
	}

	.seal > span {
		position: absolute;
	}

	.filters {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		margin: 26px 0 18px;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 20px;
	}

	.ach {
		display: grid;
		justify-items: center;
		align-content: start;
		gap: 8px;
		padding: 26px 18px 22px;
		text-align: center;
	}

	.ach.locked {
		background: var(--paper-sunken);
		box-shadow: none;
	}

	.badge {
		position: relative;
		display: grid;
		place-items: center;
		margin-bottom: 8px;
	}

	.badge .h {
		position: absolute;
	}

	.lock {
		position: absolute;
		right: 2px;
		bottom: 4px;
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: var(--ink-900);
		color: #fff;
	}

	.progress {
		display: grid;
		gap: 6px;
		width: 100%;
		margin-top: 6px;
	}
</style>
