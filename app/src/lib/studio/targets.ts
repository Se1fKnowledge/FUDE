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
