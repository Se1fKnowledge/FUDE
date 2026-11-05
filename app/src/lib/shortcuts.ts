// Keyboard shortcuts of the practice screen. They must not clash with the editor's own: H is
// PhotoCraft's Hand tool, so the hint is F1 (free in PhotoCraft, and «help» everywhere else).

export const HINT_KEY_LABEL = 'F1';
export const CHECK_KEY_LABEL = 'Ctrl+Enter';

type Key = Pick<KeyboardEvent, 'key' | 'ctrlKey' | 'metaKey' | 'altKey' | 'shiftKey' | 'repeat'>;

/** F1 without modifiers asks for a hint. */
export function isHintKey(e: Key): boolean {
	return e.key === 'F1' && !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey && !e.repeat;
}

/** Ctrl+Enter (Cmd+Enter on a Mac) checks the task. */
export function isCheckKey(e: Key): boolean {
	return e.key === 'Enter' && (e.ctrlKey || e.metaKey) && !e.altKey && !e.repeat;
}

/** Shortcuts never fire while the student types into a field. */
export function typingInto(target: EventTarget | null): boolean {
	const el = target as { closest?: (s: string) => unknown } | null;
	return !!el?.closest?.('input, textarea, select, [contenteditable="true"]');
}
