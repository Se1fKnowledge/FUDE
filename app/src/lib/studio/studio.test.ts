import { describe, expect, it } from 'vitest';
import type { GuideStep, Shape } from '../api.ts';
import { arrowHead, bounds, cleanStyle, defaultWidth, hit, moveRegion, moveShape, regionFromDrag, shapeFromDrag, simplify, strokeToShape, styleOf, svgPath } from './geometry.ts';
import { search, TARGETS } from './targets.ts';
import { captionAt, clips, clock, describeCall, frameAt, insertStep, moveStep, parseScript, removeStep, setPause, strokeStep, targetLabel, times } from './timeline.ts';

describe('shapes on the canvas', () => {
	it('simplify keeps the ends and drops points on a straight line', () => {
		const line = Array.from({ length: 50 }, (_, i) => [i / 49, 0.5] as [number, number]);
		expect(simplify(line)).toEqual([
			[0, 0.5],
			[1, 0.5]
		]);
		const bent = [[0, 0], [0.5, 0.5], [1, 0]] as [number, number][];
		expect(simplify(bent)).toEqual(bent);
	});

	it('drags make the shape of the tool, inside the picture', () => {
		expect(shapeFromDrag('line', [0.1, 0.2], [0.5, 0.6])).toEqual({ kind: 'line', from: [0.1, 0.2], to: [0.5, 0.6] });
		expect(shapeFromDrag('arrow', [0.1, 0.2], [1.4, 0.6])).toEqual({ kind: 'arrow', from: [0.1, 0.2], to: [1, 0.6] });
		expect(shapeFromDrag('rect', [0.5, 0.6], [0.1, 0.2])).toEqual({ kind: 'rect', rect: [0.1, 0.2, 0.4, 0.4] });
		expect(shapeFromDrag('ellipse', [0.3, 0.3], [0.301, 0.3])).toBeNull();
		expect(shapeFromDrag('path', [0, 0], [0, 0], [[0.1, 0.1]])).toBeNull();
		const path = shapeFromDrag('path', [0, 0], [0, 0], [[0.1, 0.1], [0.2, 0.3], [0.4, 0.1]]);
		expect(path?.kind).toBe('path');
	});

	it('regions keep a minimum size and stay inside', () => {
		expect(regionFromDrag([0.995, 0.5], [0.995, 0.5])).toEqual([0.99, 0.5, 0.01, 0.01]);
		expect(moveRegion([0.8, 0.8, 0.3, 0.1], 0.5, 0.5)).toEqual([0.7, 0.9, 0.3, 0.1]);
	});

	it('styles default to the hint look and only differences are saved', () => {
		const s: Shape = { kind: 'line', from: [0, 0], to: [1, 1] };
		expect(styleOf(s, 1000, 500)).toEqual({ color: '#BC3823', width: 3, opacity: 0.35, dashed: false, reveal: false });
		expect(defaultWidth(100, 100)).toBe(2);
		expect(cleanStyle({ color: '#bc3823', width: 3, opacity: 0.35, dashed: false }, 1000, 500)).toBeUndefined();
		expect(cleanStyle({ color: '#2f5d8c', width: 8, opacity: 0.6, dashed: true }, 1000, 500)).toEqual({ color: '#2F5D8C', width: 8, opacity: 0.6, dashed: true });
	});

	it('hit finds the nearest shape and moving keeps it on the picture', () => {
		const shapes: Shape[] = [
			{ kind: 'line', from: [0, 0.5], to: [1, 0.5] },
			{ kind: 'rect', rect: [0.2, 0.2, 0.2, 0.2] }
		];
		expect(hit(shapes, [0.5, 0.505])).toBe(0);
		expect(hit(shapes, [0.3, 0.205])).toBe(1);
		expect(hit(shapes, [0.9, 0.9])).toBe(-1);
		const moved = moveShape(shapes[1], 0.9, -0.5);
		expect(bounds(moved)).toEqual([0.8, 0, 0.2, 0.2]);
	});

	it('an arrow has a head pointing back along it', () => {
		const head = arrowHead([0, 0], [100, 0], 3);
		expect(head).toHaveLength(2);
		for (const [tip, end] of head) {
			expect(tip).toEqual([100, 0]);
			expect(end[0]).toBeLessThan(100);
		}
		expect(svgPath({ kind: 'arrow', from: [0, 0], to: [1, 0] }, 100, 100).match(/M/g)).toHaveLength(3);
	});

	it('a guide stroke becomes a ghost line in fractions', () => {
		const shape = strokeToShape([[0, 50, 1], [50, 50, 1], [100, 50, 1]], 200, 100, 12);
		expect(shape).toEqual({ kind: 'path', points: [[0, 0.5], [0.5, 0.5]], style: { width: 12 } });
		expect(strokeToShape([[1, 1]], 10, 10)).toBeNull();
	});
});

