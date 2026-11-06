<script lang="ts">
	import { t } from '#lib/i18n.svelte.ts';
	import { onDestroy, onMount } from 'svelte';
	import { beforeNavigate, goto } from '$app/navigation';
	import { page } from '$app/state';
	import { open } from '@tauri-apps/plugin-dialog';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { api, errorText, type CheckProof, type DraftDetail, type GuideEvent, type GuideScript, type Issue, type Lesson, type LessonCard, type RecordTick, type Region, type Shape, type Step } from '#lib/api.ts';
	import { defaultCheck, stepId } from '#lib/checks.ts';
	import Icon from '#lib/components/Icon.svelte';
	import LessonTile from '#lib/components/LessonTile.svelte';
	import { editorSlot, preferEmbedded } from '#lib/editor-slot.ts';
	import { count } from '#lib/format.ts';
	import StudioCanvas, { type CanvasTool, type DemoStroke, type RegionMark } from '#lib/studio/StudioCanvas.svelte';
	import GuidePanel from '#lib/studio/GuidePanel.svelte';
	import GuideTimeline from '#lib/studio/GuideTimeline.svelte';
	import InterfaceMap from '#lib/studio/InterfaceMap.svelte';
	import LessonTexts, { type LessonSection } from '#lib/studio/LessonTexts.svelte';
	import TaskPanel, { type TaskTab } from '#lib/studio/TaskPanel.svelte';
	import { captionAt, emptyScript, frameAt, insertStep, isStroke, moveStep, parseScript, removeStep, splitStep, strokePoints, strokeStep, taskStepStart, times, type BrushSettings } from '#lib/studio/timeline.ts';
	import { History } from '#lib/studio/history.ts';

	const dir = $derived(page.url.searchParams.get('dir') ?? '');

	// ---------------------------------------------------------------- the draft
	let draft = $state<DraftDetail | null>(null);
	let lesson = $state<Lesson | null>(null);
	let theory = $state('');
	let script = $state<GuideScript>({ format: 1, steps: [] });
	let issues = $state<Issue[]>([]);
	let taskImage = $state<string | null>(null);
	let referenceImage = $state<string | null>(null);
	let size = $state({ w: 1200, h: 800 });
	let error = $state('');
	let saving = $state(false);
	let savedAt = $state('');
	let dirty = $state(false);
	let loaded = false;

	type Mode = 'lesson' | 'guide' | 'task';
	let mode = $state<Mode>('task');
	let section = $state<LessonSection>('theory');

	async function load() {
		try {
			const d = await api.draft(dir);
			const editable = structuredClone(d.lesson);
			editable.quiz ??= [];
			editable.requires ??= [];
			draft = d;
			lesson = editable;
			theory = d.theory;
			script = parseScript(d.guide) ?? emptyScript(editable.task);
			issues = d.issues;
			taskImage = d.taskImage;
			referenceImage = d.referenceImage;
			await measure();
			if (!editable.task.steps.length) mode = 'lesson';
			history.reset(snapshot());
			loaded = true;
		} catch (e) {
			error = errorText(e);
		}
	}

	/** The picture's size in pixels: what checks, ghosts and strokes are measured in. */
	async function measure() {
		if (lesson?.task.canvas) {
			size = { w: lesson.task.canvas.width, h: lesson.task.canvas.height };
		} else if (taskImage) {
			const img = new Image();
			img.src = taskImage;
			await img.decode().catch(() => {});
			if (img.naturalWidth) size = { w: img.naturalWidth, h: img.naturalHeight };
		}
	}

	// ---------------------------------------------------------------- undo and redo
	const history = new History();
	let canUndo = $state(false);
	let canRedo = $state(false);
	let restoring = false;
	let historyTimer: ReturnType<typeof setTimeout> | null = null;
	const snapshot = () => JSON.stringify({ lesson: $state.snapshot(lesson), theory, script: $state.snapshot(script) });

	$effect(() => {
		// Any edit after loading marks the draft unsaved and, once edits pause, becomes an undo step.
		const now = JSON.stringify([lesson, theory, script]);
		void now;
		if (!loaded) return;
		dirty = true;
		if (restoring) {
			restoring = false;
			return;
		}
		if (historyTimer) clearTimeout(historyTimer);
		historyTimer = setTimeout(() => {
			history.record(snapshot());
			canUndo = history.canUndo;
			canRedo = history.canRedo;
		}, 350);
	});

	function restore(text: string | null) {
		if (!text) return;
		const v = JSON.parse(text) as { lesson: Lesson; theory: string; script: GuideScript };
		restoring = true;
		lesson = v.lesson;
		theory = v.theory;
		script = v.script;
		canUndo = history.canUndo;
		canRedo = history.canRedo;
		selectedShape = -1;
		selectedRegion = null;
		if (lesson && stepIndex >= lesson.task.steps.length) stepIndex = Math.max(0, lesson.task.steps.length - 1);
		if (guideStep >= script.steps.length) guideStep = script.steps.length - 1;
	}

	function undo() {
		if (historyTimer) {
			clearTimeout(historyTimer);
			historyTimer = null;
			history.record(snapshot());
		}
		restore(history.undo());
	}

	function redo() {
		restore(history.redo());
	}

	function cleanLesson(): Lesson {
		const clean = $state.snapshot(lesson) as Lesson;
		if (!clean.requires?.length) delete clean.requires;
		if (!clean.quiz?.length) delete clean.quiz;
		for (const s of clean.task.steps) {
			if (s.hint && !s.hint.text.trim() && !s.hint.target && !s.hint.shapes?.length) delete s.hint;
			if (s.hint && !s.hint.shapes?.length) delete s.hint.shapes;
		}
		return clean;
	}

	async function save(): Promise<boolean> {
		if (!lesson || saving) return false;
		saving = true;
		error = '';
		try {
			const s = $state.snapshot(script) as GuideScript;
			const guideText = s.steps.length ? JSON.stringify(s, null, 2) : lesson.guide.script ? '' : null;
			issues = await api.draftSave(dir, cleanLesson(), theory, guideText);
			if (!s.steps.length) delete lesson.guide.script;
			else lesson.guide.script ??= 'guide.json';
			dirty = false;
			savedAt = new Date().toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' });
			return true;
		} catch (e) {
			error = errorText(e);
			return false;
		} finally {
			saving = false;
			queueMicrotask(() => (dirty = false));
		}
	}

	async function attach(target: 'open' | 'reference' | 'video') {
		if (!lesson) return;
		const video = target === 'video';
		const source = await open({ multiple: false, filters: [video ? { name: t('Видео'), extensions: ['mp4', 'webm'] } : { name: t('Картинка'), extensions: ['png', 'jpg', 'jpeg', 'webp'] }] });
		if (typeof source !== 'string') return;
		try {
			const rel = await api.attach(dir, video ? 'video' : 'image', source);
			if (target === 'video') lesson.guide.video = rel;
			if (target === 'reference') {
				lesson.task.reference = rel;
				referenceImage = await api.picture(dir, rel);
			}
			if (target === 'open') {
				lesson.task.open = rel;
				delete lesson.task.canvas;
				taskImage = await api.picture(dir, rel);
				if (!script.steps.length) script = emptyScript(lesson.task);
				await measure();
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
		measure();
	}

	// ---------------------------------------------------------------- task mode
	let stepIndex = $state(0);
	let tab = $state<TaskTab>('hint');
	let tool = $state<CanvasTool>('select');
	let selectedShape = $state(-1);
	let selectedRegion = $state<string | null>(null);
	let showOthers = $state(false);
	let showRegions = $state(true);
	let showHints = $state(true);
	let refOpacity = $state(0);
	let proofs = $state<CheckProof[]>([]);
	let zoom = $state(1);
	let wrapW = $state(800);
	let wrapH = $state(600);
	const fit = $derived(Math.max(0.05, Math.min((wrapW - 40) / size.w, (wrapH - 40) / size.h)));
	const scale = $derived(fit * zoom);
	const ZOOMS = [0.5, 0.75, 1, 1.5, 2, 3, 4];
	const zoomBy = (dir: number) => (zoom = dir > 0 ? (ZOOMS.find((z) => z > zoom + 1e-6) ?? 4) : ([...ZOOMS].reverse().find((z) => z < zoom - 1e-6) ?? 0.5));
	let wrap: HTMLDivElement | undefined = $state();
	let pan: { x: number; y: number; left: number; top: number } | null = null;

	function wheel(e: WheelEvent) {
		if (!e.ctrlKey) return;
		e.preventDefault();
		zoomBy(e.deltaY < 0 ? 1 : -1);
	}

	function panDown(e: PointerEvent) {
		if (activeTool !== 'hand' || !wrap) return;
		pan = { x: e.clientX, y: e.clientY, left: wrap.scrollLeft, top: wrap.scrollTop };
		wrap.setPointerCapture(e.pointerId);
	}

	function panMove(e: PointerEvent) {
		if (!pan || !wrap) return;
		wrap.scrollLeft = pan.left - (e.clientX - pan.x);
		wrap.scrollTop = pan.top - (e.clientY - pan.y);
	}
	let mapFor = $state<'hint' | 'guide' | null>(null);

	const step = $derived<Step | null>(lesson?.task.steps[stepIndex] ?? null);
	const REGION_COLORS = ['#2F5D8C', '#E8893B', '#C29A4E', '#6B7FA3', '#8F4E16'];

	/** Letters and colours of the regions of each check of the step: A, B, C… */
	const regionMarks = $derived.by(() => {
		const marks: (RegionMark & { check: number; field: 'region' | 'than' })[] = [];
		step?.checks.forEach((c, ci) => {
			const add = (field: 'region' | 'than', region: Region, name: string) => {
				const n = marks.length;
				marks.push({ key: `${ci}:${field}`, label: String.fromCharCode(65 + n), region, color: REGION_COLORS[n % REGION_COLORS.length], name, check: ci, field });
			};
			if ('region' in c && c.region) add('region', c.region, c.kind === 'darker' ? t('темнее') : t('проверка'));
			if (c.kind === 'darker') add('than', c.than, t('светлее'));
		});
		return marks;
	});
	const lettersOf = (ci: number) => {
		const m = regionMarks.filter((r) => r.check === ci);
		return { letters: m.map((r) => r.label), colors: m.map((r) => r.color), keys: m.map((r) => r.key) };
	};

	function setShapes(shapes: Shape[]) {
		if (!lesson || !step) return;
		const s = lesson.task.steps[stepIndex];
		s.hint = { ...(s.hint ?? { text: '' }), shapes };
		tab = 'hint';
	}

	function setRegion(key: string, region: Region) {
		const m = regionMarks.find((r) => r.key === key);
		if (!lesson || !m) return;
		const c = lesson.task.steps[stepIndex].checks[m.check] as Record<string, unknown>;
		c[m.field] = region;
	}

	function drawRegion(region: Region) {
		if (!lesson) return;
		const s = lesson.task.steps[stepIndex];
		if (selectedRegion) {
			setRegion(selectedRegion, region);
		} else {
			s.checks.push({ ...defaultCheck('coverage'), region } as Step['checks'][number]);
			selectedRegion = `${s.checks.length - 1}:region`;
		}
		tab = 'checks';
		tool = 'select';
	}

	function addStep() {
		if (!lesson) return;
		lesson.task.steps.push({ id: stepId(lesson.task.steps.map((s) => s.id)), text: '', checks: [] });
		stepIndex = lesson.task.steps.length - 1;
		tab = 'text';
	}

	function moveTaskStep(by: number) {
		if (!lesson) return;
		const steps = lesson.task.steps;
		const j = stepIndex + by;
		if (j < 0 || j >= steps.length) return;
		[steps[stepIndex], steps[j]] = [steps[j], steps[stepIndex]];
		stepIndex = j;
	}

	function removeTaskStep() {
		if (!lesson) return;
		lesson.task.steps.splice(stepIndex, 1);
		stepIndex = Math.max(0, Math.min(stepIndex, lesson.task.steps.length - 1));
	}

	// Live check: re-proved shortly after the checks or pictures change.
	let proveTimer: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		if (!lesson) return;
		const snapshot = JSON.stringify(lesson.task);
		void snapshot;
		if (proveTimer) clearTimeout(proveTimer);
		proveTimer = setTimeout(async () => {
			try {
				proofs = await api.prove(dir, cleanLesson());
			} catch {
				proofs = [];
			}
		}, 500);
	});

	// ---------------------------------------------------------------- guide mode
	let guideStep = $state(-1);
	let playhead = $state(0);
	let playing = $state(false);
	let guideTool = $state<CanvasTool>('select');
	let brush = $state<BrushSettings>({ size: 40, hardness: 0.2, opacity: 0.5, color: '#3B3A40' });
	let raf = 0;
	const total = $derived(times(script.steps).total);

	function play() {
		if (playing) {
			playing = false;
			cancelAnimationFrame(raf);
			return;
		}
		if (playhead >= total) playhead = 0;
		playing = true;
		let last = performance.now();
		const tick = (now: number) => {
			playhead = Math.min(total, playhead + (now - last));
			last = now;
			guideStep = frameAt(script.steps, playhead).step;
			if (playhead >= total) playing = false;
			else raf = requestAnimationFrame(tick);
		};
		raf = requestAnimationFrame(tick);
	}

	function selectGuideStep(i: number) {
		guideStep = i;
		const { starts } = times(script.steps);
		if (i >= 0) playhead = starts[i] + (script.steps[i].pause ?? 600) - 1;
	}

	/** Strokes drawn by the guide up to the playhead, the current one partly. */
	const demo = $derived.by<DemoStroke[]>(() => {
		if (mode !== 'guide') return [];
		const { step: now, progress } = frameAt(script.steps, playhead);
		const out: DemoStroke[] = [];
		script.steps.forEach((s, i) => {
			if (i > now) return;
			for (const c of s.calls.filter(isStroke)) {
				const p = (c.params.params ?? {}) as { size?: number; color?: string; opacity?: number };
				out.push({ points: strokePoints(c), size: p.size ?? 30, color: p.color ?? '#000000', opacity: Math.max(0.15, p.opacity ?? 1), progress: i === now ? progress : 1 });
			}
		});
		return out;
	});

	function addStroke(points: number[][]) {
		const at = guideStep >= 0 ? guideStep + 1 : script.steps.length;
		script.steps = insertStep(script.steps, at, strokeStep(points, $state.snapshot(brush) as BrushSettings));
		selectGuideStep(at);
	}

	function addCaption() {
		const at = guideStep >= 0 ? guideStep + 1 : script.steps.length;
		script.steps = insertStep(script.steps, at, { caption: t('Новая подпись'), calls: [], pause: 1500 });
		selectGuideStep(at);
	}

	function splitAtPlayhead() {
		const { step: i } = frameAt(script.steps, playhead);
		if (i < 0) return;
		const before = script.steps.length;
		script.steps = splitStep(script.steps, i, playhead - times(script.steps).starts[i]);
		note = script.steps.length > before ? t('Шаг {n} разрезан на два.', { n: i + 1 }) : t('Здесь резать нечего: шаг слишком короткий.');
		selectGuideStep(i);
	}

	function moveGuideStep(by: number) {
		const to = guideStep + by;
		if (guideStep < 0 || to < 0 || to >= script.steps.length) return;
		script.steps = moveStep(script.steps, guideStep, to);
		selectGuideStep(to);
	}

	function removeGuideStep() {
		if (guideStep < 0) return;
		script.steps = removeStep(script.steps, guideStep);
		guideStep = Math.min(guideStep, script.steps.length - 1);
	}

	function ghostFromGuide(taskStep: number, shapes: Shape[]) {
		if (!lesson) return;
		const s = lesson.task.steps[taskStep];
		s.hint = { ...(s.hint ?? { text: '' }), shapes: [...(s.hint?.shapes ?? []), ...shapes] };
		note = t('Линия добавлена в подсказку шага {n}.', { n: taskStep + 1 });
	}

	// ---------------------------------------------------------------- PhotoCraft
	type Live = 'record' | 'hint' | 'guide' | null;
	let live = $state<Live>(null);
	let liveNote = $state('');
	let tick = $state<RecordTick | null>(null);
	let keepPauses = $state(false);
	let keepUndo = $state(false);
	let note = $state('');
	let unlisten: UnlistenFn[] = [];

	async function openLive(kind: Live) {
		live = kind;
		await api.editorLayout(preferEmbedded()).catch(() => {});
	}

	async function closeLive() {
		if (live === 'record') await stopRecording();
		if (live === 'guide') await api.guideStop().catch(() => {});
		live = null;
		liveNote = '';
	}

	async function showHintInEditor() {
		if (!lesson) return;
		await openLive('hint');
		liveNote = t('Открываем задание в PhotoCraft…');
		try {
			const r = await api.hintPreview(dir, cleanLesson(), stepIndex);
			liveNote = t('Подсказка шага {n}: {lines} на слое «FUDE · подсказка».', { n: stepIndex + 1, lines: count(r.shapes, ['линия', 'линии', 'линий']) });
		} catch (e) {
			liveNote = errorText(e);
		}
	}

	async function playInEditor() {
		if (!script.steps.length) return;
		if (!(await save())) return;
		await openLive('guide');
		liveNote = t('Открываем PhotoCraft…');
		try {
			await api.guidePlayDraft(dir);
		} catch (e) {
			liveNote = errorText(e);
		}
	}

	async function toggleRecording() {
		if (live === 'record') {
			await stopRecording();
			live = null;
			return;
		}
		if (!lesson) return;
		await openLive('record');
		tick = null;
		liveNote = t('Открываем задание в PhotoCraft…');
		try {
			await api.recordStart(dir, cleanLesson());
			liveNote = t('Идёт запись: рисуйте в PhotoCraft — FUDE пишет каждое действие.');
		} catch (e) {
			liveNote = errorText(e);
		}
	}

	async function stopRecording() {
		try {
			const r = await api.recordStop(keepPauses, keepUndo);
			const at = guideStep >= 0 ? guideStep + 1 : script.steps.length;
			let steps = script.steps;
			r.steps.forEach((s, i) => (steps = insertStep(steps, at + i, s)));
			script.steps = steps;
			note = r.steps.length
				? t('Записано: {steps}, {strokes}.', { steps: count(r.steps.length, ['шаг', 'шага', 'шагов']), strokes: count(r.strokes, ['штрих', 'штриха', 'штрихов']) }) + (r.skipped.length ? ' ' + t('Пропущено (гайду нельзя): {list}.', { list: r.skipped.join(', ') }) : '') + ' ' + t('Добавьте подписи к сценам.')
				: t('Ничего не записано.');
			if (r.steps.length) selectGuideStep(at);
		} catch (e) {
			note = errorText(e);
		}
	}

	async function preview() {
		if (!lesson || !(await save())) return;
		goto(`/practice/${encodeURIComponent(lesson.id)}?preview=${encodeURIComponent(dir)}`);
	}

	// ---------------------------------------------------------------- keys and lifecycle
	function onkeydown(e: KeyboardEvent) {
		const typing = (e.target as HTMLElement | null)?.closest?.('input, textarea, select, [contenteditable="true"]');
		if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
			e.preventDefault();
			save();
			return;
		}
		if (typing || live) return;
		if ((e.ctrlKey || e.metaKey) && (e.key.toLowerCase() === 'z' || e.key.toLowerCase() === 'y')) {
			e.preventDefault();
			if (e.key.toLowerCase() === 'y' || e.shiftKey) redo();
			else undo();
			return;
		}
		if ((e.ctrlKey || e.metaKey) && (e.key === '=' || e.key === '+' || e.key === '-' || e.key === '0')) {
			e.preventDefault();
			if (e.key === '0') zoom = 1;
			else zoomBy(e.key === '-' ? -1 : 1);
			return;
		}
		if (mode === 'task') {
			const keys: Record<string, CanvasTool> = { v: 'select', h: 'hand', b: 'path', l: 'line', a: 'arrow', o: 'ellipse', r: 'rect', m: 'region', e: 'eraser' };
			const t = keys[e.key.toLowerCase()];
			if (t && !e.ctrlKey && !e.metaKey) tool = t;
			if ((e.key === 'Delete' || e.key === 'Backspace') && selectedShape >= 0 && step?.hint?.shapes) {
				setShapes(step.hint.shapes.filter((_, i) => i !== selectedShape));
				selectedShape = -1;
			}
		}
		if (mode === 'guide') {
			if (e.key === ' ') {
				e.preventDefault();
				play();
			}
			if (e.key.toLowerCase() === 'b') guideTool = 'brush';
			if (e.key.toLowerCase() === 'v') guideTool = 'select';
			if (e.key.toLowerCase() === 'h') guideTool = 'hand';
		}
	}

	// Leaving with unsaved edits asks first: the studio keeps nothing it has not saved.
	beforeNavigate(({ cancel }) => {
		if (dirty && !saving && !confirm(t('В уроке есть несохранённые изменения. Уйти без сохранения?'))) cancel();
	});

	onMount(async () => {
		await load();
		unlisten.push(await listen<RecordTick>('guide-record', (e) => (tick = e.payload)));
		unlisten.push(
			await listen<GuideEvent>('guide', (e) => {
				if (live !== 'guide') return;
				const g = e.payload;
				liveNote = g.state === 'step' ? t('Шаг {n} из {of}: {caption}', { n: g.index + 1, of: g.total, caption: g.caption }) : g.state === 'done' ? t('Гайд закончился.') : g.state === 'error' ? (g.message ?? t('Ошибка')) : g.caption;
			})
		);
	});

	onDestroy(() => {
		cancelAnimationFrame(raf);
		for (const u of unlisten) u();
		if (live === 'record') api.recordStop().catch(() => {});
		api.editorClose().catch(() => {});
	});

	// ---------------------------------------------------------------- readiness
	const errors = $derived(issues.filter((i) => i.severity === 'error'));
	const badProofs = $derived(proofs.filter((p) => p.verdict === 'passesUntouched' || p.verdict === 'failsReference'));
	const readiness = $derived.by(() => {
		if (!lesson) return [];
		const list: { ok: boolean | null; text: string; go?: () => void }[] = [];
		const toLesson = (s: LessonSection) => () => ((mode = 'lesson'), (section = s));
		const texts = [lesson.summary.trim() ? '' : t('описание'), theory.trim() ? '' : t('теория')].filter(Boolean);
		list.push(
			texts.length
				? { ok: false, text: t('Не заполнено: {list}', { list: texts.join(', ') }), go: toLesson(lesson.summary.trim() ? 'theory' : 'about') }
				: { ok: true, text: t('Описание и теория') + (lesson.quiz?.length ? `, ${count(lesson.quiz.length, ['вопрос', 'вопроса', 'вопросов'])}` : '') }
		);
		list.push(
			script.steps.length || lesson.guide.video
				? { ok: true, text: script.steps.length ? t('Гайд: {steps}, {sec} с', { steps: count(script.steps.length, ['шаг', 'шага', 'шагов']), sec: Math.round(total / 100) / 10 }) : t('Видеогайд') }
				: { ok: false, text: t('Нет гайда: запишите его или загрузите видео'), go: () => (mode = 'guide') }
		);
		list.push(lesson.task.steps.length ? { ok: true, text: t('Задание: {steps}', { steps: count(lesson.task.steps.length, ['шаг', 'шага', 'шагов']) }) } : { ok: false, text: t('В задании нет шагов'), go: () => ((mode = 'task'), addStep()) });
		for (const p of badProofs.slice(0, 3)) {
			const i = lesson.task.steps.findIndex((s) => s.id === p.step);
			list.push({
				ok: false,
				text: p.verdict === 'passesUntouched' ? t('Шаг {n}: проверка проходит и на пустом задании', { n: i + 1 }) : t('Шаг {n}: проверка не проходит на эталоне', { n: i + 1 }),
				go: () => ((mode = 'task'), (stepIndex = Math.max(0, i)), (tab = 'checks'))
			});
		}
		if (errors.length) list.push({ ok: false, text: t('{n} в файле урока — пока не в каталоге', { n: count(errors.length, ['ошибка', 'ошибки', 'ошибок']) }), go: toLesson('about') });
		return list;
	});

	const card = $derived<LessonCard | null>(
		lesson && draft
			? {
					id: lesson.id,
					title: lesson.title || t('Без названия'),
					summary: lesson.summary,
					category: draft.category.id,
					categoryTitle: draft.category.title,
					kanji: draft.category.kanji ?? null,
					accent: draft.category.accent,
					difficulty: lesson.difficulty,
					difficultyLabel: ['', t('Первый шаг'), t('Лёгкий'), t('Средний'), t('Сложный'), t('Мастер')][lesson.difficulty] ?? '',
					minutes: lesson.minutes,
					number: lesson.order,
					of: 0,
					skills: lesson.skills,
					status: 'open',
					after: null,
					stage: null,
					editable: true
				}
			: null
	);

	const TASK_TOOLS: { id: CanvasTool; label: string; key: string; svg: string }[] = [
		{ id: 'select', label: t('Выбор и перемещение'), key: 'V', svg: 'M5 3l14 8-6 2-3 6z' },
		{ id: 'hand', label: t('Рука — двигать холст'), key: 'H', svg: 'M18 11V6a2 2 0 0 0-4 0v5M14 10V4a2 2 0 0 0-4 0v6M10 10.5V6a2 2 0 0 0-4 0v8M18 8a2 2 0 1 1 4 0v6a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.9-5.9-2.4L3.4 16a2 2 0 0 1 3-2.6L8 15' },
		{ id: 'path', label: t('Линия от руки'), key: 'B', svg: 'M3 17c3-6 6-9 9-5s6 1 9-5' },
		{ id: 'line', label: t('Прямая (Shift — под 45°)'), key: 'L', svg: 'M4 20 20 4' },
		{ id: 'arrow', label: t('Стрелка'), key: 'A', svg: 'M4 20 20 4M10 4h10v10' },
		{ id: 'ellipse', label: t('Овал'), key: 'O', svg: 'M12 6c5 0 9 2.7 9 6s-4 6-9 6-9-2.7-9-6 4-6 9-6z' },
		{ id: 'rect', label: t('Прямоугольник'), key: 'R', svg: 'M4 6h16v12H4z' },
		{ id: 'region', label: t('Область проверки'), key: 'M', svg: 'M3 8V4h4M17 4h4v4M21 16v4h-4M7 20H3v-4M8 8h8v8H8z' },
		{ id: 'eraser', label: t('Удалить линию'), key: 'E', svg: 'M7 20 3 16l10-10 6 6-8 8zM21 20H8' }
	];
	const GUIDE_TOOLS: { id: CanvasTool; label: string; key: string; svg: string }[] = [
		{ id: 'select', label: t('Выбор'), key: 'V', svg: 'M5 3l14 8-6 2-3 6z' },
		{ id: 'hand', label: t('Рука — двигать холст'), key: 'H', svg: 'M18 11V6a2 2 0 0 0-4 0v5M14 10V4a2 2 0 0 0-4 0v6M10 10.5V6a2 2 0 0 0-4 0v8M18 8a2 2 0 1 1 4 0v6a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.9-5.9-2.4L3.4 16a2 2 0 0 1 3-2.6L8 15' },
		{ id: 'brush', label: t('Кисть показа'), key: 'B', svg: 'M14.5 3.5 20.5 9.5l-7.5 7.5-6-6zM7 11l-1.5 1.5c-1.5 1.5-1 3-2 4.5-.5.8-1 1.5-1 2.5 1 0 1.7-.5 2.5-1 1.5-1 3-.5 4.5-2L11 15' }
	];
	const tools = $derived(mode === 'guide' ? GUIDE_TOOLS : TASK_TOOLS);
	const activeTool = $derived(mode === 'guide' ? guideTool : tool);
	const hint = $derived(
		mode === 'guide'
			? guideTool === 'brush'
				? t('Кисть показа: рисуйте — штрих станет клипом на дорожке «Штрихи». Пробел — проиграть.')
				: guideTool === 'hand'
					? t('Рука: тяните холст. Ctrl+колесо — масштаб.')
					: t('Выберите клип на дорожке или возьмите кисть показа (B).')
			: (
					{
						select: t('Выбор: щёлкните линию или область и тяните; у выбранной линии тяните ручки. Delete — удалить, Ctrl+Z — отменить.'),
						hand: t('Рука: тяните холст. Ctrl+колесо — масштаб.'),
						path: t('Линия от руки: ведите мышью или пером. Alt — пунктир.'),
						line: t('Прямая: тяните. Shift — под 45°, Alt — пунктир.'),
						arrow: t('Стрелка: тяните от начала к острию.'),
						ellipse: t('Овал: растяните рамку.'),
						rect: t('Прямоугольник: растяните рамку.'),
						region: t('Область проверки: растяните рамку — проверка смотрит только внутрь неё.'),
						eraser: t('Щёлкните линию, чтобы удалить её.'),
						brush: ''
					} as Record<CanvasTool, string>
				)[tool]
	);
