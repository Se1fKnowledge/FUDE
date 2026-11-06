// Ink art from real sumi brush strokes (docs/design-system.md, «Тушь и формы»). The pictures are
// luminance masks — white is ink — made by design/ink-assets.py; the components fill them with a
// theme colour, so the same stroke works on washi and in the dark theme. A seed picks the
// picture, so a menu item keeps the same stroke between renders.

import bar1 from '#lib/assets/ink/bar-1.webp';
import bar2 from '#lib/assets/ink/bar-2.webp';
import bar3 from '#lib/assets/ink/bar-3.webp';
import dab1 from '#lib/assets/ink/dab-1.webp';
import dab2 from '#lib/assets/ink/dab-2.webp';
import dab3 from '#lib/assets/ink/dab-3.webp';
import dab4 from '#lib/assets/ink/dab-4.webp';
import dab5 from '#lib/assets/ink/dab-5.webp';
import tap1 from '#lib/assets/ink/tap-1.webp';
import tap2 from '#lib/assets/ink/tap-2.webp';
import tap3 from '#lib/assets/ink/tap-3.webp';
import tap4 from '#lib/assets/ink/tap-4.webp';
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

/** Short dabs, one per step of the difficulty scale. These and the two below carry the ink as
 * alpha, for CSS `mask-image`. */
export const DABS = [dab1, dab2, dab3, dab4, dab5];

/** Round taps of ink: the days of the activity year. */
export const TAPS = [tap1, tap2, tap3, tap4];

/** Strokes stood upright, loaded end on top and a dry foot: the week bars. */
export const BARS = [bar1, bar2, bar3];

/** A CSS `mask-image` value for a picture. */
export const maskUrl = (src: string) => `url("${src}")`;

/** The picture a seed stands for: the same seed always gives the same one. */
export function pick<T>(list: readonly T[], seed: number): T {
	const i = Math.abs(Math.trunc(seed)) % list.length;
	return list[i];
}

/** A seed from text (a date, a name), for pictures that should stay put between renders. */
export function seedOf(text: string): number {
	let h = 0;
	for (let i = 0; i < text.length; i++) h = (Math.imul(h, 31) + text.charCodeAt(i)) | 0;
	return Math.abs(h);
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
