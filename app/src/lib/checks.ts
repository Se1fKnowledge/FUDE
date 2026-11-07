// What the lesson editor offers for each check kind (docs/lesson-format.md, «Проверки»).
import { quote, t } from '#lib/i18n.svelte.ts';
import type { Check, CheckKind } from './api.ts';

export const CHECK_KINDS: { kind: CheckKind['kind']; label: string; canvas: boolean }[] = [
	{ kind: 'tool', label: t('Инструмент'), canvas: false },
	{ kind: 'brush', label: t('Параметры кисти'), canvas: false },
	{ kind: 'zoom', label: t('Масштаб'), canvas: false },
	{ kind: 'layers', label: t('Число слоёв'), canvas: false },
	{ kind: 'layer', label: t('Есть слой'), canvas: false },
	{ kind: 'selection', label: t('Выделение'), canvas: false },
	{ kind: 'history', label: t('История и отмена'), canvas: false },
	{ kind: 'foreground', label: t('Основной цвет'), canvas: false },
	{ kind: 'coverage', label: t('Закрашена область'), canvas: true },
	{ kind: 'luminance', label: t('Яркость области'), canvas: true },
	{ kind: 'darker', label: t('Область темнее другой'), canvas: true },
	{ kind: 'color', label: t('Цвет области'), canvas: true },
	{ kind: 'similar', label: t('Похоже на эталон'), canvas: true }
];

/** Tools a check or a hint usually points at, with their Russian names. */
export const TOOLS: [string, string][] = [
	['Brush', t('Кисть')],
	['Pencil', t('Карандаш')],
	['Eraser', t('Ластик')],
	['RectMarquee', t('Прямоугольная область')],
	['EllipseMarquee', t('Овальная область')],
	['Lasso', t('Лассо')],
	['MagicWand', t('Волшебная палочка')],
	['Move', t('Перемещение')],
	['PaintBucket', t('Заливка')],
	['Gradient', t('Градиент')],
	['Eyedropper', t('Пипетка')],
	['Smudge', t('Палец')],
	['Blur', t('Размытие')],
	['Dodge', t('Осветлитель')],
	['Burn', t('Затемнитель')],
	['Zoom', t('Масштаб')],
	['Hand', t('Рука')]
];

const WHOLE: [number, number, number, number] = [0.25, 0.25, 0.5, 0.5];

/** A sensible new check of a kind, valid as soon as it is added. */
export function defaultCheck(kind: CheckKind['kind']): Check {
	switch (kind) {
		case 'tool':
			return { kind, tool: 'Brush' };
		case 'brush':
			return { kind, hardness: { max: 0.3 } };
		case 'zoom':
			return { kind, zoom: { min: 2 } };
		case 'layers':
			return { kind, count: { min: 2 } };
		case 'layer':
			return { kind, name: t('Тень') };
		case 'selection':
			return { kind, present: true };
		case 'history':
			return { kind, can_redo: true };
		case 'foreground':
			return { kind, color: '#BC3823', tolerance: 0.12 };
		case 'coverage':
			return { kind, region: [...WHOLE], changed: { min: 0.3 } };
		case 'luminance':
			return { kind, region: [...WHOLE], value: { max: 0.5 } };
		case 'darker':
			return { kind, region: [0.55, 0.5, 0.1, 0.1], than: [0.35, 0.3, 0.1, 0.1], by: 0.1 };
		case 'color':
			return { kind, region: [...WHOLE], color: '#2F5D8C', tolerance: 0.15 };
		case 'similar':
			return { kind, score: { min: 0.8 } };
	}
}

/** A short Russian summary of a check, for the step list. */
export function describe(check: Check): string {
	const label = CHECK_KINDS.find((k) => k.kind === check.kind)?.label ?? check.kind;
	switch (check.kind) {
		case 'tool':
			return `${label}: ${TOOLS.find(([id]) => id === check.tool)?.[1] ?? check.tool}`;
		case 'layer':
			return check.name ? `${label} ${quote(check.name)}` : label;
		case 'foreground':
		case 'color':
			return `${label} ${check.color}`;
		default:
			return label;
	}
}

/** A step id from its position: step-1, step-2… unique within the lesson. */
export function stepId(existing: string[]): string {
	let n = existing.length + 1;
	while (existing.includes(`step-${n}`)) n++;
	return `step-${n}`;
}
