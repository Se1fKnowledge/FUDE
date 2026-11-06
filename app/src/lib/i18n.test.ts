import { describe, expect, it } from 'vitest';
import { fill, tIn } from './i18n.svelte.ts';
import { EN, EN_PLURALS } from './i18n/en.ts';

// Every interface file, read as text: the checks below are about the source, not the runtime.
const raw = import.meta.glob<string>('/src/**/*.{svelte,ts}', { query: '?raw', import: 'default', eager: true });
const sources = Object.entries(raw)
	.filter(([p]) => !/\.test\.ts$|\/i18n\/|i18n\.svelte\.ts$/.test(p))
	.map(([p, text]) => ({ p, text }));

const KEY = /\b(?:t|tk)\(\s*(['"`])((?:\\.|(?!\1).)*)\1/g;
const FORMS = /\b(?:count|plural)\([^[\]]*\[\s*(['"])((?:\\.|(?!\1).)*)\1/g;
const unescape = (s: string) => s.replace(/\\n/g, '\n').replace(/\\(['"`\\])/g, '$1');
const placeholders = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe('interface translation', () => {
	const keys = new Set<string>();
	const forms = new Set<string>();
	for (const { text } of sources) {
		for (const m of text.matchAll(KEY)) keys.add(unescape(m[2]));
		for (const m of text.matchAll(FORMS)) forms.add(unescape(m[2]));
	}

	it('finds the strings it checks', () => {
		expect(keys.size).toBeGreaterThan(400);
		expect(forms.size).toBeGreaterThan(5);
	});

	it('has an English entry for every t() and tk() string, with the same placeholders', () => {
		const missing = [...keys].filter((k) => !(k in EN));
		expect(missing).toEqual([]);
		const mismatched = [...keys].filter((k) => k in EN && placeholders(k).join() !== placeholders(EN[k]).join());
		expect(mismatched).toEqual([]);
	});

	it('has English plural forms for every count() and plural() word', () => {
		expect([...forms].filter((f) => !(f in EN_PLURALS))).toEqual([]);
	});

	it('keeps no English entries nobody uses', () => {
		expect(Object.keys(EN).filter((k) => !keys.has(k))).toEqual([]);
	});

	it('leaves no Russian text outside t()', () => {
		const cyr = /[а-яё]/i;
		const left: string[] = [];
		for (const { p, text } of sources) {
			let style = false;
			text.split('\n').forEach((line, i) => {
				const s = line.trim();
				if (s.startsWith('<style')) style = true;
				if (style) {
					if (s.startsWith('</style')) style = false;
					return;
				}
				if (/^(\/\/|\/\*|\*|<!--)/.test(s) || line.includes('i18n-ignore')) return;
				const code = line
					.replace(/\s\/\/\s.*$/, '')
					.replace(KEY, '')
					.replace(/\b(?:count|plural)\([^[\]]*\[[^\]]*\]/g, '');
				if (cyr.test(code)) left.push(`${p}:${i + 1}: ${s.slice(0, 80)}`);
			});
		}
		expect(left).toEqual([]);
	});
});

describe('t()', () => {
	it('gives the English text and fills placeholders', () => {
		expect(tIn('en', 'Главная')).toBe('Home');
		expect(tIn('ru', 'Главная')).toBe('Главная');
		expect(tIn('en', 'Шаг {n} из {of}', { n: 2, of: 5 })).toBe('Step 2 of 5');
		expect(tIn('ru', 'Шаг {n} из {of}', { n: 2, of: 5 })).toBe('Шаг 2 из 5');
	});

	it('falls back to the Russian text and keeps unknown placeholders', () => {
		expect(tIn('en', 'Строка без перевода')).toBe('Строка без перевода');
		expect(fill('{a} и {b}', { a: 1 })).toBe('1 и {b}');
	});
});
