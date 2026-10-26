import { describe, expect, it } from 'vitest';
import { clock, count, duration, longDate, percent, plural, shortDate, thousands, timeOf, two } from './format';

describe('plural', () => {
	const forms: [string, string, string] = ['урок', 'урока', 'уроков'];
	it.each([
		[1, 'урок'],
		[2, 'урока'],
		[4, 'урока'],
		[5, 'уроков'],
		[11, 'уроков'],
		[12, 'уроков'],
		[14, 'уроков'],
		[21, 'урок'],
		[22, 'урока'],
		[111, 'уроков'],
		[0, 'уроков']
	])('%i → %s', (n, word) => expect(plural(n, forms)).toBe(word));

	it('counts', () => expect(count(6, forms)).toBe('6 уроков'));
});

describe('time', () => {
	it('formats durations like the mockup', () => {
		expect(duration(40)).toBe('40 мин');
		expect(duration(115)).toBe('1 ч 55 мин');
		expect(duration(65)).toBe('1 ч 05 мин');
		expect(duration(120)).toBe('2 ч');
		expect(duration(-3)).toBe('0 мин');
	});

	it('formats clocks', () => {
		expect(clock(468)).toBe('7:48');
		expect(clock(5)).toBe('0:05');
		expect(clock(3725)).toBe('1:02:05');
	});
});

describe('numbers and dates', () => {
	it('formats percent and thousands', () => {
		expect(percent(0.923)).toBe('92%');
		expect(thousands(9120)).toBe('9 120');
		expect(thousands(950)).toBe('950');
		expect(thousands(1234567)).toBe('1 234 567');
	});

	it('formats dates without time zone shifts', () => {
		expect(shortDate('2026-10-25')).toBe('25 окт');
		expect(longDate('2026-01-01T09:00:00')).toBe('1 января');
		expect(timeOf('2026-10-25T09:25:00')).toBe('09:25');
		expect(two(3)).toBe('03');
	});
});
