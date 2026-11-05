// Places in the PhotoCraft window a hint can point at («куда нажать»), grouped the way the
// interface map shows them. The ids are the ones the lesson format accepts (validate.rs).
import { TOOLS } from '../checks.ts';
import { targetLabel } from './timeline.ts';

export type TargetGroup = 'tool' | 'panel' | 'option' | 'menu';

export interface Target {
	id: string;
	group: TargetGroup;
	label: string;
}

export const GROUP_LABELS: Record<TargetGroup, string> = {
	tool: 'Инструменты',
	panel: 'Панели',
	option: 'Параметры инструмента',
	menu: 'Команды меню'
};

export const PANELS = ['Layers', 'Color', 'Brushes', 'History', 'Properties', 'Navigator'];
export const OPTIONS = ['brushSize', 'hardness', 'opacity', 'flow', 'blend', 'layerOpacity'];
export const MENU: [string, string][] = [
	['edit.undo', 'Отменить'],
	['edit.redo', 'Повторить'],
	['select.all', 'Выделить всё'],
	['select.deselect', 'Снять выделение'],
	['select.inverse', 'Инверсия выделения'],
	['layer.new.layer', 'Новый слой'],
	['edit.fill', 'Заливка']
];

export const TARGETS: Target[] = [
	...TOOLS.map(([id, label]) => ({ id: `tool:${id}`, group: 'tool' as const, label })),
	...PANELS.map((p) => ({ id: `panel:${p}`, group: 'panel' as const, label: targetLabel(`panel:${p}`) })),
	...OPTIONS.map((o) => ({ id: `option:${o}`, group: 'option' as const, label: targetLabel(`option:${o}`) })),
	...MENU.map(([id, label]) => ({ id: `menu:${id}`, group: 'menu' as const, label }))
];

/** Targets whose Russian name or id contains `query` (any case, ё = е). */
export function search(query: string, list: Target[] = TARGETS): Target[] {
	const norm = (s: string) => s.toLowerCase().replaceAll('ё', 'е').trim();
	const q = norm(query);
	if (!q) return list;
	return list.filter((t) => norm(t.label).includes(q) || norm(t.id).includes(q));
}

/** Line icons (24 px grid) for the tools on the interface map. */
export const TOOL_ICONS: Record<string, string> = {
	Brush: 'M14.5 3.5 20.5 9.5l-7.5 7.5-6-6zM7 11l-1.5 1.5c-1.5 1.5-1 3-2 4.5-.5.8-1 1.5-1 2.5 1 0 1.7-.5 2.5-1 1.5-1 3-.5 4.5-2L11 15',
	Pencil: 'M4 20l1-4.5L16 4.5l3.5 3.5L8.5 19zM14 7l3.5 3.5',
	Eraser: 'M7 20 3 16l10-10 6 6-8 8zM21 20H8',
	RectMarquee: 'M4 6h2M9 6h2M14 6h2M19 6h1v2M20 11v2M20 16v2h-2M15 18h-2M10 18H8M5 18H4v-2M4 13v-2M4 8V6',
	EllipseMarquee: 'M12 6c5 0 8 2.7 8 6s-3 6-8 6-8-2.7-8-6 3-6 8-6z',
	Lasso: 'M7 21a4 4 0 0 1-2-3.5M3.5 14A7.5 6 0 1 1 12 18',
	MagicWand: 'M15 4l5 5M3 21l12-12M9 3v2M3 9h2M16 15v2M13 18h2',
	Move: 'M5 9l-3 3 3 3M9 5l3-3 3 3M15 19l-3 3-3-3M19 9l3 3-3 3M2 12h20M12 2v20',
	PaintBucket: 'M19 11 11 3 2.5 11.5a2 2 0 0 0 0 2.8l5.2 5.2c.8.8 2 .8 2.8 0zM5 2l5 5M2 13h15M22 20a2 2 0 1 1-4 0c0-1.6 1.7-2.4 2-4 .3 1.6 2 2.4 2 4z',
	Gradient: 'M4 4h16v16H4zM4 20 20 4',
	Eyedropper: 'M2 22l1-1h3l9-9M3 21v-3l9-9M15 6l3.4-3.4a2.1 2.1 0 1 1 3 3L18 9l.4.4a2.1 2.1 0 1 1-3 3l-3.8-3.8a2.1 2.1 0 1 1 3-3z',
	Smudge: 'M9 21V11a2 2 0 0 1 4 0v4M13 13a2 2 0 0 1 4 0v3M17 15a2 2 0 0 1 4 0v2a5 5 0 0 1-5 5h-3M9 15H7a2 2 0 0 0-2 2',
	Blur: 'M12 3s6 6.5 6 11a6 6 0 0 1-12 0c0-4.5 6-11 6-11z',
	Dodge: 'M12 3a4 4 0 0 1 0 8M12 11v10M8 21h8'
};
