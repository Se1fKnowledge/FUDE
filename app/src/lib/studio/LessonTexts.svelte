<script lang="ts" module>
	import { t } from '#lib/i18n.svelte.ts';
	export type LessonSection = 'about' | 'theory' | 'quiz';
</script>

<script lang="ts">
	import type { Lesson, Skill } from '../api.ts';
	import Icon from '../components/Icon.svelte';
	import Markdown from '../components/Markdown.svelte';

	// «Урок»: the words of the lesson — its card, theory with a live student view, and quiz.
	let {
		lesson = $bindable(),
		theory = $bindable(),
		section,
		lessonIds = []
	}: { lesson: Lesson; theory: string; section: LessonSection; lessonIds?: [string, string][] } = $props();

	const SKILLS: [Skill, string][] = [
		['tools', t('Инструменты')],
		['line', t('Линия')],
		['form', t('Форма')],
		['light', t('Свет и тень')],
		['color', t('Цвет')],
		['composition', t('Композиция')]
	];
	const LEVELS: [number, string][] = [
		[1, t('Первый шаг')],
		[2, t('Лёгкий')],
		[3, t('Средний')],
		[4, t('Сложный')],
		[5, t('Мастер')]
	];

	function toggleSkill(s: Skill) {
		lesson.skills = lesson.skills.includes(s) ? lesson.skills.filter((x) => x !== s) : [...lesson.skills, s];
	}

	let area: HTMLTextAreaElement | undefined = $state();

	/** Wraps the selection in `before`/`after`, or inserts `block` on its own line. */
	function format(before: string, after = '', block = false) {
		if (!area) return;
		const { selectionStart: a, selectionEnd: b, value } = area;
		const picked = value.slice(a, b);
		const insert = block ? `\n${before}${picked}${after}\n` : `${before}${picked}${after}`;
		theory = value.slice(0, a) + insert + value.slice(b);
		requestAnimationFrame(() => {
			area?.focus();
			area?.setSelectionRange(a + before.length + (block ? 1 : 0), a + before.length + (block ? 1 : 0) + picked.length);
		});
	}
</script>

