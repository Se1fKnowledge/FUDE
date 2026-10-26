// Russian wording for numbers, times and dates.

/** Picks the Russian plural form: plural(5, ['урок', 'урока', 'уроков']) → 'уроков'. */
export function plural(n: number, forms: [string, string, string]): string {
	const abs = Math.abs(Math.trunc(n));
	const tens = abs % 100;
	const ones = abs % 10;
	if (tens >= 11 && tens <= 14) return forms[2];
	if (ones === 1) return forms[0];
	if (ones >= 2 && ones <= 4) return forms[1];
	return forms[2];
}

/** '5 уроков'. */
export function count(n: number, forms: [string, string, string]): string {
	return `${n} ${plural(n, forms)}`;
}

/** Minutes as '1 ч 55 мин', '40 мин'. */
export function duration(minutes: number): string {
	const m = Math.max(0, Math.round(minutes));
	const h = Math.floor(m / 60);
	const rest = m % 60;
	if (h === 0) return `${rest} мин`;
	return rest === 0 ? `${h} ч` : `${h} ч ${String(rest).padStart(2, '0')} мин`;
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

/** Thousands with a thin space: 9 120. */
export function thousands(n: number): string {
	return Math.round(n)
		.toString()
		.replace(/\B(?=(\d{3})+(?!\d))/g, ' ');
}

const MONTHS_SHORT = ['янв', 'фев', 'мар', 'апр', 'май', 'июн', 'июл', 'авг', 'сен', 'окт', 'ноя', 'дек'];
const MONTHS = ['января', 'февраля', 'марта', 'апреля', 'мая', 'июня', 'июля', 'августа', 'сентября', 'октября', 'ноября', 'декабря'];

function parseDate(iso: string): Date {
	const [y, mo, d] = iso.slice(0, 10).split('-').map(Number);
	return new Date(y, (mo ?? 1) - 1, d ?? 1);
}

/** '2026-10-25' → '25 окт'. */
export function shortDate(iso: string): string {
	const d = parseDate(iso);
	return `${d.getDate()} ${MONTHS_SHORT[d.getMonth()]}`;
}

/** '2026-10-25' → '25 октября'. */
export function longDate(iso: string): string {
	const d = parseDate(iso);
	return `${d.getDate()} ${MONTHS[d.getMonth()]}`;
}

export function monthShort(iso: string): string {
	return MONTHS_SHORT[parseDate(iso).getMonth()];
}

/** '2026-10-25T09:25:00' → '09:25'. */
export function timeOf(iso: string): string {
	return iso.slice(11, 16);
}

/** Lesson numbers on cards: 3 → '03'. */
export function two(n: number): string {
	return String(n).padStart(2, '0');
}
