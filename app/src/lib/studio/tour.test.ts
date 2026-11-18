import { describe, expect, it } from 'vitest';
import { CHAPTERS, STEPS, columnPlace, words, type TourSnap } from './tour.ts';

const base: TourSnap = {
	mode: 'task',
	section: 'theory',
	tab: 'hint',
	tool: 'select',
	title: 'Мой первый урок',
	summary: '',
	difficulty: 1,
	theoryWords: 6,
	stepText: 'Нарисуйте что-нибудь кистью.',
	hintText: '',
	shapes: 0,
	checks: 2,
	saves: 0
};
const step = (id: string) => STEPS.find((s) => s.id === id)!;
const done = (id: string, now: Partial<TourSnap>, start: Partial<TourSnap> = {}) => step(id).done({ ...base, ...now }, { ...base, ...start });

describe('lesson-editor tour', () => {
	it('has unique steps in chapter order, each with a place to light', () => {
		expect(new Set(STEPS.map((s) => s.id)).size).toBe(STEPS.length);
		expect(STEPS.map((s) => s.chapter)).toEqual([...STEPS.map((s) => s.chapter)].sort((a, b) => a - b));
		expect(new Set(STEPS.map((s) => s.chapter)).size).toBe(CHAPTERS.length);
		for (const s of STEPS) {
			expect(s.target).toMatch(/^\[data-tour="[a-z-]+"\]$/);
			expect(s.title && s.body && s.task).toBeTruthy();
		}
	});

	it('no step is done before the author does anything', () => {
		// A fresh draft opens in «Задание» on the hint tab: the steps that only open a place
		// that is already open are the exception.
		const already = ['mode-task', 'tab-hint'];
		for (const s of STEPS) expect(s.done(base, base), s.id).toBe(already.includes(s.id));
	});

	it('a field step needs a real change, not the text it started with', () => {
		expect(done('title', { title: 'Мой первый урок' })).toBe(false);
		expect(done('title', { title: 'Пятно тушью' })).toBe(true);
		expect(done('title', { title: 'Я' })).toBe(false);
		expect(done('summary', { summary: 'коротко' })).toBe(false);
		expect(done('summary', { summary: 'Учимся ставить пятно одним движением' })).toBe(true);
		expect(done('difficulty', { difficulty: 2 })).toBe(true);
		expect(done('theory', { theoryWords: 15 })).toBe(false);
		expect(done('theory', { theoryWords: 16 })).toBe(true);
		expect(done('step-text', { stepText: 'Закрасьте пятно в центре листа' })).toBe(true);
		expect(done('hint-text', { hintText: 'Мягкой кистью' })).toBe(true);
	});

	it('drawing steps count new shapes and checks, saving counts a new save', () => {
		expect(done('ghost', { shapes: 1 })).toBe(true);
		expect(done('ghost', { shapes: 2 }, { shapes: 2 })).toBe(false);
		expect(done('region', { checks: 3 })).toBe(true);
		expect(done('region', { checks: 2 })).toBe(false);
		expect(done('save', { saves: 1 })).toBe(true);
		expect(done('save', { saves: 1 }, { saves: 1 })).toBe(false);
		expect(done('preview', { saves: 5 })).toBe(false);
	});

	it('opening steps follow the studio mode, section and tab', () => {
		expect(done('mode-lesson', { mode: 'lesson' })).toBe(true);
		expect(done('about', { mode: 'lesson', section: 'about' })).toBe(true);
		expect(done('about', { mode: 'task', section: 'about' })).toBe(false);
		expect(done('tab-text', { tab: 'text' })).toBe(true);
		expect(done('tool-ellipse', { tool: 'ellipse' })).toBe(true);
		expect(done('tool-region', { tool: 'region' })).toBe(true);
	});

	it('counts words', () => {
		expect(words('  одна   две\nтри ')).toBe(3);
		expect(words('')).toBe(0);
	});

	it('puts the card beside the lit control and never over it', () => {
		const view = { w: 1440, h: 900 };
		const size = { w: 340, h: 300 };
		const overlap = (p: { x: number; y: number }, h: { x: number; y: number; w: number; h: number }) => p.x < h.x + h.w && p.x + size.w > h.x && p.y < h.y + h.h && p.y + size.h > h.y;
		const tab = { x: 600, y: 10, w: 80, h: 36 };
		const leftItem = { x: 10, y: 120, w: 260, h: 44 };
		const field = { x: 320, y: 140, w: 700, h: 60 };
		const rightTab = { x: 1150, y: 110, w: 90, h: 34 };
		for (const hole of [tab, leftItem, field, rightTab]) {
			const p = columnPlace(view, hole, size);
			expect(overlap(p, hole), JSON.stringify(hole)).toBe(false);
			expect(p.x >= 0 && p.y >= 0 && p.x + size.w <= view.w && p.y + size.h <= view.h).toBe(true);
		}
		expect(columnPlace(view, leftItem, size).x).toBe(leftItem.x + leftItem.w + 22);
		// A control that fills the window leaves no room: the card goes into the bottom right corner.
		const canvas = { x: 300, y: 80, w: 1000, h: 780 };
		expect(columnPlace(view, canvas, size)).toEqual({ x: view.w - size.w - 12, y: view.h - size.h - 12 });
		const centred = columnPlace(view, null, size);
		expect(centred).toEqual({ x: 550, y: 300 });
	});
});
