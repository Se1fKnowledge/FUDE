// The guide on tracks. A guide is a list of steps that run one after another: each step makes
// its calls, shows its caption and highlight, then waits `pause` milliseconds. The timeline
// draws each step's parts on their own track at the time the step runs.
import type { GuideCall, GuideScript, GuideStep } from '../api.ts';
import { TOOLS } from '../checks.ts';

export const DEFAULT_PAUSE = 600;
export const MAX_PAUSE = 15000;
export const MIN_PAUSE = 100;

export type Track = 'caption' | 'highlight' | 'action' | 'stroke';
export const TRACKS: { id: Track; label: string }[] = [
	{ id: 'caption', label: 'Подписи' },
	{ id: 'highlight', label: 'Подсветка' },
	{ id: 'action', label: 'Действия' },
	{ id: 'stroke', label: 'Штрихи' }
];

export interface Clip {
	track: Track;
	/** The guide step the clip belongs to. */
	step: number;
	start: number;
	end: number;
	label: string;
}

export const pauseOf = (s: GuideStep) => s.pause ?? DEFAULT_PAUSE;

/** Start time of every step and the total length, in milliseconds. */
export function times(steps: GuideStep[]): { starts: number[]; total: number } {
	const starts: number[] = [];
	let t = 0;
	for (const s of steps) {
		starts.push(t);
		t += pauseOf(s);
	}
	return { starts, total: t };
}

export const isStroke = (c: GuideCall) => c.method === 'engine.execute' && c.params.command === 'paint.stroke';

/** Points of a stroke call, `[x, y, pressure?]` in picture pixels. */
export function strokePoints(c: GuideCall): number[][] {
	const inner = (c.params.params ?? {}) as { points?: number[][] };
	return Array.isArray(inner.points) ? inner.points : [];
}

/** The clips of every track. A caption stays on screen until the next one. */
export function clips(steps: GuideStep[]): Clip[] {
	const { starts, total } = times(steps);
	const out: Clip[] = [];
	steps.forEach((s, i) => {
		const start = starts[i];
		const end = start + pauseOf(s);
		if (s.caption.trim()) {
			const next = steps.findIndex((n, j) => j > i && n.caption.trim() !== '');
			out.push({ track: 'caption', step: i, start, end: next === -1 ? total : starts[next], label: s.caption.trim() });
		}
		if (s.highlight) out.push({ track: 'highlight', step: i, start, end, label: targetLabel(s.highlight) });
		const strokes = s.calls.filter(isStroke).length;
		const actions = s.calls.filter((c) => !isStroke(c));
		if (strokes) out.push({ track: 'stroke', step: i, start, end, label: strokes === 1 ? 'Штрих' : `Штрихи ×${strokes}` });
		if (actions.length) out.push({ track: 'action', step: i, start, end, label: actions.map(describeCall).join(' · ') });
	});
	return out;
}

/** The step running at `ms` and how far into it, 0–1. */
export function frameAt(steps: GuideStep[], ms: number): { step: number; progress: number } {
	const { starts } = times(steps);
	for (let i = steps.length - 1; i >= 0; i--) {
		if (ms >= starts[i]) return { step: i, progress: Math.min(1, (ms - starts[i]) / Math.max(1, pauseOf(steps[i]))) };
	}
	return { step: -1, progress: 0 };
}

/** The caption on screen at `ms`. */
export function captionAt(steps: GuideStep[], ms: number): string {
	const { step } = frameAt(steps, ms);
	for (let i = step; i >= 0; i--) if (steps[i].caption.trim()) return steps[i].caption.trim();
	return '';
}

const toolName = (t: string) => TOOLS.find(([id]) => id.toLowerCase() === t.toLowerCase())?.[1] ?? t;

const PANELS: Record<string, string> = {
	Layers: 'Слои',
	Color: 'Цвет',
	Brushes: 'Кисти',
	BrushSettings: 'Настройки кисти',
	History: 'История',
	Properties: 'Свойства',
	Navigator: 'Навигатор',
	Toolbar: 'Инструменты',
	OptionsBar: 'Параметры инструмента'
};
const OPTIONS: Record<string, string> = {
	brushSize: 'Размер кисти',
	hardness: 'Жёсткость',
	opacity: 'Непрозрачность',
	flow: 'Нажим',
	zoom: 'Масштаб',
	blend: 'Режим наложения',
	layerOpacity: 'Непрозрачность слоя'
};

/** «tool:Brush» → «Кисть», «panel:Layers» → «Панель «Слои»». */
export function targetLabel(target: string): string {
	const [kind, name = ''] = target.split(':');
	if (kind === 'tool') return toolName(name);
	if (kind === 'panel') return `Панель «${PANELS[name] ?? name}»`;
	if (kind === 'option') return OPTIONS[name] ?? name;
	if (kind === 'menu') return `Меню: ${name}`;
	return target;
}

const pct = (v: unknown) => `${Math.round(Number(v) * 100)}%`;

