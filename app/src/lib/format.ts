// Numbers, times and dates in the interface language.
import { i18n } from './i18n.svelte.ts';
import { EN_PLURALS } from './i18n/en.ts';

/** Picks the Russian plural form: plural(5, ['урок', 'урока', 'уроков']) → 'уроков'. In English
 * the forms come from EN_PLURALS, keyed by the Russian singular: 'lesson' / 'lessons'. */
export function plural(n: number, forms: [string, string, string]): string {
	const abs = Math.abs(Math.trunc(n));
	if (i18n.lang === 'en') {
		const en = EN_PLURALS[forms[0]];
		if (en) return abs === 1 ? en[0] : en[1];
	}
	const tens = abs % 100;
	const ones = abs % 10;
	if (tens >= 11 && tens <= 14) return forms[2];
	if (ones === 1) return forms[0];
	if (ones >= 2 && ones <= 4) return forms[1];
	return forms[2];
}

/** '5 уроков' / '5 lessons'. */
export function count(n: number, forms: [string, string, string]): string {
	return `${n} ${plural(n, forms)}`;
}

/** Minutes as '1 ч 55 мин', '40 мин' (English: '1 h 55 min'). */
export function duration(minutes: number): string {
	const m = Math.max(0, Math.round(minutes));
	const h = Math.floor(m / 60);
	const rest = m % 60;
	const [hu, mu] = i18n.lang === 'en' ? ['h', 'min'] : ['ч', 'мин']; // i18n-ignore: the Russian units
	if (h === 0) return `${rest} ${mu}`;
	return rest === 0 ? `${h} ${hu}` : `${h} ${hu} ${String(rest).padStart(2, '0')} ${mu}`;
}

/** Seconds as a clock: 7:48, 1:02:05. */
export function clock(seconds: number): string {
	const s = Math.max(0, Math.round(seconds));
	const h = Math.floor(s / 3600);
	const m = Math.floor((s % 3600) / 60);
	const sec = String(s % 60).padStart(2, '0');
	return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${sec}` : `${m}:${sec}`;
}

/** 0.923 → '92%'. */
export function percent(share: number): string {
	return `${Math.round(share * 100)}%`;
}

/** Thousands with a narrow space (Russian) or a comma (English): 9 120 / 9,120. */
export function thousands(n: number): string {
	return Math.round(n)
		.toString()
		.replace(/\B(?=(\d{3})+(?!\d))/g, i18n.lang === 'en' ? ',' : ' ');
}

const MONTHS_SHORT = {
	ru: ['янв', 'фев', 'мар', 'апр', 'май', 'июн', 'июл', 'авг', 'сен', 'окт', 'ноя', 'дек'], // i18n-ignore
	en: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
};
const MONTHS = {
	ru: ['января', 'февраля', 'марта', 'апреля', 'мая', 'июня', 'июля', 'августа', 'сентября', 'октября', 'ноября', 'декабря'], // i18n-ignore
	en: ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December']
};

function parseDate(iso: string): Date {
	const [y, mo, d] = iso.slice(0, 10).split('-').map(Number);
	return new Date(y, (mo ?? 1) - 1, d ?? 1);
}

/** '2026-10-25' → '25 окт' / 'Oct 25'. */
export function shortDate(iso: string): string {
	const d = parseDate(iso);
	return i18n.lang === 'en' ? `${MONTHS_SHORT.en[d.getMonth()]} ${d.getDate()}` : `${d.getDate()} ${MONTHS_SHORT.ru[d.getMonth()]}`;
}

/** '2026-10-25' → '25 октября' / 'October 25'. */
export function longDate(iso: string): string {
	const d = parseDate(iso);
	return i18n.lang === 'en' ? `${MONTHS.en[d.getMonth()]} ${d.getDate()}` : `${d.getDate()} ${MONTHS.ru[d.getMonth()]}`;
}

export function monthShort(iso: string): string {
	return MONTHS_SHORT[i18n.lang][parseDate(iso).getMonth()];
}

/** '2026-10-25T09:25:00' → '09:25'. */
export function timeOf(iso: string): string {
	return iso.slice(11, 16);
}

/** Lesson numbers on cards: 3 → '03'. */
export function two(n: number): string {
	return String(n).padStart(2, '0');
}
