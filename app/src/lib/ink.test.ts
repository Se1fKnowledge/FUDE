import { describe, expect, it } from 'vitest';
import { brushStroke, enso, ridge, rng } from './ink';

/** Every coordinate pair of an SVG path made of M/L/Z commands. */
function points(d: string): [number, number][] {
	return d
		.replace(/[MZ]/g, ' ')
		.split('L')
		.map((p) => p.trim())
		.filter(Boolean)
		.map((p) => p.split(/\s+/).map(Number) as [number, number]);
}

describe('rng', () => {
	it('is seeded and stays in [0, 1)', () => {
		const a = rng(42);
		const b = rng(42);
		const values = Array.from({ length: 1000 }, () => a());
		expect(values).toEqual(Array.from({ length: 1000 }, () => b()));
		expect(values.every((v) => v >= 0 && v < 1)).toBe(true);
		expect(rng(1)()).not.toBe(rng(2)());
	});
});

describe('brush stroke', () => {
	it('stays inside its box and is repeatable', () => {
		const s = brushStroke(220, 44, 5);
		for (const [x, y] of points(s.body)) {
			expect(x).toBeGreaterThanOrEqual(0);
			expect(x).toBeLessThanOrEqual(220);
			expect(y).toBeGreaterThanOrEqual(0);
			expect(y).toBeLessThanOrEqual(44);
		}
		expect(s.hairs.length).toBeGreaterThanOrEqual(4);
		expect(brushStroke(220, 44, 5)).toEqual(s);
		expect(brushStroke(220, 44, 6)).not.toEqual(s);
	});
});

describe('enso', () => {
	it('rings the centre without closing', () => {
		const d = enso(100, 100, 80, 14);
		const pts = points(d);
		for (const [x, y] of pts) {
			const dist = Math.hypot(x - 100, y - 100);
			expect(dist).toBeGreaterThan(60);
			expect(dist).toBeLessThan(100);
		}
		const [sx, sy] = pts[0];
		const [ex, ey] = pts[pts.length / 2 - 1];
		expect(Math.hypot(sx - ex, sy - ey)).toBeGreaterThan(10);
	});
});

describe('ridge', () => {
	it('fills down to the bottom edge', () => {
		const d = ridge(400, 100, 0.5, 0.1);
		const pts = points(d);
		expect(pts[0]).toEqual([0, 100]);
		expect(pts[pts.length - 1]).toEqual([400, 100]);
		for (const [, y] of pts) expect(y).toBeLessThanOrEqual(100);
	});
});
