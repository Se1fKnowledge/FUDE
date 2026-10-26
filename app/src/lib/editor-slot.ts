// The editor window can sit inside the FUDE window. A screen marks the area for it with
// `use:editorSlot`; the Rust side then places PhotoCraft's window exactly over that element.
import { api } from './api.ts';

const KEY = 'fude-editor-embedded';

/** The student's choice: the editor inside FUDE (default) or in its own window. */
export function preferEmbedded(): boolean {
	try {
		return localStorage.getItem(KEY) !== 'false';
	} catch {
		return true;
	}
}

export function rememberEmbedded(embedded: boolean): void {
	try {
		localStorage.setItem(KEY, String(embedded));
	} catch {
		// Private mode or blocked storage: the choice lasts until the window closes.
	}
}

/** The element's box in physical pixels of the window. */
export function physicalRect(el: Element, ratio = window.devicePixelRatio || 1) {
	const r = el.getBoundingClientRect();
	return {
		x: Math.round(r.left * ratio),
		y: Math.round(r.top * ratio),
		width: Math.round(r.width * ratio),
		height: Math.round(r.height * ratio)
	};
}

/** Svelte action: keeps the editor window over `node` while it is mounted. */
export function editorSlot(node: HTMLElement) {
	let frame = 0;
	let last = '';
	const send = () => {
		cancelAnimationFrame(frame);
		frame = requestAnimationFrame(() => {
			const rect = physicalRect(node);
			const key = JSON.stringify(rect);
			if (key === last) return;
			last = key;
			api.editorArea(rect).catch(() => {});
		});
	};
	const observer = new ResizeObserver(send);
	observer.observe(node);
	window.addEventListener('resize', send);
	window.addEventListener('scroll', send, true);
	// Layout can shift without a resize (fonts, panels): re-check now and then, cheaply.
	const poll = setInterval(send, 700);
	send();
	return {
		destroy() {
			observer.disconnect();
			window.removeEventListener('resize', send);
			window.removeEventListener('scroll', send, true);
			clearInterval(poll);
			cancelAnimationFrame(frame);
			api.editorArea(null).catch(() => {});
		}
	};
}
