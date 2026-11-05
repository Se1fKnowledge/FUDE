import { describe, expect, it, vi } from 'vitest';

const invoke = vi.fn((_cmd: string, _args?: unknown) => Promise.resolve('ok'));
vi.mock('@tauri-apps/api/core', () => ({ invoke: (cmd: string, args?: unknown) => invoke(cmd, args) }));

const { api, errorText } = await import('./api.ts');

describe('error text', () => {
	it('shows the Rust error string, an Error message or a fallback', () => {
		expect(errorText('редактор не найден')).toBe('редактор не найден');
		expect(errorText(new Error('сломалось'))).toBe('сломалось');
		expect(errorText({ code: 1 })).toBe('Что-то пошло не так');
		expect(errorText(undefined)).toBe('Что-то пошло не так');
	});
});

describe('commands', () => {
	it('call the Rust commands by their snake_case names', async () => {
		await api.practiceHintKey(true);
		expect(invoke).toHaveBeenLastCalledWith('practice_hint_key', { enabled: true });
		await api.editorArea(null);
		expect(invoke).toHaveBeenLastCalledWith('editor_area', { rect: null });
		await api.editorLayout(false);
		expect(invoke).toHaveBeenLastCalledWith('editor_layout', { embedded: false });
	});
});
