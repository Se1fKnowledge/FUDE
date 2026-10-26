import { describe, expect, it } from 'vitest';
import { CHECK_KINDS, defaultCheck, describe as describeCheck, stepId } from './checks';

describe('lesson editor checks', () => {
	it('offers a default for every kind with the right fields', () => {
		for (const { kind, canvas } of CHECK_KINDS) {
			const check = defaultCheck(kind);
			expect(check.kind).toBe(kind);
			expect('region' in check || kind === 'similar' || !canvas).toBe(true);
		}
		expect(CHECK_KINDS).toHaveLength(13);
	});

	it('gives each new region its own array', () => {
		const a = defaultCheck('coverage');
		const b = defaultCheck('coverage');
		if (a.kind === 'coverage' && b.kind === 'coverage') {
			a.region[0] = 0.9;
			expect(b.region[0]).toBe(0.25);
		}
	});

	it('describes checks in Russian', () => {
		expect(describeCheck({ kind: 'tool', tool: 'EllipseMarquee' })).toBe('Инструмент: Овальная область');
		expect(describeCheck({ kind: 'layer', name: 'Тень' })).toBe('Есть слой «Тень»');
		expect(describeCheck({ kind: 'darker', region: [0, 0, 1, 1], than: [0, 0, 1, 1] })).toBe('Область темнее другой');
	});

	it('numbers new steps without clashes', () => {
		expect(stepId([])).toBe('step-1');
		expect(stepId(['layer', 'step-2'])).toBe('step-3');
		expect(stepId(['step-2', 'x'])).toBe('step-3');
	});
});
