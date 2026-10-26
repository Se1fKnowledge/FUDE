// Typed calls to the Rust side (app/src-tauri/src). Lesson models mirror the TOML format
// (snake_case, docs/lesson-format.md); the app's own views are camelCase.
import { invoke } from '@tauri-apps/api/core';

export type Accent = 'ai' | 'shu' | 'kaki' | 'kin';
export type Skill = 'tools' | 'line' | 'form' | 'light' | 'color' | 'composition';
export type Status = 'passed' | 'progress' | 'open' | 'locked';

export interface Profile {
	name: string;
	dailyGoal: number;
	created: string;
}

export interface Rank {
	index: number;
	name: string;
	title: string;
	xp: number;
	floor: number;
	next: number | null;
}

export interface Streak {
	current: number;
	best: number;
	bestEnd: string | null;
}

export interface Boot {
	profile: Profile;
	firstRun: boolean;
	editor: string | null;
	version: string;
	problems: number;
	rank: Rank;
	streak: Streak;
	doneToday: number;
}

export interface LessonCard {
	id: string;
	title: string;
	summary: string;
	category: string;
	categoryTitle: string;
	kanji: string | null;
	accent: Accent;
	difficulty: number;
	difficultyLabel: string;
	minutes: number;
	number: number;
	of: number;
	skills: Skill[];
	status: Status;
	after: string | null;
	stage: string | null;
	editable: boolean;
}

export interface CategoryView {
	id: string;
	title: string;
	kanji: string | null;
	description: string;
	accent: Accent;
	user: boolean;
	passed: number;
	minutes: number;
	difficulty: [number, number];
	lessons: LessonCard[];
}

export interface Catalog {
	categories: CategoryView[];
	total: number;
	passed: number;
}

// ---------------------------------------------------------------- lesson model

export type Region = [number, number, number, number];
export interface Range {
	min?: number;
	max?: number;
}

export type Shape =
	| { kind: 'ellipse'; rect: Region }
	| { kind: 'rect'; rect: Region }
	| { kind: 'line'; from: [number, number]; to: [number, number] }
	| { kind: 'path'; points: [number, number][] };

export type CheckKind =
	| { kind: 'tool'; tool: string }
	| { kind: 'brush'; size?: Range; hardness?: Range; opacity?: Range }
	| { kind: 'zoom'; zoom: Range }
	| { kind: 'layers'; count: Range }
	| { kind: 'layer'; name?: string; visible?: boolean; opacity?: Range; type?: string }
	| { kind: 'selection'; present?: boolean; width?: Range; height?: Range }
	| { kind: 'history'; entries?: Range; can_redo?: boolean }
	| { kind: 'foreground'; color: string; tolerance?: number }
	| { kind: 'coverage'; region: Region; changed: Range }
	| { kind: 'luminance'; region: Region; value: Range }
	| { kind: 'darker'; region: Region; than: Region; by?: number }
	| { kind: 'color'; region: Region; color: string; tolerance?: number }
	| { kind: 'similar'; reference?: string; score: Range };

export type Check = CheckKind & { message?: string };

export interface Hint {
	text: string;
	target?: string;
	shapes?: Shape[];
}

export interface Step {
	id: string;
	text: string;
	hint?: Hint;
	checks: Check[];
}

export interface Question {
	question: string;
	options: string[];
	answer: number;
	explain?: string;
}

export interface Lesson {
	format: number;
	id: string;
	title: string;
	summary: string;
	difficulty: number;
	minutes: number;
	order: number;
	skills: Skill[];
	requires?: string[];
	author?: string;
	theory: { file: string };
	guide: { script?: string; video?: string; poster?: string };
	quiz?: Question[];
	task: {
		brief: string;
		open?: string;
		reference?: string;
		canvas?: { width: number; height: number; background: string };
		steps: Step[];
	};
}

export interface Category {
	id: string;
	title: string;
	kanji?: string | null;
	description: string;
	order: number;
	accent: Accent;
	author?: string | null;
}