{#if section === 'about'}
	<div class="sheet">
		<h2>{t('Описание и карточка')}</h2>
		<label class="field">{t('Название')} <input type="text" bind:value={lesson.title} maxlength="120" /></label>
		<label class="field">{t('Кратко — строка на карточке')} <textarea rows="2" bind:value={lesson.summary}></textarea></label>
		<div class="grid3">
			<label class="field">{t('Сложность')}
				<select bind:value={lesson.difficulty}>
					{#each LEVELS as [v, l] (v)}<option value={v}>{v} — {l}</option>{/each}
				</select>
			</label>
			<label class="field">{t('Минут')} <input type="number" min="1" max="600" bind:value={lesson.minutes} /></label>
			<label class="field">{t('Порядок в категории')} <input type="number" min="0" bind:value={lesson.order} /></label>
		</div>
		<fieldset>
			<legend>{t('Стороны навыка — для статистики ученика')}</legend>
			<div class="chips">
				{#each SKILLS as [s, l] (s)}<button type="button" class="chip" aria-pressed={lesson.skills.includes(s)} onclick={() => toggleSkill(s)}>{l}</button>{/each}
			</div>
		</fieldset>
		<label class="field">{t('Открывается после урока')}
			<select value={lesson.requires?.[0] ?? ''} onchange={(e) => (lesson.requires = e.currentTarget.value ? [e.currentTarget.value] : [])}>
				<option value="">{t('Сразу открыт')}</option>
				{#each lessonIds.filter(([id]) => id !== lesson.id) as [id, title] (id)}<option value={id}>{title}</option>{/each}
			</select>
		</label>
		<label class="field">{t('Автор')} <input type="text" value={lesson.author ?? ''} onchange={(e) => (lesson.author = e.currentTarget.value.trim() || undefined)} /></label>
		<label class="field">{t('Что нужно сделать в задании')} <textarea rows="2" bind:value={lesson.task.brief}></textarea></label>
	</div>
{:else if section === 'theory'}
	<div class="split">
		<div class="source">
			<div class="toolbar" role="toolbar" aria-label={t('Оформление текста')}>
				<button onclick={() => format('**', '**')} aria-label={t('Жирный')}><b>{t('Ж')}</b></button>
				<button onclick={() => format('*', '*')} aria-label={t('Курсив')}><i>{t('К')}</i></button>
				<button onclick={() => format('## ', '', true)} aria-label={t('Заголовок')}>{t('Заголовок')}</button>
				<button onclick={() => format('- ', '', true)} aria-label={t('Список')}>{t('Список')}</button>
				<button onclick={() => format(`## ${t('Совет сэнсэя')}\n\n`, '', true)} class="sensei"><span class="kanji">筆</span> {t('Совет сэнсэя')}</button>
			</div>
			<textarea bind:this={area} bind:value={theory} spellcheck="true" aria-label={t('Теория в Markdown')}></textarea>
		</div>
		<div class="view" aria-label={t('Как увидит ученик')}>
			<p class="view-label">{t('Как увидит ученик')}</p>
			<Markdown text={theory} />
		</div>
	</div>
{:else}
	<div class="sheet">
		<h2>{t('Вопросы после теории')}</h2>
		{#each lesson.quiz ?? [] as q, qi (qi)}
			<div class="question">
				<div class="qhead">
					<label class="field grow">{t('Вопрос {n}', { n: qi + 1 })} <input type="text" bind:value={q.question} /></label>
					<button class="icon" onclick={() => lesson.quiz?.splice(qi, 1)} aria-label={t('Удалить вопрос')}><Icon name="trash" size={16} /></button>
				</div>
				{#each q.options as _, oi (oi)}
					<div class="option">
						<input type="radio" name="answer-{qi}" checked={q.answer === oi} onchange={() => (q.answer = oi)} aria-label={t('Верный ответ')} />
						<input type="text" bind:value={q.options[oi]} placeholder={t('Вариант ответа')} />
						<button class="icon" onclick={() => (q.options.splice(oi, 1), (q.answer = Math.min(q.answer, q.options.length - 1)))} aria-label={t('Удалить вариант')}><Icon name="close" size={14} /></button>
					</div>
				{/each}
				<button class="small" onclick={() => q.options.push('')} disabled={q.options.length >= 8}><Icon name="plus" size={14} /> {t('Вариант')}</button>
				<label class="field">{t('Пояснение после ответа')} <input type="text" value={q.explain ?? ''} onchange={(e) => (q.explain = e.currentTarget.value.trim() || undefined)} /></label>
			</div>
		{/each}
		<button class="small" onclick={() => (lesson.quiz ??= []).push({ question: '', options: ['', ''], answer: 0 })}><Icon name="plus" size={16} /> {t('Добавить вопрос')}</button>
	</div>
{/if}

<style>
	.sheet {
		display: flex;
		flex-direction: column;
		gap: 16px;
		width: min(760px, 100%);
		margin: 0 auto;
		padding: 28px 32px 40px;
		border-radius: var(--radius-card);
		background: var(--paper-base);
		color: var(--ink-900);
	}

	h2 {
		margin: 0 0 4px;
		font: 600 32px/1.15 var(--font-display);
	}

	.field,
	legend {
		display: flex;
		flex-direction: column;
		gap: 6px;
		color: var(--ink-500);
		font: 600 13px var(--font-ui);
	}

	fieldset {
		margin: 0;
		padding: 0;
		border: 0;
	}

	legend {
		margin-bottom: 8px;
	}

	input[type='text'],
	input[type='number'],
	textarea,
	select {
		padding: 10px 12px;
		border: 1px solid var(--paper-line);
		border-radius: 8px;
		background: var(--paper-raised);
		color: var(--ink-900);
		font: 15px/1.5 var(--font-ui);
	}

	.grid3 {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 12px;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.split {
		display: grid;
		grid-template-columns: 1fr 1fr;
		height: 100%;
		min-height: 0;
	}

	.source {
		display: flex;
		flex-direction: column;
		gap: 12px;
		min-height: 0;
		padding: 16px 20px;
		border-right: 1px solid var(--sumi-line);
		background: var(--sumi-panel);
	}

	.toolbar {
		display: flex;
		gap: 4px;
		padding: 4px;
		border-radius: 8px;
		background: var(--sumi-bg);
		align-self: flex-start;
	}

	.toolbar button {
		padding: 6px 10px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--sumi-text);
		font: 600 12px var(--font-ui);
		cursor: pointer;
	}

	.toolbar button:hover {
		background: var(--sumi-raised);
	}

	.sensei .kanji {
		color: var(--shu-500);
		font-family: var(--font-kanji);
	}

	.source textarea {
		flex: 1;
		min-height: 0;
		border: 0;
		background: none;
		color: var(--sumi-text);
		font: 500 13px/1.7 var(--font-mono);
		resize: none;
		outline: none;
	}

	.view {
		min-height: 0;
		overflow-y: auto;
		padding: 24px 32px;
		background: var(--paper-base);
		color: var(--ink-900);
	}

	.view-label {
		margin: 0 0 12px;
		color: var(--ink-500);
		font: 700 12px var(--font-ui);
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	.question {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 16px;
		border-radius: var(--radius-m);
		background: var(--paper-raised);
	}

	.qhead,
	.option {
		display: flex;
		gap: 8px;
		align-items: flex-end;
	}

	.option {
		align-items: center;
	}

	.option input[type='text'] {
		flex: 1;
	}

	.grow {
		flex: 1;
	}

	.icon {
		display: grid;
		place-items: center;
		width: 36px;
		height: 36px;
		border: 0;
		border-radius: 8px;
		background: none;
		color: var(--ink-500);
		cursor: pointer;
	}

	.small {
		align-self: flex-start;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		border: 1px solid var(--paper-line);
		border-radius: 8px;
		background: var(--paper-raised);
		color: var(--ink-900);
		font: 600 13px var(--font-ui);
		cursor: pointer;
	}
</style>
