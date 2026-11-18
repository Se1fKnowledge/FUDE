// The lesson-editor tour: an author builds a small lesson of their own, one lit control at a
// time. Each step names the control to light, says what it is for and sets a small task; the
// task is done when the draft itself shows it (a title changed, a ghost line drawn). Steps that
// change something a student will see also say which student's view to show beside the card.
import { t } from '#lib/i18n.svelte.ts';

/** What the tour reads from the studio to tell whether a step is done. */
export interface TourSnap {
	mode: 'lesson' | 'guide' | 'task';
	section: string;
	tab: 'text' | 'hint' | 'checks';
	tool: string;
	title: string;
	summary: string;
	difficulty: number;
	theoryWords: number;
	stepText: string;
	hintText: string;
	/** Ghost lines of the selected task step. */
	shapes: number;
	/** Checks of the selected task step. */
	checks: number;
	/** Successful saves since the studio opened. */
	saves: number;
}

/** A place in the studio: its mode, the section of «Урок», the inspector tab, the canvas tool. */
export interface TourPlace {
	mode?: TourSnap['mode'];
	section?: string;
	tab?: TourSnap['tab'];
	tool?: string;
}

/** The student's view a step shows beside its card. */
export type StudentView = 'none' | 'card' | 'row' | 'task' | 'hint' | 'check';

export interface TourStep {
	id: string;
	/** Index into `CHAPTERS`. */
	chapter: number;
	title: string;
	body: string;
	task: string;
	/** CSS selector of the control to light. */
	target: string;
	/** Where in the studio the control lives; the tour opens that place when the step is shown,
	 * so a step reached by «Пропустить» or after a break is never out of sight. */
	place?: TourPlace;
	view: StudentView;
	/** Whether the draft shows the task is done; `start` is the draft when the step was shown. */
	done: (now: TourSnap, start: TourSnap) => boolean;
}

export const words = (text: string) => text.trim().split(/\s+/).filter(Boolean).length;

const at = (name: string) => `[data-tour="${name}"]`;

export const CHAPTERS: string[] = [t('Карточка урока'), t('Теория'), t('Задание'), t('Подсказка'), t('Проверка'), t('Глазами ученика')];

