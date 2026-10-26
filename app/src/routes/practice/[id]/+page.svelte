<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, errorText, type CheckResult, type HintReport, type LessonView, type StepView } from '#lib/api.ts';
	import Icon from '#lib/components/Icon.svelte';
	import LessonHeader from '#lib/components/LessonHeader.svelte';
	import { refreshSession } from '#lib/session.svelte.ts';

	const id = $derived(page.params.id ?? '');

	let view = $state<LessonView | null>(null);
	let steps = $state<StepView[]>([]);
	let attempt = $state<number | null>(null);
	let phase = $state<'starting' | 'ready' | 'failed'>('starting');
	let failure = $state('');
	let result = $state<CheckResult | null>(null);
	let hint = $state<HintReport | null>(null);
	let checking = $state(false);
	let auto = $state(true);
	let note = $state('');
	let timer: ReturnType<typeof setInterval> | null = null;

	const done = $derived(result?.done ?? 0);
	const current = $derived(steps[done] ?? null);
	const currentOutcome = $derived(result?.outcome.steps[done] ?? null);
	const failing = $derived(currentOutcome && !currentOutcome.passed ? currentOutcome.checks.filter((c) => !c.passed) : []);

	async function start() {
		phase = 'starting';
		failure = '';
		try {
			view ??= await api.lesson(id);
			const r = await api.practiceStart(id);
			steps = r.steps;
			attempt = r.attempt;
			result = null;
			hint = null;
			phase = 'ready';
			startTimer();
		} catch (e) {
			failure = errorText(e);
			phase = 'failed';
		}
	}

	function startTimer() {
		stopTimer();
		timer = setInterval(() => {
			if (auto && phase === 'ready' && !checking) check(true);
		}, 4000);
	}

	function stopTimer() {
		if (timer) clearInterval(timer);
		timer = null;
	}

	async function check(quiet = false) {
		if (checking || phase !== 'ready') return;
		checking = true;
		if (!quiet) note = '';
		try {
			const r = await api.practiceCheck();
			const advanced = r.done > (result?.done ?? 0);
			result = r;
			if (advanced) hint = null;
			if (!quiet && !advanced && !r.finished) note = 'Пока не получилось — смотрите подсказку под шагом.';
			if (r.finished) {
				stopTimer();
				await refreshSession();
				goto(`/result/${r.attempt}`);
			}
		} catch (e) {
			if (!quiet) note = errorText(e);
			if (errorText(e).includes('редактор закрыт')) {
				stopTimer();
				failure = errorText(e);
				phase = 'failed';
			}
		} finally {
			checking = false;
		}
	}

	async function showHint() {
		try {
			hint = await api.practiceHint();
		} catch (e) {
			note = errorText(e);
		}
	}

	async function confirm(step: string) {
		await api.practiceConfirm(step);
		check();
	}

	function onkeydown(e: KeyboardEvent) {
		const target = e.target as HTMLElement | null;
		if (target?.closest('input, textarea')) return;
		if (e.key.toLowerCase() === 'h' || e.key.toLowerCase() === 'р') showHint();
		if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) check();
	}

	async function leave() {
		stopTimer();
		await api.practiceStop();
	}

	onMount(start);
	onDestroy(stopTimer);
</script>

<svelte:window {onkeydown} />