/** A short Russian name for a guide call. */
export function describeCall(c: GuideCall): string {
	if (c.method === 'ui.set') {
		const p = c.params;
		if (typeof p.tool === 'string') return toolName(p.tool);
		if (p.brushSize !== undefined) return `Кисть ${p.brushSize} px`;
		if (p.zoom !== undefined) return `Масштаб ${pct(p.zoom)}`;
		if (p.fit) return 'Вписать';
		return 'Вид';
	}
	if (c.method === 'ui.pointer') return 'Движение мыши';
	const command = String(c.params.command ?? '');
	const p = (c.params.params ?? {}) as Record<string, unknown>;
	switch (command) {
		case 'layer.new.layer':
			return typeof p.name === 'string' ? `Слой «${p.name}»` : 'Новый слой';
		case 'layer.setProps':
			return p.opacity !== undefined ? `Слой ${pct(p.opacity)}` : 'Свойства слоя';
		case 'layer.select':
		case 'layer.selectTop':
		case 'layer.selectBottom':
			return 'Выбор слоя';
		case 'layer.delete':
			return 'Удалить слой';
		case 'tools.setBrush': {
			const b = ((p.brush ?? p) as Record<string, unknown>) ?? {};
			const parts = [b.size !== undefined ? `${b.size} px` : '', b.hardness !== undefined ? `жёсткость ${pct(b.hardness)}` : '', b.opacity !== undefined ? pct(b.opacity) : ''].filter(Boolean);
			return parts.length ? `Кисть ${parts.join(', ')}` : 'Кисть';
		}
		case 'tools.setColors':
			return typeof p.foreground === 'string' ? `Цвет ${p.foreground}` : 'Цвета';
		case 'tools.defaultColors':
			return 'Цвета по умолчанию';
		case 'tools.swapColors':
			return 'Поменять цвета';
		case 'select.rect':
			return 'Выделение';
		case 'select.all':
			return 'Выделить всё';
		case 'select.deselect':
			return 'Снять выделение';
		case 'select.inverse':
			return 'Инверсия';
		case 'edit.fill':
			return 'Заливка';
		case 'edit.undo':
			return 'Отмена';
		case 'edit.redo':
			return 'Повтор';
		case 'file.new':
			return 'Новый холст';
		case 'paint.stroke':
			return 'Штрих';
		default:
			return command || c.method;
	}
}

// ---------------------------------------------------------------- edits (pure)

export function setPause(steps: GuideStep[], i: number, ms: number): GuideStep[] {
	const pause = Math.round(Math.min(MAX_PAUSE, Math.max(MIN_PAUSE, ms)) / 50) * 50;
	return steps.map((s, j) => (j === i ? { ...s, pause } : s));
}

export function moveStep(steps: GuideStep[], from: number, to: number): GuideStep[] {
	if (from === to || from < 0 || to < 0 || from >= steps.length || to >= steps.length) return steps;
	const next = steps.slice();
	const [s] = next.splice(from, 1);
	next.splice(to, 0, s);
	return next;
}

export function removeStep(steps: GuideStep[], i: number): GuideStep[] {
	return steps.filter((_, j) => j !== i);
}

export function insertStep(steps: GuideStep[], at: number, step: GuideStep): GuideStep[] {
	const next = steps.slice();
	next.splice(Math.max(0, Math.min(at, steps.length)), 0, step);
	return next;
}

export interface BrushSettings {
	size: number;
	hardness: number;
	opacity: number;
	color: string;
}

/** A step that paints one stroke with `brush`; points in picture pixels. */
export function strokeStep(points: number[][], brush: BrushSettings, pause = 1200): GuideStep {
	return {
		caption: '',
		calls: [{ method: 'engine.execute', params: { command: 'paint.stroke', params: { points, size: brush.size, hardness: brush.hardness, opacity: brush.opacity, flow: 1, color: brush.color } } }],
		pause
	};
}

/** A guide starting on the lesson's own task picture or canvas. */
export function emptyScript(task: { open?: string; canvas?: { width: number; height: number; background: string } }): GuideScript {
	return { format: 1, ...(task.open ? { open: task.open } : task.canvas ? { canvas: task.canvas } : {}), steps: [] };
}

/** Parses `guide.json` text; an empty or broken text gives `null`. */
export function parseScript(text: string | null): GuideScript | null {
	if (!text?.trim()) return null;
	try {
		const s = JSON.parse(text) as GuideScript;
		return Array.isArray(s.steps) ? { ...s, steps: s.steps.map((st) => ({ ...st, caption: st.caption ?? '', calls: st.calls ?? [] })) } : null;
	} catch {
		return null;
	}
}

/** `m:ss,d` for the transport clock. */
export function clock(ms: number): string {
	const s = Math.max(0, ms) / 1000;
	const m = Math.floor(s / 60);
	const rest = s - m * 60;
	return `${String(m).padStart(2, '0')}:${rest.toFixed(1).padStart(4, '0').replace('.', ',')}`;
}