export const STEPS: TourStep[] = [
	{
		id: 'mode-lesson',
		chapter: 0,
		title: t('Три режима студии'),
		body: t('В «Уроке» — тексты: описание, теория и вопросы. В «Гайде» — показ приёма. В «Задании» — шаги, подсказки и проверки.'),
		task: t('Откройте режим «Урок»'),
		target: at('mode-lesson'),
		view: 'none',
		done: (s) => s.mode === 'lesson'
	},
	{
		id: 'about',
		chapter: 0,
		title: t('Описание и карточка'),
		body: t('Здесь то, что ученик видит до начала урока: название, строка описания, сложность.'),
		task: t('Откройте «Описание и карточка»'),
		target: at('section-about'),
		place: { mode: 'lesson' },
		view: 'none',
		done: (s) => s.mode === 'lesson' && s.section === 'about'
	},
	{
		id: 'title',
		chapter: 0,
		title: t('Название урока'),
		body: t('Название стоит на карточке в каталоге и наверху страницы урока. Хорошее название говорит, чему ученик научится.'),
		task: t('Назовите урок по-своему'),
		target: at('field-title'),
		place: { mode: 'lesson', section: 'about' },
		view: 'card',
		done: (s, start) => s.title.trim().length >= 3 && s.title.trim() !== start.title.trim()
	},
	{
		id: 'summary',
		chapter: 0,
		title: t('Краткое описание'),
		body: t('Одно предложение: о чём урок и что получится в конце. Ученик читает его в списке уроков, под названием.'),
		task: t('Напишите краткое описание — не короче десяти знаков'),
		target: at('field-summary'),
		place: { mode: 'lesson', section: 'about' },
		view: 'row',
		done: (s, start) => s.summary.trim().length >= 10 && s.summary.trim() !== start.summary.trim()
	},
	{
		id: 'difficulty',
		chapter: 0,
		title: t('Сложность'),
		body: t('На карточке сложность показана мазками туши: от одного до пяти. По ней же считается опыт за урок.'),
		task: t('Выберите другую сложность'),
		target: at('field-difficulty'),
		place: { mode: 'lesson', section: 'about' },
		view: 'card',
		done: (s, start) => s.difficulty !== start.difficulty
	},
	{
		id: 'section-theory',
		chapter: 1,
		title: t('Теория'),
		body: t('Теорию ученик читает после гайда и перед заданием.'),
		task: t('Откройте раздел «Теория»'),
		target: at('section-theory'),
		place: { mode: 'lesson' },
		view: 'none',
		done: (s) => s.mode === 'lesson' && s.section === 'theory'
	},
	{
		id: 'theory',
		chapter: 1,
		title: t('Текст теории'),
		body: t('Пишется в Markdown: строка с «## » становится заголовком, строки с «- » — списком. Кнопки над полем ставят разметку сами. Справа — страница, какой её увидит ученик.'),
		task: t('Добавьте к теории хотя бы десять слов'),
		target: at('theory'),
		place: { mode: 'lesson', section: 'theory' },
		view: 'none',
		done: (s, start) => s.theoryWords >= start.theoryWords + 10
	},
	{
		id: 'mode-task',
		chapter: 2,
		title: t('Задание'),
		body: t('Задание — то, что ученик делает в PhotoCraft. Оно состоит из шагов, у каждого шага своя подсказка и свои проверки.'),
		task: t('Откройте режим «Задание»'),
		target: at('mode-task'),
		view: 'none',
		done: (s) => s.mode === 'task'
	},
	{
		id: 'tab-text',
		chapter: 2,
		title: t('Инспектор шага'),
		body: t('Справа — выбранный шаг. У него три вкладки: текст, подсказка и проверки.'),
		task: t('Откройте вкладку «Текст»'),
		target: at('tab-text'),
		place: { mode: 'task' },
		view: 'none',
		done: (s) => s.mode === 'task' && s.tab === 'text'
	},
	{
		id: 'step-text',
		chapter: 2,
		title: t('Текст шага'),
		body: t('Шаг — одно действие ученика. Пишите так, чтобы было ясно, что сделать и где.'),
		task: t('Перепишите текст шага своими словами'),
		target: at('step-text'),
		place: { mode: 'task', tab: 'text' },
		view: 'task',
		done: (s, start) => s.stepText.trim().length >= 8 && s.stepText.trim() !== start.stepText.trim()
	},
	{
		id: 'tab-hint',
		chapter: 3,
		title: t('Подсказка шага'),
		body: t('Подсказку ученик открывает сам, кнопкой или клавишей F1. Каждое открытие видно в его статистике.'),
		task: t('Откройте вкладку «Подсказка»'),
		target: at('tab-hint'),
		place: { mode: 'task' },
		view: 'none',
		done: (s) => s.mode === 'task' && s.tab === 'hint'
	},
	{
		id: 'hint-text',
		chapter: 3,
		title: t('Текст подсказки'),
		body: t('Скажите, как сделать шаг: каким инструментом и с чего начать.'),
		task: t('Напишите подсказку — не короче восьми знаков'),
		target: at('hint-text'),
		place: { mode: 'task', tab: 'hint' },
		view: 'hint',
		done: (s) => s.hintText.trim().length >= 8
	},
	{
		id: 'tool-ellipse',
		chapter: 3,
		title: t('Линии-призраки'),
		body: t('Подсказка может нарисовать на холсте ученика полупрозрачную фигуру: линию, стрелку, овал или прямоугольник.'),
		task: t('Возьмите инструмент «Овал»'),
		target: at('tool-ellipse'),
		place: { mode: 'task', tab: 'hint' },
		view: 'none',
		done: (s) => s.tool === 'ellipse'
	},
	{
		id: 'ghost',
		chapter: 3,
		title: t('Овал на холсте'),
		body: t('Нажмите на холсте и тяните: получится рамка овала. Ученик увидит его там же, на отдельном слое.'),
		task: t('Растяните овал там, где ученик должен рисовать'),
		target: at('canvas'),
		place: { mode: 'task', tab: 'hint', tool: 'ellipse' },
		view: 'hint',
		done: (s, start) => s.shapes > start.shapes
	},
	{
		id: 'tool-region',
		chapter: 4,
		title: t('Проверка по холсту'),
		body: t('Приложение само смотрит на холст ученика. Область проверки говорит, в какую часть листа смотреть.'),
		task: t('Возьмите инструмент «Область проверки»'),
		target: at('tool-region'),
		place: { mode: 'task' },
		view: 'none',
		done: (s) => s.tool === 'region'
	},
	{
		id: 'region',
		chapter: 4,
		title: t('Область проверки'),
		body: t('Растяните рамку вокруг овала. Появится проверка «Закрашена область»: шаг засчитается, когда ученик закрасит в ней достаточно.'),
		task: t('Растяните область проверки на холсте'),
		target: at('canvas'),
		place: { mode: 'task', tool: 'region' },
		view: 'check',
		done: (s, start) => s.checks > start.checks
	},
	{
		id: 'save',
		chapter: 5,
		title: t('Сохранение'),
		body: t('Черновик сохраняется в вашу папку уроков. Урок с недочётами тоже сохранится: список «Готовность урока» слева покажет, чего не хватает.'),
		task: t('Сохраните урок'),
		target: at('save'),
		view: 'none',
		done: (s, start) => s.saves > start.saves
	},
	{
		id: 'preview',
		chapter: 5,
		title: t('Глазами ученика'),
		body: t('Откроется настоящий экран ученика с PhotoCraft: шаги, подсказка с вашим овалом и автопроверка. Прогресс при этом не записывается.'),
		task: t('Нажмите «Глазами ученика»'),
		target: at('preview'),
		view: 'none',
		done: () => false
	}
];

