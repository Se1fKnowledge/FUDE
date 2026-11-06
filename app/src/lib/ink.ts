// Ink art from real sumi brush strokes (docs/design-system.md, «Тушь и формы»). The pictures are
// luminance masks — white is ink — made by design/ink-assets.py; the components fill them with a
// theme colour, so the same stroke works on washi and in the dark theme. A seed picks the
// picture, so a menu item keeps the same stroke between renders.

import enso1 from '#lib/assets/ink/enso-1.webp';
import enso2 from '#lib/assets/ink/enso-2.webp';
import enso3 from '#lib/assets/ink/enso-3.webp';
import enso4 from '#lib/assets/ink/enso-4.webp';
import mistFar from '#lib/assets/ink/mist-far.webp';
import mistMid from '#lib/assets/ink/mist-mid.webp';
import mistNear from '#lib/assets/ink/mist-near.webp';
import stroke1 from '#lib/assets/ink/stroke-1.webp';
import stroke2 from '#lib/assets/ink/stroke-2.webp';
import stroke3 from '#lib/assets/ink/stroke-3.webp';
import stroke4 from '#lib/assets/ink/stroke-4.webp';
import stroke5 from '#lib/assets/ink/stroke-5.webp';
import stroke6 from '#lib/assets/ink/stroke-6.webp';
import stroke7 from '#lib/assets/ink/stroke-7.webp';
import stroke8 from '#lib/assets/ink/stroke-8.webp';

/** Horizontal strokes: a loaded start on the left, a dry tail on the right. */
export const STROKES = [stroke1, stroke2, stroke3, stroke4, stroke5, stroke6, stroke7, stroke8];

/** One-stroke circles. Each starts 70° above 3 o'clock and runs clockwise, like the Enso draw
 * animation; the ring sits at 0.40 of the picture's size. */
export const ENSO = [enso1, enso2, enso3, enso4];
export const ENSO_RADIUS = 0.4;
export const ENSO_START = -70;

/** Mountain washes, far to near, each 2400×260. */
export const MIST = { far: mistFar, mid: mistMid, near: mistNear, width: 2400, height: 260 };

/** The picture a seed stands for: the same seed always gives the same one. */
export function pick<T>(list: readonly T[], seed: number): T {
	const i = Math.abs(Math.trunc(seed)) % list.length;
	return list[i];
}

/** Where a `width`×`height` window looks into a mountain picture scaled to the window's height:
 * the picture's drawn width and its left offset (≤ 0). Different seeds look at different hills;
 * a window wider than the picture stretches it instead. */
export function mistWindow(width: number, height: number, seed: number): { width: number; x: number } {
	const drawn = Math.max(width, (height * MIST.width) / MIST.height);
	const room = Math.floor(drawn - width);
	const x = room > 0 ? -(Math.abs(Math.trunc(seed) * 389) % (room + 1)) : 0;
	return { width: drawn, x };
}
