<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { open } from '@tauri-apps/plugin-dialog';
	import { api, errorText, type DraftDetail, type Issue, type Lesson, type Skill, type Step } from '#lib/api.ts';
	import { describe, stepId } from '#lib/checks.ts';
	import CheckEditor from '#lib/components/CheckEditor.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Markdown from '#lib/components/Markdown.svelte';
	import RegionPicker from '#lib/components/RegionPicker.svelte';
	import { defaultCheck } from '#lib/checks.ts';
	import { count } from '#lib/format.ts';

	const dir = $derived(page.url.searchParams.get('dir') ?? '');

	let draft = $state<DraftDetail | null>(null);
	let lesson = $state<Lesson | null>(null);
	let theory = $state('');
	let guide = $state('');
	let issues = $state<Issue[]>([]);
	let taskImage = $state<string | null>(null);
	let referenceImage = $state<string | null>(null);
	let section = $state<'about' | 'theory' | 'guide' | 'quiz' | 'task'>('about');
	let preview = $state(false);
	let saving = $state(false);
	let saved = $state('');
	let error = $state('');
	let openStep = $state(0);

	async function load() {
		try {
			// Clone the plain reply before Svelte makes it reactive.
			const d = await api.draft(dir);
			const editable = structuredClone(d.lesson);
			editable.quiz ??= [];
			editable.requires ??= [];
			draft = d;
			lesson = editable;
			theory = d.theory;
			guide = d.guide ?? '';
			issues = d.issues;
			taskImage = d.taskImage;
			referenceImage = d.referenceImage;
		} catch (e) {
			error = errorText(e);
		}
	}

	onMount(load);

	async function saveAll() {
		if (!lesson) return;
		saving = true;
		error = '';
		try {
			const clean = $state.snapshot(lesson) as Lesson;
			if (!clean.requires?.length) delete clean.requires;
			if (!clean.quiz?.length) delete clean.quiz;
			issues = await api.draftSave(dir, clean, theory, guide.trim() ? guide : lesson.guide.script ? '' : null);
			saved = issues.some((i) => i.severity === 'error') ? 'Сохранено как черновик: исправьте ошибки, чтобы урок попал в каталог.' : 'Сохранено. Урок в каталоге.';
			draft = await api.draft(dir);
		} catch (e) {
			error = errorText(e);
		} finally {
			saving = false;
		}
	}

	async function attach(kind: 'video' | 'image', target: 'video' | 'open' | 'reference' | 'poster') {
		if (!lesson) return;
		const filters = kind === 'video' ? [{ name: 'Видео', extensions: ['mp4', 'webm'] }] : [{ name: 'Картинка', extensions: ['png', 'jpg', 'jpeg', 'webp'] }];
		const source = await open({ multiple: false, filters });
		if (typeof source !== 'string') return;
		try {
			const rel = await api.attach(dir, kind, source);
			if (target === 'video') lesson.guide.video = rel;
			if (target === 'poster') lesson.guide.poster = rel;
			if (target === 'reference') {
				lesson.task.reference = rel;
				referenceImage = await api.picture(dir, rel);
			}
			if (target === 'open') {
				lesson.task.open = rel;
				delete lesson.task.canvas;
				taskImage = await api.picture(dir, rel);
			}
		} catch (e) {
			error = errorText(e);
		}
	}

	function useCanvas() {
		if (!lesson) return;
		delete lesson.task.open;
		lesson.task.canvas = { width: 1200, height: 800, background: '#FFFFFF' };
		taskImage = null;
	}

	function addStep() {
		if (!lesson) return;
		const step: Step = { id: stepId(lesson.task.steps.map((s) => s.id)), text: '', checks: [defaultCheck('tool')] };
		lesson.task.steps.push(step);
		openStep = lesson.task.steps.length - 1;
	}

	function moveStep(i: number, by: number) {
		if (!lesson) return;
		const steps = lesson.task.steps;
		const j = i + by;
		if (j < 0 || j >= steps.length) return;
		[steps[i], steps[j]] = [steps[j], steps[i]];
		openStep = j;
	}

	function toggleSkill(s: Skill) {
		if (!lesson) return;
		lesson.skills = lesson.skills.includes(s) ? lesson.skills.filter((x) => x !== s) : [...lesson.skills, s];
	}

	const SKILLS: [Skill, string][] = [
		['tools', 'Инструменты'],
		['line', 'Линия'],
		['form', 'Форма'],
		['light', 'Свет и тень'],
		['color', 'Цвет'],
		['composition', 'Композиция']
	];
	const SECTIONS = [
		['about', 'Описание'],
		['theory', 'Теория'],
		['guide', 'Гайд и видео'],
		['quiz', 'Вопросы'],
		['task', 'Задание']
	] as const;
	const aspect = $derived(lesson?.task.canvas ? lesson.task.canvas.width / lesson.task.canvas.height : 1.5);
	const errors = $derived(issues.filter((i) => i.severity === 'error'));
	const warnings = $derived(issues.filter((i) => i.severity === 'warning'));
	const issuesFor = (prefix: string) => issues.filter((i) => i.field.startsWith(prefix));
	const sectionOf = (field: string) =>
		field.startsWith('task') ? 'task' : field.startsWith('quiz') ? 'quiz' : field.startsWith('guide') || field.includes('.json') ? 'guide' : field.startsWith('theory') ? 'theory' : 'about';