const KEY = 'fude-studio-tour';

/** Where the author stopped, per draft folder. */
export function savedIndex(dir: string): number {
	try {
		const saved = JSON.parse(localStorage.getItem(KEY) ?? 'null') as { dir: string; index: number } | null;
		return saved && saved.dir === dir && Number.isInteger(saved.index) ? Math.min(Math.max(saved.index, 0), STEPS.length - 1) : 0;
	} catch {
		return 0;
	}
}

export function rememberIndex(dir: string, index: number): void {
	try {
		localStorage.setItem(KEY, JSON.stringify({ dir, index }));
	} catch {
		// Private mode or blocked storage: the tour starts over next time.
	}
}

/** The author reached «Глазами ученика»: the tour is walked to its end. */
export function finish(dir: string): void {
	try {
		localStorage.setItem(KEY, JSON.stringify({ dir, index: STEPS.length - 1, done: true }));
	} catch {
		// As in `rememberIndex`.
	}
}

/** Whether the tour was walked to its end; starting it again clears this. */
export function finished(): boolean {
	try {
		return (JSON.parse(localStorage.getItem(KEY) ?? 'null') as { done?: boolean } | null)?.done === true;
	} catch {
		return false;
	}
}

/** The draft the tour last worked on, to go on with it instead of making another. */
export function savedDir(): string | null {
	try {
		const saved = JSON.parse(localStorage.getItem(KEY) ?? 'null') as { dir: string } | null;
		return saved?.dir ?? null;
	} catch {
		return null;
	}
}

interface Box {
	x: number;
	y: number;
	w: number;
	h: number;
}

/** Where the card column goes: beside the lit control without covering it — right, left,
 * below, above — and into the bottom right corner when the control leaves no room (the canvas). */
export function columnPlace(view: { w: number; h: number }, hole: Box | null, size: { w: number; h: number }, gap = 22): { x: number; y: number } {
	const pad = 12;
	const cx = (x: number) => Math.min(Math.max(x, pad), Math.max(pad, view.w - size.w - pad));
	const cy = (y: number) => Math.min(Math.max(y, pad), Math.max(pad, view.h - size.h - pad));
	if (!hole) return { x: cx((view.w - size.w) / 2), y: cy((view.h - size.h) / 2) };
	const fits = (p: { x: number; y: number }) => p.x >= pad && p.y >= pad && p.x + size.w <= view.w - pad && p.y + size.h <= view.h - pad;
	const y = cy(hole.y + hole.h / 2 - size.h / 2);
	const x = cx(hole.x + hole.w / 2 - size.w / 2);
	const right = { x: hole.x + hole.w + gap, y };
	const left = { x: hole.x - gap - size.w, y };
	const below = { x, y: hole.y + hole.h + gap };
	const above = { x, y: hole.y - gap - size.h };
	// A control in a bar along the top or bottom has neighbours beside it: go below or above first.
	const inBar = hole.h <= 64 && (hole.y < 90 || hole.y + hole.h > view.h - 90);
	for (const p of inBar ? [below, above, right, left] : [right, left, below, above]) {
		if (fits(p)) return p;
	}
	return { x: view.w - size.w - pad, y: view.h - size.h - pad };
}
