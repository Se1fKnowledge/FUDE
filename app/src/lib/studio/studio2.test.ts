import { describe, expect, it } from 'vitest';
import type { GuideStep, Shape } from '../api.ts';
import { cleanStyle, handles, moveHandle, styleOf } from './geometry.ts';
import { History } from './history.ts';
import { clips, isStroke, splitStep, strokePoints, strokeStep, taskStepStart } from './timeline.ts';

describe('undo and redo', () => {
	it('walks back and forth through recorded versions', () => {
		const h = new History();
		h.reset('a');
		h.record('b');
		h.record('b');
		h.record('c');
		expect(h.undo()).toBe('b');
		expect(h.undo()).toBe('a');
		expect(h.undo()).toBeNull();
		expect(h.canRedo).toBe(true);
		expect(h.redo()).toBe('b');
		h.record('x');
		expect(h.canRedo).toBe(false);
		expect(h.undo()).toBe('b');
	});

	it('keeps at most the limit', () => {
		const h = new History(3);
		h.reset('0');
		for (const v of ['1', '2', '3', '4', '5']) h.record(v);
		expect([h.undo(), h.undo(), h.undo(), h.undo()]).toEqual(['4', '3', '2', null]);
	});
});

describe('shape handles', () => {
	it('lines move their ends, boxes grow from the opposite corner', () => {
		const line: Shape = { kind: 'arrow', from: [0.1, 0.1], to: [0.5, 0.5] };
		expect(handles(line)).toEqual([
			[0.1, 0.1],
			[0.5, 0.5]
		]);
		expect(moveHandle(line, 1, [0.9, 1.2])).toEqual({ kind: 'arrow', from: [0.1, 0.1], to: [0.9, 1] });
		const box: Shape = { kind: 'ellipse', rect: [0.2, 0.2, 0.2, 0.2] };
		expect(handles(box)).toHaveLength(4);
		expect(moveHandle(box, 2, [0.6, 0.5])).toEqual({ kind: 'ellipse', rect: [0.2, 0.2, 0.4, 0.3] });
		expect(moveHandle(box, 0, [0.5, 0.5])).toEqual({ kind: 'ellipse', rect: [0.4, 0.4, 0.1, 0.1] });
		const path: Shape = { kind: 'path', points: [[0, 0], [1, 1]] };
		expect(handles(path)).toEqual([]);
		expect(moveHandle(path, 0, [0.5, 0.5])).toBe(path);
	});

	it('reveal is part of the style and saved only when on', () => {
		const s: Shape = { kind: 'line', from: [0, 0], to: [1, 1], style: { reveal: true } };
		expect(styleOf(s, 100, 100).reveal).toBe(true);
		expect(cleanStyle({ reveal: true }, 100, 100)).toEqual({ reveal: true });
		expect(cleanStyle({ reveal: false }, 100, 100)).toBeUndefined();
	});
});

const brush = { size: 20, hardness: 0.5, opacity: 1, color: '#000000' };

describe('task steps on the timeline', () => {
	it('neighbouring guide steps that show one task step make one clip', () => {
		const steps: GuideStep[] = [
			{ caption: 'a', calls: [], pause: 500, task_step: 'layer' },
			{ caption: '', calls: [], pause: 500, task_step: 'layer' },
			{ caption: '', calls: [], pause: 500 },
			{ caption: '', calls: [], pause: 500, task_step: 'shadow' }
		];
		const task = clips(steps, { layer: '1 · Слой', shadow: '2 · Тень' }).filter((c) => c.track === 'task');
		expect(task).toEqual([
			{ track: 'task', step: 0, start: 0, end: 1000, label: '1 · Слой' },
			{ track: 'task', step: 3, start: 1500, end: 2000, label: '2 · Тень' }
		]);
		expect(taskStepStart(steps, 'shadow')).toBe(1500);
		expect(taskStepStart(steps, 'nope')).toBeNull();
	});
});

describe('splitting a guide step', () => {
	it('splits a single stroke into two strokes at the playhead', () => {
		const points = Array.from({ length: 11 }, (_, i) => [i * 10, 0, 1]);
		const steps = [strokeStep(points, brush, 1000)];
		const out = splitStep(steps, 0, 300);
		expect(out).toHaveLength(2);
		expect(out.map((s) => s.pause)).toEqual([300, 700]);
		expect(out.every((s) => s.calls.length === 1 && isStroke(s.calls[0]))).toBe(true);
		const [a, b] = out.map((s) => strokePoints(s.calls[0]));
		expect(a[a.length - 1]).toEqual(b[0]);
		expect(a[0]).toEqual([0, 0, 1]);
		expect(b[b.length - 1]).toEqual([100, 0, 1]);
	});

	it('splits any other step into its calls and a pause', () => {
		const steps: GuideStep[] = [{ caption: 'Слой', calls: [{ method: 'engine.execute', params: { command: 'layer.new.layer' } }], pause: 1200, task_step: 'layer' }];
		const out = splitStep(steps, 0, 400);
		expect(out).toEqual([
			{ caption: 'Слой', calls: steps[0].calls, pause: 400, task_step: 'layer' },
			{ caption: '', calls: [], pause: 800, task_step: 'layer' }
		]);
		expect(splitStep([{ caption: '', calls: [], pause: 150 }], 0, 50)).toHaveLength(1);
		expect(splitStep(steps, 5, 100)).toBe(steps);
	});
});
