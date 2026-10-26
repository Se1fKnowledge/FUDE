<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, errorText, type ResultView } from '#lib/api.ts';
	import Enso from '#lib/components/Enso.svelte';
	import Hanko from '#lib/components/Hanko.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Mountains from '#lib/components/Mountains.svelte';
	import { clock, count, percent } from '#lib/format.ts';

	let r = $state<ResultView | null>(null);
	let error = $state('');

	onMount(async () => {
		try {
			r = await api.result(Number(page.params.attempt));
		} catch (e) {
			error = errorText(e);
		}
	});

	const toNext = $derived(r && r.rank.next !== null ? r.rank.next - r.rank.xp : null);
	const verdict = $derived.by(() => {
		if (!r) return '';
		if (r.hints === 0) return 'Без единой подсказки — так держать. Сравните свою работу с эталоном: что бы вы сделали иначе?';
		return `Задание выполнено с ${count(r.hints, ['подсказкой', 'подсказками', 'подсказками'])}. Пройдите урок ещё раз самостоятельно — это даст больше опыта и очков навыка.`;
	});
</script>

<div class="result">
	<a class="brand" href="/"><Hanko kanji="筆" size={32} /> <span>FUDE</span></a>
	<a class="close" href="/" aria-label="На главную"><Icon name="close" /></a>
	<div class="ring" aria-hidden="true"><Enso size={620} width={40} color="rgb(22 22 26 / 5%)" seed={12} /></div>

	{#if error}
		<p class="muted center">{error}</p>
	{:else if r}
		<section class="head">
			<Hanko kanji={r.passed ? '完' : '再'} size={110} tone={r.passed ? 'shu' : 'ink'} tilt={-5} stamp />
			<h1 class="display-xl">{r.card?.title ?? 'Урок'}</h1>
			<p class="muted">
				{r.passed ? 'Урок пройден' : 'Урок не закончен'} · {r.card?.categoryTitle}, урок {r.card?.number} из {r.card?.of}
			</p>
			<p class="verdict">{verdict}</p>
		</section>

		<section class="facts">
			<div class="fact card">
				<span class="ico ai"><Icon name="target" /></span>
				<span>
					<span class="body-s muted">Сходство</span>
					<span class="value number">{r.similarity !== null ? percent(r.similarity) : '—'}</span>
					<span class="body-s muted">{r.similarity !== null ? 'с эталоном урока' : 'у урока нет эталона'}</span>
				</span>
			</div>
			<div class="fact card">
				<span class="ico"><Icon name="clock" /></span>
				<span>
					<span class="body-s muted">Время</span>
					<span class="value number">{clock(r.seconds)}</span>
					<span class="body-s muted">норма — {r.normMinutes} мин</span>
				</span>
			</div>
			<div class="fact card">
				<span class="ico"><Icon name="bulb" /></span>
				<span>
					<span class="body-s muted">Подсказки</span>
					<span class="value number">{r.hints} из {r.steps}</span>
					<span class="body-s muted">{r.hints === 0 ? 'без подсказок — ×1,5 опыта' : 'шагов с подсказкой'}</span>
				</span>
			</div>
			<div class="fact dark">
				<span class="ico kaki"><Icon name="sparkle" /></span>
				<span>
					<span class="body-s">Опыт</span>
					<span class="value number xp">+{r.xp} XP</span>
					<span class="body-s">{r.streak.current > 0 ? `серия ${count(r.streak.current, ['день', 'дня', 'дней'])}` : 'первый день серии'}</span>
				</span>
			</div>
		</section>

		<section class="lower">
			<div class="compare card">
				<figure>
					{#if r.canvas}<img src={r.canvas} alt="Ваша работа" />{:else}<div class="none">нет снимка</div>{/if}
					<figcaption>Ваша работа</figcaption>
				</figure>
				<figure>
					{#if r.reference}<img src={r.reference} alt="Эталон" />{:else}<div class="none">нет эталона</div>{/if}
					<figcaption>Эталон</figcaption>
				</figure>
			</div>
			<div class="growth card">
				<h2 class="h2">Рост навыков</h2>
				{#each r.skills as [label, points] (label)}
					<div class="skill">
						<span>{label}</span>
						<span class="track"><i style="width: {Math.min(100, points * 2)}%"></i></span>
						<span class="pill pill-ai">+{points}</span>
					</div>
				{/each}
				<div class="rank">
					<Hanko kanji="級" size={34} tone="ai" tilt={0} />
					<span>
						<span class="body-s muted">{r.rank.title} · {r.rank.name}</span>
						<span class="label">{toNext !== null ? `До следующего ранга осталось ${toNext} XP` : 'Высший ранг получен'}</span>
					</span>
				</div>
				{#each r.achievements as a (a.id)}
					<div class="ach reveal"><Hanko kanji={a.kanji} size={30} /> Новое достижение: «{a.title}»</div>
				{/each}
			</div>
		</section>

		<section class="actions">
			{#if r.next}
				<a class="btn btn-accent" href="/lesson/{r.next.id}">Следующий урок: «{r.next.title}»</a>
			{/if}
			<a class="btn btn-outline" href="/lesson/{r.card?.id}">Повторить урок</a>
			<a class="btn btn-quiet" href="/">На главную</a>
		</section>
	{/if}

	<div class="mist" aria-hidden="true"><Mountains height={200} seed={8} /></div>
</div>

<style>
	.result {
		position: relative;
		min-height: 100vh;
		padding: 34px 40px 220px;
		overflow: hidden;
		background:
			radial-gradient(700px 400px at 0% 0%, rgb(246 217 210 / 70%), transparent 70%),
			radial-gradient(700px 400px at 100% 10%, rgb(220 231 241 / 80%), transparent 70%), var(--paper-base);
	}

	.brand {
		position: absolute;
		left: 40px;
		top: 30px;
		display: flex;
		align-items: center;
		gap: 12px;
		color: var(--ink-900);
		font-family: var(--font-display);
		font-weight: 600;
		font-size: 28px;
		letter-spacing: 0.08em;
	}

	.close {
		position: absolute;
		right: 40px;
		top: 30px;
		display: grid;
		place-items: center;
		width: 42px;
		height: 42px;
		border-radius: 50%;
		background: var(--paper-raised);
		box-shadow: var(--shadow-card);
		color: var(--ink-900);
	}

	.ring {
		position: absolute;
		left: 50%;
		top: 10px;
		transform: translateX(-50%);
		pointer-events: none;
	}

	.head {
		position: relative;
		display: grid;
		justify-items: center;
		gap: 10px;
		margin-top: 40px;
		text-align: center;
	}

	.head h1 {
		margin-top: 18px;
	}

	.verdict {
		max-width: 62ch;
		margin-top: 6px;
	}

	.facts {
		position: relative;
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 18px;
		max-width: 1100px;
		margin: 40px auto 0;
	}

	.fact {
		display: flex;
		gap: 16px;
		padding: 22px;
		border-radius: var(--radius-card);
	}

	.fact > span:last-child {
		display: grid;
	}

	.fact.dark {
		background: var(--ink-900);
		color: var(--sumi-text);
	}

	.fact.dark .body-s {
		color: var(--sumi-muted);
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

	.ico.ai {
		background: var(--ai-100);
		color: var(--ai-500);
	}

	.ico.kaki {
		background: var(--sumi-raised);
		color: var(--kaki-500);
	}

	.value {
		font-size: 34px;
		line-height: 1.1;
	}

	.xp {
		color: var(--kaki-500);
	}

	.lower {
		position: relative;
		display: grid;
		grid-template-columns: 1.4fr 1fr;
		gap: 18px;
		max-width: 1100px;
		margin: 18px auto 0;
	}

	.compare {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
		padding: 20px;
	}

	figure {
		display: grid;
		gap: 8px;
		margin: 0;
	}

	figure img,
	.none {
		width: 100%;
		aspect-ratio: 3 / 2;
		object-fit: cover;
		border-radius: 10px;
		box-shadow: inset 0 0 0 1px var(--paper-line);
	}

	.none {
		display: grid;
		place-items: center;
		background: var(--paper-sunken);
		color: var(--ink-500);
		font-size: 13px;
	}

	figcaption {
		font-size: 13px;
		color: var(--ink-500);
	}

	.growth {
		display: grid;
		align-content: start;
		gap: 14px;
		padding: 22px;
	}

	.skill {
		display: grid;
		grid-template-columns: 130px 1fr auto;
		align-items: center;
		gap: 12px;
		font-size: 14px;
	}

	.rank {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px;
		border-radius: 12px;
		background: var(--paper-sunken);
	}

	.rank > span:last-child {
		display: grid;
	}

	.ach {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 14px;
		border-radius: 12px;
		background: var(--kin-100);
		color: var(--kin-700);
		font-weight: 600;
		font-size: 14px;
	}

	.actions {
		position: relative;
		z-index: 1;
		display: flex;
		justify-content: center;
		gap: 12px;
		margin-top: 30px;
	}

	.mist {
		position: absolute;
		left: 0;
		right: 0;
		bottom: 0;
		height: 200px;
	}

	.center {
		text-align: center;
		margin-top: 200px;
	}
</style>
