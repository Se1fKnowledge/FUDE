import { afterEach, describe, expect, it, vi } from 'vitest';
import { next, readChoice, resolve } from './theme.svelte.ts';

describe('theme choice', () => {
	afterEach(() => vi.unstubAllGlobals());

	it('resolves «system» through the system preference', () => {
		expect(resolve('light', true)).toBe('light');
		expect(resolve('dark', false)).toBe('dark');
		expect(resolve('system', true)).toBe('dark');
		expect(resolve('system', false)).toBe('light');
	});

	it('cycles light → dark → system', () => {
		expect(next('light')).toBe('dark');
		expect(next('dark')).toBe('system');
		expect(next('system')).toBe('light');
	});

	it('reads a saved choice and falls back to the light theme', () => {
		const store = new Map<string, string>([['fude-theme', 'dark']]);
		vi.stubGlobal('localStorage', { getItem: (k: string) => store.get(k) ?? null, setItem: () => {} });
		expect(readChoice()).toBe('dark');
		store.set('fude-theme', 'purple');
		expect(readChoice()).toBe('light');
		vi.stubGlobal('localStorage', {
			getItem: () => {
				throw new Error('blocked');
			}
		});
		expect(readChoice()).toBe('light');
	});
});
