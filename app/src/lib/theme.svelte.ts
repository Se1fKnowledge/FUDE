// The student's choice of theme: light «Васи» (the default), dark «Тушь», or whatever the system uses. The
// choice lives in localStorage; app.html applies it before the first paint, this module keeps it
// in step afterwards.
import { tk } from '#lib/i18n.svelte.ts';

export type ThemeChoice = 'light' | 'dark' | 'system';
export type Theme = 'light' | 'dark';

const KEY = 'fude-theme';

export const THEME_LABELS: Record<ThemeChoice, string> = {
	light: tk('Светлая — «Васи»'),
	dark: tk('Тёмная — «Тушь»'),
	system: tk('Как в системе')
};

/** The theme a choice resolves to, given whether the system prefers dark. */
export function resolve(choice: ThemeChoice, systemDark: boolean): Theme {
	return choice === 'system' ? (systemDark ? 'dark' : 'light') : choice;
}

/** The next choice when the toggle is pressed: light → dark → system → light. */
export function next(choice: ThemeChoice): ThemeChoice {
	return choice === 'light' ? 'dark' : choice === 'dark' ? 'system' : 'light';
}

export function readChoice(): ThemeChoice {
	try {
		const v = localStorage.getItem(KEY);
		return v === 'light' || v === 'dark' || v === 'system' ? v : 'light';
	} catch {
		return 'light';
	}
}

const systemDark = () => typeof matchMedia === 'function' && matchMedia('(prefers-color-scheme: dark)').matches;

export const theme = $state({ choice: 'light' as ThemeChoice, current: 'light' as Theme });

function apply() {
	theme.current = resolve(theme.choice, systemDark());
	if (typeof document !== 'undefined') document.documentElement.dataset.theme = theme.current;
}

/** Reads the saved choice and follows system changes while «Как в системе» is chosen. */
export function initTheme(): () => void {
	theme.choice = readChoice();
	apply();
	if (typeof matchMedia !== 'function') return () => {};
	const mq = matchMedia('(prefers-color-scheme: dark)');
	const onChange = () => theme.choice === 'system' && apply();
	mq.addEventListener('change', onChange);
	return () => mq.removeEventListener('change', onChange);
}

export function setTheme(choice: ThemeChoice): void {
	theme.choice = choice;
	try {
		localStorage.setItem(KEY, choice);
	} catch {
		// Blocked storage: the choice lasts until the window closes.
	}
	apply();
}