</script>

<svelte:window {onkeydown} />

<div class="studio sumi-scope">
	<header class="top">
		<a class="back" href="/studio" aria-label={t('В мастерскую')}><Icon name="back" size={18} /></a>
		<div class="title">
			<h1>{lesson?.title || t('Урок')}</h1>
			<p>{t('Мастерская')}{draft ? ` · ${draft.category.title}` : ''}</p>
		</div>
		<span class="grow"></span>
		<div class="modes" role="tablist" aria-label={t('Режим студии')}>
			<button role="tab" aria-selected={mode === 'lesson'} onclick={() => (mode = 'lesson')}>{t('Урок')}</button>
			<button role="tab" aria-selected={mode === 'guide'} onclick={() => (mode = 'guide')}>{t('Гайд')}</button>
			<button role="tab" aria-selected={mode === 'task'} onclick={() => (mode = 'task')}>{t('Задание')}</button>
		</div>
		<span class="grow"></span>
		<span class="saved" aria-live="polite">{saving ? t('Сохраняем…') : dirty ? t('Есть несохранённые изменения') : savedAt ? t('Сохранено · {time}', { time: savedAt }) : ''}</span>
		{#if mode === 'guide'}
			<button class="btn-o" onclick={playInEditor} disabled={!script.steps.length}>{t('Проиграть в PhotoCraft')}</button>
		{:else if mode === 'task'}
			<button class="btn-o" onclick={showHintInEditor} disabled={!step?.hint}>{t('Показать в PhotoCraft')}</button>
		{/if}
		<button class="btn-q" onclick={preview} disabled={!lesson?.task.steps.length}>{t('Глазами ученика')}</button>
		<button class="btn-p" onclick={save} disabled={saving || !lesson}>{t('Сохранить')}</button>
	</header>

	{#if error}<p class="error" role="alert"><Icon name="alert" size={16} /> {error}</p>{/if}

	{#if lesson}
		<div class="body" class:lessonmode={mode === 'lesson'}>
			<!-- left -->
			<aside class="left">
				{#if mode === 'lesson'}
					<p class="sec">{t('Разделы урока')}</p>
					{#each [['about', t('Описание и карточка'), t('как урок выглядит в каталоге')], ['theory', t('Теория'), count(theory.trim().split(/\s+/).filter(Boolean).length, ['слово', 'слова', 'слов'])], ['quiz', t('Вопросы после теории'), count(lesson.quiz?.length ?? 0, ['вопрос', 'вопроса', 'вопросов'])]] as [key, name, meta] (key)}
						<button class="item" class:on={section === key} onclick={() => (section = key as LessonSection)}>
							<span class="item-name">{name}</span><span class="item-meta">{meta}</span>
						</button>
					{/each}
				{:else if mode === 'guide'}
					<p class="sec">{t('С чего начинается')}</p>
					<div class="item static"><span class="item-name">{lesson.task.open ? t('Картинка задания') : t('Чистый холст')}</span><span class="item-meta">{lesson.task.open ?? `${size.w} × ${size.h}`}</span></div>
					<button class="item" onclick={() => attach('video')}>
						<span class="item-name">{t('Видео урока')}</span><span class="item-meta">{lesson.guide.video ? `${lesson.guide.video} · ${t('щёлкните, чтобы заменить')}` : t('необязательно · mp4 или webm')}</span>
					</button>
					{#if lesson.guide.video}<button class="link" onclick={() => lesson && delete lesson.guide.video}>{t('Убрать видео')}</button>{/if}
					<p class="sec">{t('Сцены гайда')}</p>
					{#each script.steps as s, i (i)}
						{#if s.caption.trim()}
							<button class="item scene" class:on={guideStep === i} onclick={() => selectGuideStep(i)}>
								<span class="num">{String(i + 1).padStart(2, '0')}</span><span class="item-name">{s.caption}</span>
							</button>
						{/if}
					{/each}
					{#if !script.steps.some((s) => s.caption.trim())}
						<p class="muted-note">{t('Сцены появятся, когда у шагов гайда будут подписи: выберите клип и впишите подпись справа.')}</p>
					{/if}
				{:else}
					<p class="sec">{t('Холст задания')}</p>
					<button class="item" onclick={() => attach('open')}>
						<span class="item-name">{lesson.task.open ? t('Картинка задания') : t('Чистый холст')}</span>
						<span class="item-meta">{lesson.task.open ? `${lesson.task.open} · ${size.w} × ${size.h}` : t('щёлкните, чтобы загрузить картинку')}</span>
					</button>
					{#if lesson.task.canvas}
						<div class="canvas-size">
							<label>{t('Ширина')} <input type="number" min="16" max="8192" bind:value={lesson.task.canvas.width} onchange={measure} /></label>
							<label>{t('Высота')} <input type="number" min="16" max="8192" bind:value={lesson.task.canvas.height} onchange={measure} /></label>
							<label>{t('Фон')} <input type="color" bind:value={lesson.task.canvas.background} /></label>
						</div>
					{:else}
						<button class="link" onclick={useCanvas}>{t('Начать с чистого холста')}</button>
					{/if}
					<button class="item" onclick={() => attach('reference')}>
						<span class="item-name">{t('Эталон')}</span><span class="item-meta">{lesson.task.reference ?? t('нужен для живой проверки')}</span>
					</button>
					<p class="sec row">{t('Шаги · {n}', { n: lesson.task.steps.length })} <button class="mini" onclick={addStep} aria-label={t('Добавить шаг')}><Icon name="plus" size={14} /></button></p>
					{#each lesson.task.steps as s, i (s.id + i)}
						{@const bad = proofs.some((p) => p.step === s.id && (p.verdict === 'passesUntouched' || p.verdict === 'failsReference'))}
						<button class="step" class:on={stepIndex === i} onclick={() => ((stepIndex = i), (selectedShape = -1), (selectedRegion = null))}>
							<span class="seal" class:filled={stepIndex === i}>{i + 1}</span>
							<span class="grow">
								<span class="item-name">{s.text || t('Новый шаг')}</span>
								<span class="item-meta">{count(s.hint?.shapes?.length ?? 0, ['линия', 'линии', 'линий'])}{s.hint?.target ? ` · ${t('куда нажать')}` : ''} · {count(s.checks.length, ['проверка', 'проверки', 'проверок'])}</span>
							</span>
							{#if bad}<span class="warn-dot" aria-label={t('проверку стоит поправить')}></span>{/if}
						</button>
					{/each}
					<button class="add" onclick={addStep}><Icon name="plus" size={14} /> {t('Добавить шаг')}</button>
				{/if}
				<div class="ready">
					<p class="ready-title">{t('Готовность урока')}</p>
					{#each readiness as r, i (i)}
						{#if r.go}
							<button class="ready-line bad link-line" onclick={r.go}><Icon name="alert" size={14} /> {r.text}</button>
						{:else}
							<p class="ready-line" class:bad={r.ok === false}><Icon name={r.ok ? 'check' : 'alert'} size={14} /> {r.text}</p>
						{/if}
					{/each}
				</div>
			</aside>

			<!-- center -->
			<section class="center">
				{#if live}
					<div class="livebar" class:rec={live === 'record'}>
						{#if live === 'record'}<span class="dot"></span><b class="mono">{t('{n} с', { n: tick ? (tick.elapsedMs / 1000).toFixed(0) : 0 })}</b>{/if}
						<span class="grow">{liveNote}{#if live === 'record' && tick} · {t('штрихов: {s}, действий: {a}', { s: tick.strokes, a: tick.actions })}{/if}</span>
						{#if live === 'record'}
							<label class="opt"><input type="checkbox" bind:checked={keepPauses} /> {t('Паузы как есть')}</label>
							<label class="opt"><input type="checkbox" bind:checked={keepUndo} /> {t('Писать отмены')}</label>
							<button class="btn-w" onclick={toggleRecording}>{t('Остановить и добавить в гайд')}</button>
						{:else}
							<button class="btn-w" onclick={closeLive}>{t('Вернуться к холсту')}</button>
						{/if}
					</div>
					<div class="slot" use:editorSlot aria-label={t('Редактор PhotoCraft')}></div>
				{:else if mode === 'lesson'}
					<div class="texts" class:pad={section !== 'theory'}>
						<LessonTexts bind:lesson bind:theory {section} lessonIds={draft?.lessonIds ?? []} />
					</div>
				{:else}
					<div class="stage">
						<div class="toolbar-top">
							{#if mode === 'task'}
								<button class="vis" aria-pressed={showHints} onclick={() => (showHints = !showHints)}><span class="d shu"></span> {t('Подсказки шага')}</button>
								<button class="vis" aria-pressed={showOthers} onclick={() => (showOthers = !showOthers)}><span class="d"></span> {t('Другие шаги')}</button>
								<button class="vis" aria-pressed={showRegions} onclick={() => (showRegions = !showRegions)}><span class="d ai"></span> {t('Области проверок')}</button>
							{/if}
							{#if referenceImage}
								<label class="vis ref">{t('Эталон')} <input type="range" min="0" max="100" step="5" value={refOpacity * 100} oninput={(e) => (refOpacity = Number(e.currentTarget.value) / 100)} /> <b>{Math.round(refOpacity * 100)}%</b></label>
							{/if}
						</div>
						<div class="tools" role="toolbar" aria-label={t('Инструменты автора')}>
							{#each tools as tl (tl.id)}
								<button class="tool" class:on={activeTool === tl.id} onclick={() => (mode === 'guide' ? (guideTool = tl.id) : (tool = tl.id))} title="{tl.label} · {tl.key}" aria-label={tl.label}>
									<svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true"><path d={tl.svg} /></svg>
								</button>
							{/each}
						</div>
						<div class="paper-wrap" class:panning={activeTool === 'hand'} bind:this={wrap} bind:clientWidth={wrapW} bind:clientHeight={wrapH} onwheel={wheel} onpointerdown={panDown} onpointermove={panMove} onpointerup={() => (pan = null)} role="presentation">
							<div class="paper-inner">
							<StudioCanvas
								{scale}
								image={taskImage}
								reference={referenceImage}
								referenceOpacity={refOpacity}
								width={size.w}
								height={size.h}
								background={lesson.task.canvas?.background ?? '#FFFFFF'}
								tool={activeTool}
								shapes={mode === 'task' && showHints ? (step?.hint?.shapes ?? []) : []}
								dimShapes={mode === 'task' && showOthers ? lesson.task.steps.flatMap((s, i) => (i === stepIndex ? [] : (s.hint?.shapes ?? []))) : []}
								selected={selectedShape}
								regions={mode === 'task' && showRegions ? regionMarks : []}
								{selectedRegion}
								{demo}
								caption={mode === 'guide' ? captionAt(script.steps, playhead) : ''}
								onshapes={setShapes}
								onselect={(i) => (selectedShape = i)}
								onregion={setRegion}
								onregionselect={(k) => ((selectedRegion = k), k && (tab = 'checks'))}
								onregiondraw={drawRegion}
								onstroke={addStroke}
							/>
							</div>
						</div>
						<div class="zoombar" role="toolbar" aria-label={t('Масштаб и история')}>
							<button onclick={undo} disabled={!canUndo} title={t('Отменить · Ctrl+Z')} aria-label={t('Отменить')}><svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><path d="M9 14 4 9l5-5M4 9h10.5a5.5 5.5 0 0 1 0 11H11" /></svg></button>
							<button onclick={redo} disabled={!canRedo} title={t('Повторить · Ctrl+Shift+Z')} aria-label={t('Повторить')}><svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><path d="m15 14 5-5-5-5M20 9H9.5a5.5 5.5 0 0 0 0 11H13" /></svg></button>
							<span class="sep"></span>
							<button onclick={() => zoomBy(-1)} aria-label={t('Уменьшить')} title={t('Уменьшить · Ctrl+−')}><svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><path d="M5 12h14" /></svg></button>
							<button class="pct mono" onclick={() => (zoom = 1)} title={t('Вписать · Ctrl+0')}>{Math.round(zoom * 100)}%</button>
							<button onclick={() => zoomBy(1)} aria-label={t('Увеличить')} title={t('Увеличить · Ctrl++')}><svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg></button>
						</div>
						<p class="hintbar">{hint}</p>
						{#if note}<p class="note" role="status">{note} <button onclick={() => (note = '')} aria-label={t('Скрыть')}><Icon name="close" size={12} /></button></p>{/if}
					</div>
				{/if}
				{#if mode === 'guide' || live === 'record'}
					<GuideTimeline
						steps={script.steps}
						selected={guideStep}
						{playhead}
						{playing}
						recording={live === 'record'}
						onselect={selectGuideStep}
						onseek={(ms) => ((playhead = ms), (guideStep = frameAt(script.steps, ms).step))}
						onsteps={(s) => (script.steps = s)}
						onplay={play}
						onrecord={toggleRecording}
						onaddcaption={addCaption}
						onremove={removeGuideStep}
						onsplit={splitAtPlayhead}
						taskLabels={Object.fromEntries(lesson.task.steps.map((s, i) => [s.id, `${i + 1} · ${s.text || s.id}`]))}
					/>
				{:else if mode === 'task'}
					<button class="collapsed" onclick={() => (mode = 'guide')}>
						<Icon name="video" size={16} /> {t('Гайд · {steps} · {sec} с', { steps: count(script.steps.length, ['шаг', 'шага', 'шагов']), sec: Math.round(total / 100) / 10 })}
						{#if step && taskStepStart(script.steps, step.id) !== null}
							<span class="muted-inline">· {t('шаг {n} задания показан в гайде с {sec} с', { n: stepIndex + 1, sec: (taskStepStart(script.steps, step.id)! / 1000).toFixed(1) })}</span>
						{/if}
						<span class="grow"></span> {t('Открыть дорожки')}
					</button>
				{/if}
			</section>

			<!-- right -->
			{#if mode !== 'lesson' || section === 'about'}
				<aside class="right">
					{#if mode === 'task'}
						{#if step}
							<TaskPanel
								bind:lesson
								step={stepIndex}
								bind:tab
								{selectedShape}
								{selectedRegion}
								width={size.w}
								height={size.h}
								{proofs}
								letters={lettersOf}
								onselectshape={(i) => (selectedShape = i)}
								onfocusregion={(k) => (selectedRegion = k)}
								onmap={() => (mapFor = 'hint')}
								onhintpreview={showHintInEditor}
								onremovestep={removeTaskStep}
								onmovestep={moveTaskStep}
								{taskImage}
								{referenceImage}
								marks={regionMarks}
								background={lesson.task.canvas?.background ?? '#FFFFFF'}
							/>
						{:else}
							<p class="empty">{t('В задании пока нет шагов.')} <button class="link" onclick={addStep}>{t('Добавить первый шаг')}</button></p>
						{/if}
					{:else if mode === 'guide'}
						<GuidePanel bind:script bind:brush selected={guideStep} {lesson} width={size.w} height={size.h} onmap={() => (mapFor = 'guide')} onghost={ghostFromGuide} onmove={moveGuideStep} />
					{:else if card}
						<p class="sec">{t('Карточка в каталоге')}</p>
						<div class="card-preview" inert><LessonTile {card} /></div>
						{#if issues.length}
							<p class="sec">{t('Замечания')}</p>
							{#each issues as i (i.field + i.message)}<p class="issue" class:err={i.severity === 'error'}><span class="mono">{i.field}</span> {i.message}</p>{/each}
						{/if}
					{/if}
				</aside>
			{/if}
		</div>
	{/if}
</div>

{#if mapFor && lesson}
	<InterfaceMap
		value={mapFor === 'hint' ? (step?.hint?.target ?? null) : (script.steps[guideStep]?.highlight ?? null)}
		step={mapFor === 'hint' ? t('шаг {n}', { n: stepIndex + 1 }) : t('шаг гайда {n}', { n: guideStep + 1 })}
		onclose={() => (mapFor = null)}
		onpick={(target) => {
			if (mapFor === 'hint' && lesson) {
				const s = lesson.task.steps[stepIndex];
				s.hint = { ...(s.hint ?? { text: '' }), target: target ?? undefined };
				if (!target) delete s.hint.target;
			} else if (guideStep >= 0) {
				if (target) script.steps[guideStep].highlight = target;
				else delete script.steps[guideStep].highlight;
			}
			mapFor = null;
		}}
	/>
{/if}

<style>
	.studio {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--sumi-bg);
		color: var(--sumi-text);
		font-family: var(--font-ui);
	}

	.top {
		display: flex;
		align-items: center;
		gap: 12px;
		height: 60px;
		padding: 0 16px;
		background: var(--sumi-panel);
		border-bottom: 1px solid var(--sumi-line);
	}

	.back {
		display: grid;
		place-items: center;
		width: 36px;
		height: 36px;
		border-radius: 8px;
		background: var(--sumi-raised);
		color: var(--sumi-text);
	}

	.title h1 {
		margin: 0;
		font: 600 16px/1.3 var(--font-ui);
	}

	.title p {
		margin: 0;
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
	}

	.grow {
		flex: 1;
		min-width: 0;
	}

	.modes {
		display: flex;
		gap: 2px;
		padding: 3px;
		border-radius: 8px;
		background: var(--sumi-bg);
	}

	.modes button {
		padding: 6px 14px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--sumi-muted);
		font: 600 13px var(--font-ui);
		cursor: pointer;
	}

	.modes button[aria-selected='true'] {
		background: var(--sumi-line);
		color: var(--sumi-text);
	}

	.saved {
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
	}

	.btn-o,
	.btn-q,
	.btn-p,
	.btn-w {
		padding: 8px 14px;
		border-radius: 8px;
		border: 1px solid var(--sumi-line);
		background: transparent;
		color: var(--sumi-text);
		font: 600 13px var(--font-ui);
		cursor: pointer;
		white-space: nowrap;
	}

	.btn-q {
		border-color: transparent;
		background: var(--sumi-raised);
	}

	.btn-p {
		border-color: transparent;
		background: var(--shu-500);
		color: #fff;
	}

	.btn-w {
		border-color: transparent;
		background: #fff;
		color: var(--shu-700);
	}

	button:disabled {
		opacity: 0.45;
		cursor: default;
	}

	.error {
		display: flex;
		gap: 8px;
		align-items: center;
		margin: 0;
		padding: 8px 16px;
		background: var(--shu-700);
		color: #fff;
		font-size: 13px;
	}

	.body {
		flex: 1;
		min-height: 0;
		display: grid;
		grid-template-columns: 288px 1fr 336px;
	}

	.body.lessonmode:not(:has(.right)) {
		grid-template-columns: 288px 1fr;
	}

	.left,
	.right {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-height: 0;
		overflow-y: auto;
		padding: 16px 12px;
		background: var(--sumi-panel);
		scrollbar-color: var(--sumi-line) transparent;
	}

	.left {
		border-right: 1px solid var(--sumi-line);
	}

	.right {
		padding: 16px;
		border-left: 1px solid var(--sumi-line);
	}

	.sec {
		margin: 10px 0 4px;
		color: var(--sumi-muted);
		font: 700 12px var(--font-ui);
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	.sec:first-child {
		margin-top: 0;
	}

	.sec.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.item,
	.step {
		display: flex;
		flex-direction: column;
		gap: 1px;
		width: 100%;
		padding: 8px 10px;
		border: 1px solid transparent;
		border-radius: 8px;
		background: none;
		color: var(--sumi-text);
		text-align: left;
		cursor: pointer;
	}

	.item:hover,
	.step:hover {
		background: var(--sumi-raised);
	}

	.item.static {
		cursor: default;
	}

	.item.on,
	.step.on {
		border-color: var(--shu-500);
		background: var(--sumi-raised);
	}

	.item.scene {
		flex-direction: row;
		align-items: center;
		gap: 10px;
	}

	.num {
		color: var(--sumi-muted);
		font: 500 12px var(--font-mono);
	}

	.item-name {
		display: block;
		font: 600 13px var(--font-ui);
	}

	.item-meta {
		display: block;
		color: var(--sumi-muted);
		font: 500 12px var(--font-ui);
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.step {
		flex-direction: row;
		align-items: center;
		gap: 12px;
	}

	.seal {
		flex: none;
		display: grid;
		place-items: center;
		width: 26px;
		height: 26px;
		border: 1.5px solid var(--shu-500);
		border-radius: 50%;
		color: var(--shu-500);
		font: 700 13px var(--font-ui);
	}

	.seal.filled {
		background: var(--shu-500);
		color: #fff;
	}

	.warn-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--kaki-500);
	}

	.mini {
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--sumi-muted);
		cursor: pointer;
	}

	.add {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		border: 1px dashed var(--sumi-line);
		border-radius: 8px;
		background: none;
		color: var(--sumi-muted);
		font: 600 13px var(--font-ui);
		cursor: pointer;
	}

	.link {
		align-self: flex-start;
		padding: 2px 10px;
		border: 0;
		background: none;
		color: var(--ai-300);
		font: 600 12px var(--font-ui);
		cursor: pointer;
	}

	.canvas-size {
		display: grid;
		grid-template-columns: 1fr 1fr 48px;
		gap: 6px;
		padding: 0 10px;
	}

	.canvas-size label {
		display: flex;
		flex-direction: column;
		gap: 4px;
		color: var(--sumi-muted);
		font: 600 11px var(--font-ui);
	}

	.canvas-size input {
		min-width: 0;
		padding: 6px;
		border: 1px solid var(--sumi-line);
		border-radius: 6px;
		background: var(--sumi-bg);
		color: var(--sumi-text);
	}

	.muted-note,
	.empty {
		color: var(--sumi-muted);
		font: 13px/1.5 var(--font-ui);
	}

	.ready {
		margin-top: auto;
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 12px;
		border-radius: 8px;
		background: var(--sumi-raised);
	}

	.ready-title {
		margin: 0 0 2px;
		font: 600 13px var(--font-ui);
	}

	.ready-line {
		display: flex;
		gap: 8px;
		align-items: flex-start;
		margin: 0;
		color: var(--sumi-muted);
		font: 500 12px/1.4 var(--font-ui);
	}

	.ready-line :global(svg) {
		flex: none;
		color: var(--ai-300);
	}

	.ready-line.bad {
		color: var(--sumi-text);
	}

	.ready-line.bad :global(svg) {
		color: var(--kaki-500);
	}

	.center {
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 0;
	}

	.stage {
		position: relative;
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 16px 16px 12px 72px;
	}

	.toolbar-top {
		display: flex;
		gap: 6px;
		padding: 6px;
		border: 1px solid var(--sumi-line);
		border-radius: 10px;
		background: var(--sumi-panel);
	}

	.vis {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 10px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
		cursor: pointer;
	}

	.vis[aria-pressed='true'] {
		background: var(--sumi-raised);
		color: var(--sumi-text);
	}

	.vis .d {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--sumi-muted);
	}

	.d.shu {
		background: var(--shu-500);
	}

	.d.ai {
		background: var(--ai-500);
	}

	.vis.ref input {
		width: 90px;
		accent-color: var(--kin-500);
	}

	.vis.ref b {
		color: var(--sumi-text);
		font: 500 12px var(--font-mono);
	}

	.tools {
		position: absolute;
		left: 14px;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 6px;
		border: 1px solid var(--sumi-line);
		border-radius: 14px;
		background: var(--sumi-panel);
		box-shadow: 0 12px 32px rgb(0 0 0 / 35%);
	}

	.tool {
		display: grid;
		place-items: center;
		width: 36px;
		height: 36px;
		border: 0;
		border-radius: 8px;
		background: none;
		color: var(--sumi-text);
		cursor: pointer;
	}

	.tool svg {
		fill: none;
		stroke: currentColor;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.tool:hover {
		background: var(--sumi-raised);
	}

	.tool.on {
		background: var(--shu-500);
		color: #fff;
	}

	.paper-wrap {
		flex: 1;
		min-height: 0;
		width: 100%;
		display: flex;
		overflow: auto;
		margin: 12px 0 8px;
		scrollbar-color: var(--sumi-line) transparent;
	}

	.paper-wrap.panning {
		cursor: grab;
	}

	.paper-inner {
		margin: auto;
		padding: 20px;
	}

	.zoombar {
		position: absolute;
		left: 14px;
		bottom: 14px;
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 4px;
		border: 1px solid var(--sumi-line);
		border-radius: 10px;
		background: var(--sumi-panel);
	}

	.zoombar button {
		display: grid;
		place-items: center;
		min-width: 30px;
		height: 30px;
		padding: 0 6px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--sumi-text);
		font: 600 15px var(--font-ui);
		cursor: pointer;
	}

	.zoombar button:hover:not(:disabled) {
		background: var(--sumi-raised);
	}

	.zoombar .pct {
		font: 500 12px var(--font-mono);
		min-width: 52px;
	}

	.zoombar svg {
		fill: none;
		stroke: currentColor;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.zoombar .sep {
		width: 1px;
		height: 18px;
		margin: 0 4px;
		background: var(--sumi-line);
	}

	.muted-inline {
		color: var(--sumi-muted);
		font-weight: 500;
	}

	.opt {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font: 600 12px var(--font-ui);
		white-space: nowrap;
	}

	.link-line {
		border: 0;
		background: none;
		padding: 0;
		text-align: left;
		cursor: pointer;
		text-decoration: underline;
		text-decoration-color: var(--sumi-line);
		text-underline-offset: 3px;
	}

	.hintbar {
		margin: 0;
		padding: 7px 14px;
		border: 1px solid var(--sumi-line);
		border-radius: 999px;
		background: var(--sumi-panel);
		color: var(--sumi-muted);
		font: 500 12px var(--font-ui);
	}

	.note {
		position: absolute;
		top: 64px;
		display: flex;
		align-items: center;
		gap: 10px;
		margin: 0;
		padding: 8px 8px 8px 14px;
		border-radius: 8px;
		background: var(--ai-700);
		color: #fff;
		font: 600 13px var(--font-ui);
		box-shadow: 0 12px 32px rgb(0 0 0 / 35%);
	}

	.note button {
		display: grid;
		place-items: center;
		width: 22px;
		height: 22px;
		border: 0;
		border-radius: 6px;
		background: rgb(255 255 255 / 15%);
		color: #fff;
		cursor: pointer;
	}

	.collapsed {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 44px;
		padding: 0 16px;
		border: 0;
		border-top: 1px solid var(--sumi-line);
		background: var(--sumi-panel);
		color: var(--sumi-text);
		font: 600 13px var(--font-ui);
		cursor: pointer;
	}

	.livebar {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 12px;
		background: var(--ai-700);
		color: #fff;
		font: 600 13px var(--font-ui);
	}

	.livebar.rec {
		background: var(--shu-700);
	}

	.livebar .dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: #fff;
		animation: blink 1s steps(2) infinite;
	}

	@keyframes blink {
		50% {
			opacity: 0.3;
		}
	}

	.slot {
		flex: 1;
		min-height: 0;
		background: var(--sumi-bg);
	}

	.texts {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
	}

	.texts.pad {
		padding: 24px;
	}

	.card-preview {
		pointer-events: none;
	}

	.issue {
		margin: 0;
		color: var(--sumi-muted);
		font: 12px/1.5 var(--font-ui);
	}

	.issue.err {
		color: var(--kaki-500);
	}
</style>
