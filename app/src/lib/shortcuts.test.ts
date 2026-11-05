import { describe, expect, it } from 'vitest';
import { HINT_KEY_LABEL, isCheckKey, isHintKey, typingInto } from './shortcuts';

const key = (k: string, mods: Partial<Record<'ctrlKey' | 'metaKey' | 'altKey' | 'shiftKey' | 'repeat', boolean>> = {}) => ({
	key: k,
	ctrlKey: false,
	metaKey: false,
	altKey: false,
	shiftKey: false,
	repeat: false,
	...mods
});

describe('hint shortcut', () => {
	it('is F1 and nothing that PhotoCraft uses', () => {
		expect(HINT_KEY_LABEL).toBe('F1');
		expect(isHintKey(key('F1'))).toBe(true);
		// H and its Russian-layout twin are the editor's Hand tool now.
		for (const k of ['h', 'H', 'р', 'Р']) expect(isHintKey(key(k))).toBe(false);
	});

	it('ignores F1 with modifiers and held-down repeats', () => {
		expect(isHintKey(key('F1', { ctrlKey: true }))).toBe(false);
		expect(isHintKey(key('F1', { shiftKey: true, ctrlKey: true }))).toBe(false);
		expect(isHintKey(key('F1', { altKey: true }))).toBe(false);
		expect(isHintKey(key('F1', { repeat: true }))).toBe(false);
	});
});

describe('check shortcut', () => {
	it('is Ctrl+Enter or Cmd+Enter', () => {
		expect(isCheckKey(key('Enter', { ctrlKey: true }))).toBe(true);
		expect(isCheckKey(key('Enter', { metaKey: true }))).toBe(true);
		expect(isCheckKey(key('Enter'))).toBe(false);
		expect(isCheckKey(key('Enter', { ctrlKey: true, altKey: true }))).toBe(false);
		expect(isCheckKey(key('Enter', { ctrlKey: true, repeat: true }))).toBe(false);
	});
});

describe('typing guard', () => {
	const inside = (match: boolean) => ({ closest: () => (match ? {} : null) });
	it('blocks shortcuts in fields only', () => {
		expect(typingInto(inside(true) as unknown as EventTarget)).toBe(true);
		expect(typingInto(inside(false) as unknown as EventTarget)).toBe(false);
		expect(typingInto(null)).toBe(false);
		expect(typingInto({} as EventTarget)).toBe(false);
	});
});
