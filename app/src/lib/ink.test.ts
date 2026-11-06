import { describe, expect, it } from 'vitest';
import { BARS, DABS, ENSO, maskUrl, MIST, mistWindow, pick, seedOf, STROKES, TAPS } from './ink';

describe('ink pictures', () => {
	it('ships every stroke, ensō and mountain layer', () => {
		expect(STROKES).toHaveLength(8);
		expect(ENSO).toHaveLength(4);
		for (const src of [...STROKES, ...ENSO, MIST.far, MIST.mid, MIST.near]) expect(src).toMatch(/\.webp/);
		expect(new Set(STROKES).size).toBe(STROKES.length);
		expect(DABS).toHaveLength(5);
		expect(TAPS.length).toBeGreaterThan(1);
		expect(BARS.length).toBeGreaterThan(1);
		expect(maskUrl(DABS[0])).toBe(`url("${DABS[0]}")`);
	});

	it('gives a day the same tap every time, and days differ', () => {
		expect(seedOf('2026-11-06')).toBe(seedOf('2026-11-06'));
		expect(seedOf('2026-11-06')).toBeGreaterThanOrEqual(0);
		const taps = new Set(['2026-11-01', '2026-11-02', '2026-11-03', '2026-11-04', '2026-11-05', '2026-11-06'].map((d) => pick(TAPS, seedOf(d))));
		expect(taps.size).toBeGreaterThan(1);
	});

	it('picks the same picture for the same seed, any seed', () => {
		expect(pick(STROKES, 5)).toBe(pick(STROKES, 5));
		expect(pick(STROKES, 5 + STROKES.length)).toBe(pick(STROKES, 5));
		expect(pick(ENSO, -3)).toBe(ENSO[3]);
		expect(pick(ENSO, 2.7)).toBe(ENSO[2]);
		expect(pick(STROKES, 0)).toBe(STROKES[0]);
	});
});

describe('mountain window', () => {
	it('keeps the picture covering the window', () => {
		for (const seed of [0, 1, 7, 42, 1000]) {
			const v = mistWindow(520, 150, seed);
			expect(v.width).toBeCloseTo((150 * MIST.width) / MIST.height);
			expect(v.x).toBeLessThanOrEqual(0);
			expect(v.x + v.width).toBeGreaterThanOrEqual(520);
		}
	});

	it('shows different hills for different seeds', () => {
		const xs = new Set([1, 2, 3, 4, 5].map((s) => mistWindow(520, 150, s).x));
		expect(xs.size).toBeGreaterThan(3);
	});

	it('stretches the picture over a window wider than it', () => {
		expect(mistWindow(1440, 100, 3)).toEqual({ width: 1440, x: 0 });
	});
});