</script>

<div class="page reveal">
	<header class="page-head">
		<div>
			<a class="back" href="/studio"><Icon name="back" size={16} /> Мастерская</a>
			<h1 class="display-l">{lesson?.title || 'Урок'}</h1>
			{#if draft}<p>{draft.category.title} · <span class="mono">{lesson?.id}</span></p>{/if}
		</div>
		<div class="head-actions">
			{#if lesson && !errors.length && draft}<a class="btn btn-quiet" href="/lesson/{lesson.id}">Открыть как ученик</a>{/if}
			<button class="btn btn-accent" onclick={saveAll} disabled={saving || !lesson}>{saving ? 'Сохраняем…' : 'Сохранить'}</button>
		</div>
	</header>

	{#if error}<div class="banner warn" role="alert"><Icon name="alert" size={18} /> {error}</div>{/if}
	{#if saved}<div class="banner {errors.length ? 'warn' : 'ok'}" role="status">{saved}</div>{/if}

	{#if lesson}
		<div class="layout">
			<nav class="sections card" aria-label="Разделы урока">
				{#each SECTIONS as [key, label] (key)}
					{@const n = issues.filter((i) => i.severity === 'error' && sectionOf(i.field) === key).length}
					<button aria-pressed={section === key} onclick={() => (section = key)}>
						{label}
						{#if n}<span class="pill pill-kaki">{n}</span>{/if}
					</button>
				{/each}
				<div class="summary body-s">
					{#if errors.length}
						<p class="bad">{count(errors.length, ['ошибка', 'ошибки', 'ошибок'])} — урок не в каталоге</p>
					{:else}
						<p class="good">Ошибок нет</p>
					{/if}
					{#if warnings.length}<p class="muted">{count(warnings.length, ['предупреждение', 'предупреждения', 'предупреждений'])}</p>{/if}
				</div>
			</nav>

			<section class="card body">
				{#if section === 'about'}
					<h2 class="h2">Описание</h2>
					<label class="field">Название <input type="text" bind:value={lesson.title} maxlength="120" /></label>
					<label class="field">Кратко <textarea rows="2" bind:value={lesson.summary}></textarea></label>
					<div class="grid3">
						<label class="field">
							Сложность
							<select bind:value={lesson.difficulty}>
								{#each [[1, 'Первый шаг'], [2, 'Лёгкий'], [3, 'Средний'], [4, 'Сложный'], [5, 'Мастер']] as [v, l] (v)}<option value={v}>{v} — {l}</option>{/each}
							</select>
						</label>
						<label class="field">Минут <input type="number" min="1" max="600" bind:value={lesson.minutes} /></label>
						<label class="field">Порядок в категории <input type="number" min="0" bind:value={lesson.order} /></label>
					</div>
					<fieldset>
						<legend class="label">Стороны навыка (для статистики)</legend>
						<div class="chips">
							{#each SKILLS as [s, l] (s)}<button type="button" class="chip" aria-pressed={lesson.skills.includes(s)} onclick={() => toggleSkill(s)}>{l}</button>{/each}
						</div>
					</fieldset>
					<label class="field">
						Открывается после урока
						<select value={lesson.requires?.[0] ?? ''} onchange={(e) => lesson && (lesson.requires = e.currentTarget.value ? [e.currentTarget.value] : [])}>
							<option value="">— сразу открыт —</option>
							{#each draft?.lessonIds.filter(([id]) => id !== lesson?.id) ?? [] as [id, title] (id)}<option value={id}>{title}</option>{/each}
						</select>
					</label>
					<label class="field">Автор <input type="text" value={lesson.author ?? ''} onchange={(e) => lesson && (lesson.author = e.currentTarget.value.trim() || undefined)} /></label>
				{:else if section === 'theory'}
					<div class="row-head">
						<h2 class="h2">Теория</h2>
						<div class="segmented">
							<button aria-pressed={!preview} onclick={() => (preview = false)}>Текст</button>
							<button aria-pressed={preview} onclick={() => (preview = true)}>Как увидит ученик</button>
						</div>
					</div>
					{#if preview}
						<div class="preview"><Markdown text={theory} /></div>
					{:else}
						<textarea class="theory mono" rows="24" bind:value={theory} aria-label="Теория в Markdown"></textarea>
						<p class="body-s muted">Markdown: <span class="mono"># заголовок</span>, <span class="mono">**жирный**</span>, списки <span class="mono">- пункт</span>, таблицы. Завершите разделом «## Совет сэнсэя».</p>
					{/if}
				{:else if section === 'guide'}
					<h2 class="h2">Видео</h2>
					{#if lesson.guide.video && draft?.video}
						<!-- svelte-ignore a11y_media_has_caption -->
						<video src={convertFileSrc(draft.video)} controls></video>
					{/if}
					<div class="row">
						<button class="btn btn-quiet" onclick={() => attach('video', 'video')}><Icon name="video" size={18} /> {lesson.guide.video ? 'Заменить видео' : 'Загрузить видео'}</button>
						<button class="btn btn-quiet" onclick={() => attach('image', 'poster')}><Icon name="upload" size={18} /> Обложка видео</button>
						{#if lesson.guide.video}<button class="btn btn-quiet" onclick={() => lesson && delete lesson.guide.video}>Убрать видео</button>{/if}
					</div>
					<p class="body-s muted">mp4 или webm до 1 ГиБ. Файл копируется в папку урока.</p>
					<h2 class="h2 gap">Анимированный гайд в редакторе</h2>
					<p class="body-s muted">Сценарий команд PhotoCraft в JSON (формат — docs/lesson-format.md). Гайд может только рисовать: файлы и сеть ему недоступны.</p>
					<textarea class="theory mono" rows="16" bind:value={guide} placeholder={'{\n  "format": 1,\n  "steps": [ { "caption": "…", "calls": [] } ]\n}'} aria-label="Сценарий гайда"></textarea>
					{#each issuesFor('guide') as i (i.field + i.message)}<p class="issue">{i.field}: {i.message}</p>{/each}
				{:else if section === 'quiz'}
					<h2 class="h2">Вопросы после теории</h2>
					{#each lesson.quiz ?? [] as q, qi (qi)}
						<div class="question">
							<div class="row-head">
								<label class="field grow">Вопрос {qi + 1} <input type="text" bind:value={q.question} /></label>
								<button class="icon" onclick={() => lesson?.quiz?.splice(qi, 1)} aria-label="Удалить вопрос"><Icon name="trash" size={16} /></button>
							</div>
							{#each q.options as _, oi (oi)}
								<div class="option">
									<input type="radio" name="answer-{qi}" checked={q.answer === oi} onchange={() => (q.answer = oi)} aria-label="Верный ответ" />
									<input type="text" bind:value={q.options[oi]} />
									<button class="icon" onclick={() => (q.options.splice(oi, 1), (q.answer = Math.min(q.answer, q.options.length - 1)))} aria-label="Удалить вариант"><Icon name="close" size={14} /></button>
								</div>
							{/each}
							<button class="btn btn-quiet btn-s" onclick={() => q.options.push('')} disabled={q.options.length >= 8}><Icon name="plus" size={14} /> Вариант</button>
							<label class="field">Пояснение <input type="text" value={q.explain ?? ''} onchange={(e) => (q.explain = e.currentTarget.value.trim() || undefined)} /></label>
						</div>
					{/each}
					<button class="btn btn-quiet" onclick={() => lesson?.quiz?.push({ question: '', options: ['', ''], answer: 0 })}><Icon name="plus" size={16} /> Добавить вопрос</button>
					{#each issuesFor('quiz') as i (i.field + i.message)}<p class="issue">{i.field}: {i.message}</p>{/each}
				{:else}
					<h2 class="h2">Задание</h2>
					<label class="field">Что нужно сделать <textarea rows="2" bind:value={lesson.task.brief}></textarea></label>
					<div class="start">
						<div>
							<p class="label">Начало</p>
							{#if lesson.task.canvas}
								<div class="grid3">
									<label class="field">Ширина <input type="number" min="16" max="8192" bind:value={lesson.task.canvas.width} /></label>
									<label class="field">Высота <input type="number" min="16" max="8192" bind:value={lesson.task.canvas.height} /></label>
									<label class="field">Фон <input type="color" bind:value={lesson.task.canvas.background} /></label>
								</div>
							{:else if taskImage}
								<img src={taskImage} alt="Картинка задания" />
							{/if}
							<div class="row">
								<button class="btn btn-quiet btn-s" onclick={() => attach('image', 'open')}><Icon name="upload" size={16} /> Картинка задания</button>
								{#if !lesson.task.canvas}<button class="btn btn-quiet btn-s" onclick={useCanvas}>Чистый холст</button>{/if}
							</div>
						</div>
						<div>
							<p class="label">Эталон</p>
							{#if referenceImage}<img src={referenceImage} alt="Эталон" />{:else}<p class="body-s muted">Необязательно. Показывается ученику и нужен для проверки «похоже на эталон».</p>{/if}
							<button class="btn btn-quiet btn-s" onclick={() => attach('image', 'reference')}><Icon name="upload" size={16} /> {referenceImage ? 'Заменить' : 'Загрузить'} эталон</button>
						</div>
					</div>

					<h3 class="h3 gap">Шаги</h3>
					{#each lesson.task.steps as step, si (step.id + si)}
						<div class="step" class:open={openStep === si}>
							<button class="step-head" onclick={() => (openStep = openStep === si ? -1 : si)} aria-expanded={openStep === si}>
								<span class="n">{si + 1}</span>
								<span class="grow">
									<span class="label">{step.text || 'Новый шаг'}</span>
									<span class="body-s muted">{step.checks.map(describe).join(' · ') || 'без проверок — кнопка «Готово»'}</span>
								</span>
								{#if issuesFor(`task.steps[${si}]`).some((i) => i.severity === 'error')}<span class="pill pill-kaki">ошибки</span>{/if}
							</button>
							{#if openStep === si}
								<div class="step-body">
									<label class="field">Текст шага <input type="text" bind:value={step.text} /></label>
									<details open={!!step.hint}>
										<summary class="label">Подсказка</summary>
										<label class="field">Текст подсказки <input type="text" value={step.hint?.text ?? ''} onchange={(e) => (step.hint = e.currentTarget.value.trim() ? { ...(step.hint ?? {}), text: e.currentTarget.value.trim() } : undefined)} /></label>
										{#if step.hint}
											<label class="field">Куда показать (tool:Brush, panel:Layers, menu:edit.undo) <input type="text" value={step.hint.target ?? ''} onchange={(e) => step.hint && (step.hint.target = e.currentTarget.value.trim() || undefined)} /></label>
											<p class="body-s muted">Теневая фигура на холсте (необязательно):</p>
											{#if step.hint.shapes?.length && (step.hint.shapes[0].kind === 'ellipse' || step.hint.shapes[0].kind === 'rect')}
												{@const shape = step.hint.shapes[0]}
												<RegionPicker label="Овал подсказки" bind:value={shape.rect} image={taskImage} {aspect} color="var(--ai-500)" />
												<button class="btn btn-quiet btn-s" onclick={() => step.hint && (step.hint.shapes = [])}>Убрать фигуру</button>
											{:else}
												<button class="btn btn-quiet btn-s" onclick={() => step.hint && (step.hint.shapes = [{ kind: 'ellipse', rect: [0.35, 0.35, 0.3, 0.3] }])}><Icon name="plus" size={14} /> Овал на холсте</button>
											{/if}
										{/if}
									</details>
									<p class="label">Проверки — шаг засчитан, когда проходят все</p>
									{#each step.checks as _, ci (ci)}
										<CheckEditor bind:check={step.checks[ci]} image={taskImage} {aspect} onremove={() => step.checks.splice(ci, 1)} />
									{/each}
									<div class="row">
										<button class="btn btn-quiet btn-s" onclick={() => step.checks.push(defaultCheck('coverage'))}><Icon name="plus" size={14} /> Проверка</button>
										<button class="btn btn-quiet btn-s" onclick={() => moveStep(si, -1)} disabled={si === 0}>Выше</button>
										<button class="btn btn-quiet btn-s" onclick={() => moveStep(si, 1)} disabled={si === lesson!.task.steps.length - 1}>Ниже</button>
										<button class="btn btn-quiet btn-s danger" onclick={() => lesson?.task.steps.splice(si, 1)}><Icon name="trash" size={14} /> Удалить шаг</button>
									</div>
									{#each issuesFor(`task.steps[${si}]`) as i (i.field + i.message)}<p class="issue" class:warn-only={i.severity === 'warning'}>{i.field.replace(`task.steps[${si}].`, '')}: {i.message}</p>{/each}
								</div>
							{/if}
						</div>
					{/each}
					<button class="btn btn-primary" onclick={addStep}><Icon name="plus" size={16} /> Добавить шаг</button>
				{/if}

				{#if section !== 'task' && section !== 'quiz' && section !== 'guide'}
					{#each issues.filter((i) => sectionOf(i.field) === section) as i (i.field + i.message)}<p class="issue" class:warn-only={i.severity === 'warning'}>{i.field}: {i.message}</p>{/each}
				{/if}
			</section>
		</div>
	{/if}
</div>

<style>
	.back {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 8px;
		font-size: 13px;
		font-weight: 600;
	}

	.head-actions {
		display: flex;
		gap: 10px;
	}

	.banner {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 16px;
		padding: 12px 16px;
		border-radius: 12px;
		font-size: 14px;
	}

	.warn {
		background: var(--kaki-100);
		color: var(--kaki-700);
	}

	.ok {
		background: var(--ai-100);
		color: var(--ai-700);
	}

	.layout {
		display: grid;
		grid-template-columns: 220px minmax(0, 1fr);
		gap: 24px;
		align-items: start;
	}

	.sections {
		position: sticky;
		top: 20px;
		display: grid;
		gap: 4px;
		padding: 12px;
	}

	.sections button {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 10px 12px;
		border: 0;
		border-radius: 10px;
		background: transparent;
		font-weight: 600;
		font-size: 14px;
		text-align: left;
		cursor: pointer;
	}

	.sections button[aria-pressed='true'] {
		background: var(--paper-sunken);
		box-shadow: inset 3px 0 0 var(--shu-500);
	}

	.summary {
		margin-top: 10px;
		padding: 10px 12px;
		border-top: 1px solid var(--paper-line);
	}

	.bad {
		color: var(--kaki-700);
	}

	.good {
		color: var(--ai-700);
	}

	.body {
		display: grid;
		gap: 16px;
		padding: 28px;
	}

	.grid3 {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 12px;
	}

	fieldset {
		margin: 0;
		padding: 0;
		border: 0;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 8px;
	}

	.row-head {
		display: flex;
		align-items: end;
		justify-content: space-between;
		gap: 12px;
	}

	.grow {
		flex: 1;
		min-width: 0;
	}

	.theory {
		font-size: 13px;
	}

	.preview {
		padding: 20px;
		border-radius: 12px;
		background: var(--paper-base);
	}

	video,
	.start img {
		width: 100%;
		border-radius: 10px;
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.gap {
		margin-top: 14px;
	}

	.question {
		display: grid;
		gap: 10px;
		padding: 16px;
		border-radius: 12px;
		background: var(--paper-base);
	}

	.option {
		display: grid;
		grid-template-columns: auto 1fr auto;
		align-items: center;
		gap: 10px;
	}

	.icon {
		display: grid;
		place-items: center;
		width: 34px;
		height: 34px;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: var(--ink-500);
		cursor: pointer;
	}

	.start {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}

	.start > div {
		display: grid;
		align-content: start;
		gap: 10px;
	}

	.step {
		border-radius: 12px;
		box-shadow: inset 0 0 0 1px var(--paper-line);
	}

	.step.open {
		box-shadow: inset 0 0 0 2px var(--ai-300);
	}

	.step-head {
		display: flex;
		align-items: center;
		gap: 14px;
		width: 100%;
		padding: 14px 16px;
		border: 0;
		background: transparent;
		text-align: left;
		cursor: pointer;
	}

	.step-head .grow {
		display: grid;
	}

	.n {
		display: grid;
		place-items: center;
		flex: none;
		width: 30px;
		height: 30px;
		border-radius: 50%;
		box-shadow: inset 0 0 0 2px var(--ink-900);
		font-weight: 700;
		font-size: 13px;
	}

	.step-body {
		display: grid;
		gap: 12px;
		padding: 0 16px 16px;
	}

	details {
		display: grid;
		gap: 10px;
		padding: 12px;
		border-radius: 10px;
		background: var(--ai-100);
	}

	details[open] > :not(summary) {
		margin-top: 10px;
	}

	summary {
		cursor: pointer;
	}

	.danger {
		color: var(--shu-700);
	}

	.issue {
		font-size: 13px;
		color: var(--kaki-700);
	}

	.warn-only {
		color: var(--ink-500);
	}
</style>
