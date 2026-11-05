import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const editorArea = vi.fn((_rect: unknown) => Promise.resolve());
vi.mock('./api.ts', () => ({ api: { editorArea: (rect: unknown) => editorArea(rect) } }));

const { editorSlot, physicalRect, preferEmbedded, rememberEmbedded } = await import('./editor-slot.ts');

const box = (left: number, top: number, width: number, height: number) =>
	({ getBoundingClientRect: () => ({ left, top, width, height }) }) as unknown as HTMLElement;

describe('embedding choice', () => {
	afterEach(() => vi.unstubAllGlobals());

	it('defaults to the editor inside FUDE and remembers the student’s choice', () => {
		const store = new Map<string, string>();
		vi.stubGlobal('localStorage', { getItem: (k: string) => store.get(k) ?? null, setItem: (k: string, v: string) => store.set(k, v) });
		expect(preferEmbedded()).toBe(true);
		rememberEmbedded(false);
		expect(preferEmbedded()).toBe(false);
		rememberEmbedded(true);
		expect(preferEmbedded()).toBe(true);
	});

	it('survives blocked storage', () => {
		const broken = () => {
			throw new Error('blocked');
		};
		vi.stubGlobal('localStorage', { getItem: broken, setItem: broken });
		expect(() => rememberEmbedded(false)).not.toThrow();
		expect(preferEmbedded()).toBe(true);
	});
});

describe('physical rect', () => {
	it('scales CSS pixels by the display ratio and rounds', () => {
		expect(physicalRect(box(10, 20, 300, 200), 1)).toEqual({ x: 10, y: 20, width: 300, height: 200 });
		expect(physicalRect(box(10.4, 20.5, 300.3, 199.9), 1.5)).toEqual({ x: 16, y: 31, width: 450, height: 300 });
		expect(physicalRect(box(0, 0, 100, 50), 2)).toEqual({ x: 0, y: 0, width: 200, height: 100 });
	});
});

describe('editor slot action', () => {
	let frames: (() => void)[];
	let listeners: Map<string, unknown>;
	let observed: unknown[];
	let disconnected: number;

	beforeEach(() => {
		vi.useFakeTimers();
		editorArea.mockClear();
		frames = [];
		listeners = new Map();
		observed = [];
		disconnected = 0;
		vi.stubGlobal('requestAnimationFrame', (f: () => void) => frames.push(f));
		vi.stubGlobal('cancelAnimationFrame', () => {});
		vi.stubGlobal('window', {
			devicePixelRatio: 2,
			addEventListener: (name: string, f: unknown) => listeners.set(name, f),
			removeEventListener: (name: string) => listeners.delete(name)
		});
		vi.stubGlobal(
			'ResizeObserver',
			class {
				observe(n: unknown) {
					observed.push(n);
				}
				disconnect() {
					disconnected++;
				}
			}
		);
	});

	afterEach(() => {
		vi.useRealTimers();
		vi.unstubAllGlobals();
	});

	const flush = () => frames.splice(0).forEach((f) => f());

	it('reports the slot once, then only when it moves, and clears it on destroy', () => {
		let rect = { left: 5, top: 10, width: 100, height: 80 };
		const node = { getBoundingClientRect: () => rect } as unknown as HTMLElement;
		const action = editorSlot(node);
		expect(observed).toEqual([node]);
		expect([...listeners.keys()].sort()).toEqual(['resize', 'scroll']);

		flush();
		expect(editorArea).toHaveBeenLastCalledWith({ x: 10, y: 20, width: 200, height: 160 });

		vi.advanceTimersByTime(700);
		flush();
		expect(editorArea).toHaveBeenCalledTimes(1);

		rect = { ...rect, width: 150 };
		(listeners.get('resize') as () => void)();
		flush();
		expect(editorArea).toHaveBeenCalledTimes(2);
		expect(editorArea).toHaveBeenLastCalledWith({ x: 10, y: 20, width: 300, height: 160 });

		action.destroy();
		expect(disconnected).toBe(1);
		expect(listeners.size).toBe(0);
		expect(editorArea).toHaveBeenLastCalledWith(null);
		vi.advanceTimersByTime(5000);
		flush();
		expect(editorArea).toHaveBeenCalledTimes(3);
	});

	it('keeps going when the Rust side refuses', async () => {
		editorArea.mockImplementation(() => Promise.reject(new Error('no window')));
		const action = editorSlot(box(0, 0, 10, 10));
		flush();
		await Promise.resolve();
		expect(() => action.destroy()).not.toThrow();
		editorArea.mockImplementation(() => Promise.resolve());
	});
});