export interface Issue {
	severity: 'error' | 'warning';
	field: string;
	message: string;
}

// ---------------------------------------------------------------- lesson page

export interface GuideStepView {
	caption: string;
	highlight: string | null;
	pause: number;
}

export interface LessonView {
	card: LessonCard;
	lesson: Lesson;
	theory: string;
	dir: string;
	guide: GuideStepView[];
	guideMs: number;
	video: string | null;
	poster: string | null;
	taskImage: string | null;
	referenceImage: string | null;
	next: LessonCard | null;
}

export interface JournalEntry {
	at: string;
	kind: string;
	title: string;
	detail: string;
}

export interface Home {
	greeting: string;
	name: string;
	date: string;
	goal: number;
	doneToday: number;
	today: JournalEntry[];
	current: LessonCard | null;
	recommended: LessonCard[];
	streak: Streak;
	rank: Rank;
	firstDay: boolean;
}

export interface Achievement {
	id: string;
	kanji: string;
	title: string;
	description: string;
	group: string;
	current: number;
	target: number;
	unlocked: string | null;
}

export interface Achievements {
	rank: Rank;
	list: Achievement[];
	unlocked: number;
}

export interface Day {
	date: string;
	tasks: number;
	minutes: number;
	noHints: boolean;
}

export interface Activity {
	days: Day[];
	today: string;
	streak: Streak;
	activeDays: number;
	tasks: number;
	minutes: number;
	week: Day[];
	goalMinutes: number;
	journal: JournalEntry[];
}

export interface SkillScore {
	skill: Skill;
	label: string;
	score: number;
	before: number;
	points: number;
	level: number;
}

export interface Stats {
	skills: SkillScore[];
	weekly: [string, number][];
	totalXp: number;
	periodXp: number;
	independence: number | null;
	hints: number;
	passed: number;
	advice: { text: string; detail: string; lesson: string | null } | null;
}

export interface ResultView {
	attempt: number;
	card: LessonCard | null;
	passed: boolean;
	seconds: number;
	normMinutes: number;
	hints: number;
	steps: number;
	xp: number;
	similarity: number | null;
	canvas: string | null;
	reference: string | null;
	skills: [string, number][];
	rank: Rank;
	streak: Streak;
	next: LessonCard | null;
	achievements: Achievement[];
}

// ---------------------------------------------------------------- editor

export interface EditorStatus {
	found: boolean;
	path: string | null;
	running: boolean;
}

export interface GuideEvent {
	lesson: string;
	state: 'starting' | 'step' | 'done' | 'stopped' | 'error';
	index: number;
	total: number;
	caption: string;
	highlight: string | null;
	message: string | null;
}

export interface StepView {
	id: string;
	text: string;
	hasHint: boolean;
	manual: boolean;
}

export interface CheckOutcome {
	kind: string;
	passed: boolean;
	message: string;
	value: number | null;
}

export interface StepOutcome {
	id: string;
	passed: boolean;
	manual: boolean;
	checks: CheckOutcome[];
}

export interface CheckResult {
	attempt: number;
	outcome: { passed: boolean; current: number | null; steps: StepOutcome[] };
	done: number;
	total: number;
	finished: boolean;
	advanced: number;
	xp: number | null;
	hints: number;
}

export interface HintReport {
	step: string;
	text: string;
	target: string | null;
	shapes: number;
	hints: number;
}

// ---------------------------------------------------------------- studio

export interface DraftView {
	dir: string;
	id: string;
	title: string;
	difficulty: number;
	errors: number;
	warnings: number;
	broken: string | null;
}

export interface Studio {
	root: string;
	categories: { dir: string; category: Category; lessons: DraftView[] }[];
	builtin: [string, string][];
}

export interface DraftDetail {
	dir: string;
	category: Category;
	lesson: Lesson;
	theory: string;
	guide: string | null;
	issues: Issue[];
	taskImage: string | null;
	referenceImage: string | null;
	video: string | null;
	lessonIds: [string, string][];
}