const caption = (text: string, pause?: number): GuideStep => ({ caption: text, calls: [], pause });
const stroke = strokeStep([[0, 0, 1], [10, 10, 1]], { size: 60, hardness: 0, opacity: 0.3, color: '#3B3A40' }, 1000);

describe('guide timeline', () => {
	it('lays steps end to end with the default pause', () => {
		expect(times([caption('a'), caption('', 1000), caption('b', 300)])).toEqual({ starts: [0, 600, 1600], total: 1900 });
	});

	it('captions last until the next one, strokes and actions get their own tracks', () => {
		const steps: GuideStep[] = [
			{ caption: 'Слой для тени', highlight: 'panel:Layers', calls: [{ method: 'engine.execute', params: { command: 'layer.new.layer', params: { name: 'Тень' } } }], pause: 900 },
			stroke,
			caption('Растушёвываем', 500)
		];
		const c = clips(steps);
		expect(c.find((x) => x.track === 'caption' && x.step === 0)).toMatchObject({ start: 0, end: 1900, label: 'Слой для тени' });
		expect(c.find((x) => x.track === 'highlight')).toMatchObject({ label: 'Панель «Слои»', end: 900 });
		expect(c.find((x) => x.track === 'action')).toMatchObject({ label: 'Слой «Тень»' });
		expect(c.find((x) => x.track === 'stroke')).toMatchObject({ step: 1, start: 900, end: 1900, label: 'Штрих' });
		expect(captionAt(steps, 1500)).toBe('Слой для тени');
		expect(captionAt(steps, 2000)).toBe('Растушёвываем');
		expect(frameAt(steps, 1400)).toEqual({ step: 1, progress: 0.5 });
		expect(frameAt([], 10)).toEqual({ step: -1, progress: 0 });
	});

	it('edits keep the list consistent', () => {
		const steps = [caption('a'), caption('b'), caption('c')];
		expect(moveStep(steps, 0, 2).map((s) => s.caption)).toEqual(['b', 'c', 'a']);
		expect(moveStep(steps, 0, 5)).toBe(steps);
		expect(removeStep(steps, 1).map((s) => s.caption)).toEqual(['a', 'c']);
		expect(insertStep(steps, 1, caption('x')).map((s) => s.caption)).toEqual(['a', 'x', 'b', 'c']);
		expect(setPause(steps, 1, 99999)[1].pause).toBe(15000);
		expect(setPause(steps, 1, 1234)[1].pause).toBe(1250);
	});

	it('calls and targets read in Russian', () => {
		expect(targetLabel('tool:Brush')).toBe('Кисть');
		expect(targetLabel('panel:Layers')).toBe('Панель «Слои»');
		expect(targetLabel('option:hardness')).toBe('Жёсткость');
		expect(describeCall({ method: 'ui.set', params: { tool: 'Eraser' } })).toBe('Ластик');
		expect(describeCall({ method: 'engine.execute', params: { command: 'tools.setBrush', params: { brush: { size: 60, hardness: 0 } } } })).toBe('Кисть 60 px, жёсткость 0%');
		expect(describeCall({ method: 'engine.execute', params: { command: 'something.else' } })).toBe('something.else');
	});

	it('parses guide text and formats the clock', () => {
		expect(parseScript('')).toBeNull();
		expect(parseScript('{oops')).toBeNull();
		expect(parseScript('{"format":1,"steps":[{"calls":[]}]}')?.steps[0].caption).toBe('');
		expect(clock(5100)).toBe('00:05,1');
		expect(clock(65000)).toBe('01:05,0');
	});
});

describe('interface map targets', () => {
	it('finds targets by Russian name or id', () => {
		expect(search('кисть').map((t) => t.id)).toContain('tool:Brush');
		expect(search('слои').map((t) => t.id)).toContain('panel:Layers');
		expect(search('HARD').map((t) => t.id)).toEqual(['option:hardness']);
		expect(search('  ')).toHaveLength(TARGETS.length);
		expect(search('ластик').every((t) => t.group === 'tool')).toBe(true);
	});
});
