// The interface language: Russian (the source language) or English. Russian strings stay in the
// code as they are and serve as keys: t('Главная') gives 'Home' in English, and a missing entry
// falls back to the Russian text. A test checks that every t() string has an English entry and
// that no Russian text in the interface bypasses t().

import { EN } from './i18n/en.ts';

export type Lang = 'ru' | 'en';
export const LANGS: { code: Lang; label: string; short: string }[] = [
	{ code: 'ru', label: 'Русский', short: 'RU' },
	{ code: 'en', label: 'English', short: 'EN' }
];

const KEY = 'fude-lang';

/** The saved choice, or the system language on first launch (Russian systems get Russian). */
export function readLang(): Lang {
	// Outside a browser (tests, server rendering) the source language.
	if (typeof localStorage === 'undefined') return 'ru';
	try {
		const v = localStorage.getItem(KEY);
		if (v === 'ru' || v === 'en') return v;
	} catch {
		// Blocked storage: fall through to the system language.
	}
	const sys = typeof navigator !== 'undefined' ? (navigator.language ?? '') : '';
	return sys.toLowerCase().startsWith('ru') || sys === '' ? 'ru' : 'en';
}

// Read at import, so lists of labels built when a module loads come out in the right language.
// Changing the language reloads the window (see setLang), so nothing needs to re-translate live.
export const i18n = $state({ lang: readLang() });

type Vars = Record<string, string | number>;

/** Fills {name} placeholders. */
export function fill(text: string, vars?: Vars): string {
	return vars ? text.replace(/\{(\w+)\}/g, (m, k: string) => (k in vars ? String(vars[k]) : m)) : text;
}

/** The interface text in the current language. `ru` is the Russian original and the key. */
export function t(ru: string, vars?: Vars): string {
	return fill(i18n.lang === 'en' ? (EN[ru] ?? ru) : ru, vars);
}

/** A name in the quotes of the current language: «Тень» in Russian, “Shadow” in English. */
export function quote(name: string): string {
	return i18n.lang === 'en' ? `“${name}”` : `«${name}»`;
}

/** Marks a Russian label kept in data (a menu list, a lookup table) as a translation key without
 * translating it yet; show it with t(label). The coverage test finds these too. */
export const tk = (ru: string): string => ru;

/** Text in a given language, for tests and for places that must not follow the switch. */
export function tIn(lang: Lang, ru: string, vars?: Vars): string {
	return fill(lang === 'en' ? (EN[ru] ?? ru) : ru, vars);
}

let listener: ((lang: Lang) => unknown) | null = null;

/** Called with the language at start and on every change, e.g. to tell the backend and the editor. */
export function onLangChange(fn: (lang: Lang) => unknown): void {
	listener = fn;
}

/** Applies the language at start: the page's lang attribute and the listener. */
export async function initLang(): Promise<void> {
	if (typeof document !== 'undefined') document.documentElement.lang = i18n.lang;
	await listener?.(i18n.lang);
}

/** Saves the choice, tells the backend and the editor (the listener), then reloads the window so
 * every screen and every list of labels comes back in the new language. */
export async function setLang(lang: Lang): Promise<void> {
	if (lang === i18n.lang) return;
	try {
		localStorage.setItem(KEY, lang);
	} catch {
		// Blocked storage: the choice lasts until the window closes.
	}
	i18n.lang = lang;
	try {
		await listener?.(lang);
	} finally {
		if (typeof location !== 'undefined') location.reload();
	}
}