export interface PackageInfo {
	manifest: { format: number; title: string; created_by?: string; categories: string[] };
	categories: Category[];
	lessons: Lesson[];
	issues: Issue[];
}

export interface ImportReport {
	title: string;
	categories_added: string[];
	lessons_added: string[];
	lessons_replaced: string[];
	warnings: string[];
}

export interface Problem {
	path: string;
	message: string;
	issues: Issue[];
}

// ---------------------------------------------------------------- calls

export const api = {
	boot: () => invoke<Boot>('boot'),
	profileSave: (name: string, dailyGoal: number) => invoke<Profile>('profile_save', { name, dailyGoal }),
	catalog: () => invoke<Catalog>('catalog'),
	lesson: (id: string) => invoke<LessonView>('lesson', { id }),
	lessonStage: (id: string, stage: string) => invoke<void>('lesson_stage', { id, stage }),
	home: () => invoke<Home>('home'),
	achievements: () => invoke<Achievements>('achievements'),
	activity: () => invoke<Activity>('activity'),
	stats: (period: 'week' | 'quarter' | 'all') => invoke<Stats>('stats', { period }),
	result: (attempt: number) => invoke<ResultView>('result', { attempt }),

	editorStatus: () => invoke<EditorStatus>('editor_status'),
	editorClose: () => invoke<void>('editor_close'),
	editorLayout: (embedded: boolean) => invoke<{ supported: boolean; embedded: boolean }>('editor_layout', { embedded }),
	editorArea: (rect: { x: number; y: number; width: number; height: number } | null) => invoke<void>('editor_area', { rect }),
	guidePlay: (id: string) => invoke<void>('guide_play', { id }),
	guideStop: () => invoke<void>('guide_stop'),
	practiceStart: (id: string) => invoke<{ attempt: number; steps: StepView[] }>('practice_start', { id }),
	practiceCheck: () => invoke<CheckResult>('practice_check'),
	practiceHint: () => invoke<HintReport>('practice_hint'),
	practiceHideHint: () => invoke<void>('practice_hide_hint'),
	practiceConfirm: (step: string) => invoke<void>('practice_confirm', { step }),
	practiceStop: () => invoke<void>('practice_stop'),

	studio: () => invoke<Studio>('studio'),
	categoryCreate: (title: string, kanji: string | null, accentName: Accent, description: string) =>
		invoke<string>('studio_category_create', { title, kanji, accentName, description }),
	categorySave: (dir: string, category: Category) => invoke<Issue[]>('studio_category_save', { dir, category }),
	lessonCreate: (categoryDir: string, title: string, difficulty: number) =>
		invoke<string>('studio_lesson_create', { categoryDir, title, difficulty }),
	draft: (dir: string) => invoke<DraftDetail>('studio_lesson', { dir }),
	draftSave: (dir: string, lesson: Lesson, theory: string, guide: string | null) =>
		invoke<Issue[]>('studio_lesson_save', { dir, lesson, theory, guide }),
	attach: (dir: string, kind: 'video' | 'image', source: string) => invoke<string>('studio_attach', { dir, kind, source }),
	picture: (dir: string, path: string) => invoke<string | null>('studio_picture', { dir, path }),
	duplicate: (lesson: string, categoryDir: string) => invoke<string>('studio_duplicate', { lesson, categoryDir }),
	remove: (dir: string) => invoke<void>('studio_remove', { dir }),
	packageExport: (category: string, lessons: string[], out: string) => invoke<number>('package_export', { category, lessons, out }),
	packageInspect: (path: string) => invoke<PackageInfo>('package_inspect', { path }),
	packageImport: (path: string, replace: boolean) => invoke<ImportReport>('package_import', { path, replace }),
	problems: () => invoke<Problem[]>('library_problems')
};

/** The error text of a failed call. */
export function errorText(e: unknown): string {
	if (typeof e === 'string') return e;
	if (e instanceof Error) return e.message;
	return 'Что-то пошло не так';
}