<div class="practice">
	<LessonHeader
		dark
		title={view?.card.title ?? 'Практика'}
		subtitle="{view?.card.categoryTitle ?? ''} · практика"
		stage={2}
		back="/lesson/{id}"
		onclose={leave}
	/>

	{#if phase === 'failed'}
		<section class="failed reveal">
			<div class="card box">
				<Icon name="alert" size={30} />
				<h2 class="h1">Редактор не запустился</h2>
				<p class="muted">{failure}</p>
				<p class="body-s muted">Урок сохранён: гайд и теория доступны, а к заданию можно вернуться в любой момент.</p>
				<div class="row">
					<button class="btn btn-accent" onclick={start}><Icon name="refresh" size={18} /> Попробовать снова</button>
					<a class="btn btn-quiet" href="/lesson/{id}">К теории</a>
				</div>
			</div>
		</section>
	{:else}
		<div class="work">
			<section class="canvas">
				<div class="note">
					<Icon name="layers" size={18} />
					{#if phase === 'starting'}
						Открываем PhotoCraft и задание…
					{:else}
						Редактор PhotoCraft открыт в отдельном окне. Рисуйте там — FUDE следит за шагами и проверяет их сам.
					{/if}
				</div>
				<div class="pictures">
					<figure>
						{#if view?.taskImage}
							<img src={view.taskImage} alt="Начало задания" />
						{:else}
							<div class="blank" style="aspect-ratio: {view?.lesson.task.canvas?.width ?? 3} / {view?.lesson.task.canvas?.height ?? 2}"></div>
						{/if}
						<figcaption>Начало</figcaption>
					</figure>
					{#if view?.referenceImage}
						<figure>
							<img src={view.referenceImage} alt="Эталон" />
							<figcaption>Эталон — к чему стремиться</figcaption>
						</figure>
					{/if}
				</div>
				{#if hint}
					<div class="hint-banner reveal">
						<Icon name="bulb" size={18} /> Режим подсказки включён — это будет видно в статистике
						{#if hint.shapes}· на холсте появился слой «FUDE · подсказка»{/if}
					</div>
				{/if}
			</section>

			<aside class="panel">
				<div class="task">
					<h2 class="h2">{view?.lesson.task.brief ?? ''}</h2>
					<ol class="steps">
						{#each steps as s, i (s.id)}
							<li class:done={i < done} class:current={i === done} aria-current={i === done ? 'step' : undefined}>
								<span class="mark">{#if i < done}<Icon name="check" size={13} stroke={2.6} />{/if}</span>
								<span class="text">
									{s.text}
									{#if i === done && s.manual}
										<button class="btn btn-quiet btn-s ready" onclick={() => confirm(s.id)}>Готово</button>
									{/if}
								</span>
							</li>
						{/each}
					</ol>

					{#if failing.length && result}
						<div class="almost" role="status">
							<p class="label">Почти получилось</p>
							{#each failing as c (c.kind + c.message)}<p class="body-s">{c.message}</p>{/each}
						</div>
					{/if}

					{#if hint}
						<div class="hint reveal">
							<span class="n">{done + 1}</span>
							<div>
								<p class="label">{current?.text}</p>
								<p class="body-s">{hint.text}</p>
								{#if hint.target}<p class="body-s target">Где: <span class="mono">{hint.target}</span></p>{/if}
							</div>
						</div>
					{/if}

					<div class="hints">
						<span class="body-s">Подсказки</span>
						<span class="bulbs" aria-label="Использовано подсказок: {result?.hints ?? hint?.hints ?? 0}">
							{#each steps as s, i (s.id)}
								<span class:used={i < (hint?.hints ?? result?.hints ?? 0)}><Icon name="bulb" size={16} /></span>
							{/each}
						</span>
					</div>
					<p class="body-s quiet">Подсказки всегда под рукой. Задание без них даёт в полтора раза больше опыта.</p>
				</div>

				<div class="actions">
					<label class="auto"><input type="checkbox" bind:checked={auto} /> Проверять автоматически</label>
					{#if note}<p class="body-s note-text" role="status">{note}</p>{/if}
					<div class="row">
						<button class="btn btn-hint" onclick={showHint} disabled={phase !== 'ready' || !current?.hasHint}>
							<Icon name="bulb" size={18} /> Подсказка <span class="kbd">H</span>
						</button>
						<button class="btn btn-accent" onclick={() => check()} disabled={phase !== 'ready' || checking}>
							{checking ? 'Проверяем…' : 'Проверить'}
						</button>
					</div>
				</div>
			</aside>
		</div>
	{/if}
</div>

<style>
	.practice {
		min-height: 100vh;
		background: var(--sumi-bg);
		color: var(--sumi-text);
	}

	.work {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 400px;
		min-height: calc(100vh - 62px);
	}

	.canvas {
		display: grid;
		align-content: start;
		gap: 24px;
		padding: 32px 40px;
	}

	.note,
	.hint-banner {
		justify-self: center;
		display: inline-flex;
		align-items: center;
		gap: 10px;
		padding: 8px 16px;
		border-radius: 99px;
		background: rgb(47 93 140 / 22%);
		box-shadow: inset 0 0 0 1px rgb(143 178 214 / 35%);
		font-size: 13px;
		font-weight: 600;
	}

	.pictures {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
		gap: 24px;
	}

	figure {
		display: grid;
		gap: 10px;
		margin: 0;
	}

	figure img,
	.blank {
		width: 100%;
		border-radius: 6px;
		background: #fff;
		box-shadow: 0 20px 50px rgb(0 0 0 / 45%);
	}

	figcaption {
		font-size: 13px;
		color: var(--sumi-muted);
	}

	.panel {
		display: grid;
		grid-template-rows: 1fr auto;
		gap: 16px;
		padding: 16px;
		background: var(--sumi-panel);
		border-left: 1px solid var(--sumi-line);
	}

	.task {
		align-self: start;
		display: grid;
		gap: 16px;
		padding: 22px;
		border-radius: 14px;
		background: var(--sumi-raised);
	}

	.steps {
		display: grid;
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.steps li {
		display: grid;
		grid-template-columns: 22px 1fr;
		gap: 10px;
		font-size: 14px;
		color: var(--sumi-muted);
	}

	.mark {
		display: grid;
		place-items: center;
		width: 22px;
		height: 22px;
		margin-top: 1px;
		border-radius: 50%;
		box-shadow: inset 0 0 0 1.5px var(--sumi-line);
	}

	.done .mark {
		background: var(--ai-500);
		box-shadow: none;
		color: #fff;
	}

	.done .text {
		text-decoration: line-through;
		text-decoration-color: rgb(163 167 175 / 60%);
	}

	.current {
		color: var(--sumi-text) !important;
		font-weight: 600;
	}

	.current .mark {
		box-shadow:
			inset 0 0 0 1.5px var(--ai-300),
			inset 0 0 0 6px var(--sumi-raised),
			inset 0 0 0 11px var(--ai-300);
		animation: breathe 1.6s ease-in-out infinite;
	}

	@keyframes breathe {
		50% {
			opacity: 0.55;
		}
	}

	.ready {
		display: flex;
		margin-top: 8px;
	}

	.almost {
		display: grid;
		gap: 4px;
		padding: 12px 14px;
		border-radius: 12px;
		background: rgb(232 137 59 / 14%);
		box-shadow: inset 0 0 0 1px rgb(232 137 59 / 35%);
		color: #f3c9a0;
	}

	.almost .label {
		color: var(--kaki-500);
	}

	.hint {
		display: grid;
		grid-template-columns: 28px 1fr;
		gap: 12px;
		padding: 14px;
		border-radius: 12px;
		background: var(--ai-700);
		color: #fff;
	}

	.hint .n {
		display: grid;
		place-items: center;
		width: 26px;
		height: 26px;
		border-radius: 50%;
		background: #fff;
		color: var(--ai-700);
		font-weight: 700;
		font-size: 13px;
	}

	.hint .body-s {
		margin-top: 4px;
		color: var(--ai-100);
	}

	.target .mono {
		color: #fff;
	}

	.hints {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding-top: 14px;
		border-top: 1px solid var(--sumi-line);
		color: var(--sumi-muted);
	}

	.bulbs {
		display: flex;
		gap: 4px;
		color: var(--sumi-line);
	}

	.bulbs .used {
		color: var(--kaki-500);
	}

	.quiet {
		color: var(--sumi-muted);
	}

	.actions {
		display: grid;
		gap: 12px;
		padding: 6px;
	}

	.auto {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		color: var(--sumi-muted);
	}

	.note-text {
		color: var(--kaki-500);
	}

	.row {
		display: flex;
		gap: 10px;
	}

	.row .btn {
		flex: 1;
	}

	.btn-hint {
		color: var(--ai-100);
	}

	.btn-hint .kbd {
		min-width: 20px;
		height: 20px;
		background: rgb(255 255 255 / 12%);
		box-shadow: none;
		color: inherit;
	}

	.failed {
		display: grid;
		place-items: center;
		min-height: calc(100vh - 62px);
		padding: 40px;
	}

	.box {
		display: grid;
		gap: 14px;
		justify-items: start;
		max-width: 560px;
		padding: 36px;
		color: var(--ink-900);
	}

	.box > :global(svg) {
		color: var(--kaki-500);
	}
</style>
