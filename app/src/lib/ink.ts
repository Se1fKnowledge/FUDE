// Ink shapes drawn in code (docs/design-system.md, «Тушь и формы»): a brush stroke with a
// dry tail, an ensō circle and misty mountain ridges. A seed makes each shape repeatable,
// so a menu item keeps the same stroke between renders.

type Pt = [number, number];

/** A small, fast, seeded random generator (mulberry32). */
export function rng(seed: number): () => number {
	let s = seed | 0;
	return () => {
		s = (s + 0x6d2b79f5) | 0;
		let t = Math.imul(s ^ (s >>> 15), 1 | s);
		t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

const f = (n: number) => Math.round(n * 10) / 10;

function polygon(top: Pt[], bottom: Pt[]): string {
	const pts = [...top, ...bottom.slice().reverse()];
	return `M${pts.map(([x, y]) => `${f(x)} ${f(y)}`).join('L')}Z`;
}

export interface Stroke {
	body: string;
	hairs: string[];
}

/** A horizontal brush stroke in a `w`×`h` box: a round loaded start, a full belly and a
 * tail that thins and splits into dry hairs. */
export function brushStroke(w: number, h: number, seed = 1): Stroke {
	const r = rng(seed);
	const n = 36;
	const mid = h / 2;
	const half = h * 0.4;
	const x0 = h * 0.35;
	const x1 = w * 0.84;
	const wave = r() * Math.PI;
	const top: Pt[] = [];
	const bottom: Pt[] = [];
	for (let i = 0; i <= n; i++) {
		const t = i / n;
		const x = x0 + t * (x1 - x0);
		const y = mid + Math.sin(t * Math.PI * 1.2 + wave) * h * 0.05;
		const shape = t < 0.1 ? 0.7 + (t / 0.1) * 0.3 : t > 0.68 ? Math.max(0.14, 1 - ((t - 0.68) / 0.32) * 0.86) : 1;
		const thick = half * shape * (0.9 + r() * 0.18);
		top.push([x, y - thick]);
		bottom.push([x, y + thick * 0.92]);
	}
	// A rounded, ink-loaded start.
	top.unshift([x0 - half * 0.55, mid - half * 0.45]);
	bottom.unshift([x0 - half * 0.55, mid + half * 0.4]);
	const hairs: string[] = [];
	const count = 4 + Math.floor(r() * 3);
	for (let i = 0; i < count; i++) {
		const start = x0 + (x1 - x0) * (0.62 + r() * 0.15);
		const end = Math.min(w - 1, x1 + (w - x1) * (0.4 + r() * 0.6));
		const dy = (r() - 0.5) * half * 1.3;
		hairs.push(`M${f(start)} ${f(mid + dy)}L${f(end)} ${f(mid + dy * 1.25 + (r() - 0.5) * 2)}`);
	}
	return { body: polygon(top, bottom), hairs };
}

/** An ensō: one circular stroke of radius `r` around (`cx`, `cy`) that swells and runs
 * dry, leaving a gap. */
export function enso(cx: number, cy: number, radius: number, width: number, seed = 7): string {
	const r = rng(seed);
	const start = -Math.PI / 2 + 0.35 + r() * 0.3;
	const sweep = Math.PI * 2 - 0.55 - r() * 0.25;
	const n = 96;
	const outer: Pt[] = [];
	const inner: Pt[] = [];
	for (let i = 0; i <= n; i++) {
		const t = i / n;
		const a = start + sweep * t;
		const swell = Math.sin(Math.PI * Math.min(1, t * 1.1)) ** 0.6;
		const w = width * (0.25 + 0.75 * swell) * (t > 0.85 ? 1 - (t - 0.85) * 4 : 1) * (0.94 + r() * 0.12);
		const rr = radius + Math.sin(t * 9 + r()) * width * 0.08;
		outer.push([cx + Math.cos(a) * (rr + w / 2), cy + Math.sin(a) * (rr + w / 2)]);
		inner.push([cx + Math.cos(a) * (rr - w / 2), cy + Math.sin(a) * (rr - w / 2)]);
	}
	return polygon(outer, inner);
}

/** A mountain ridge filling a `w`×`h` box from `base` (0–1 of the height) to the bottom. */
export function ridge(w: number, h: number, base: number, roughness: number, seed = 3): string {
	const r = rng(seed);
	const n = 28;
	const pts: Pt[] = [];
	let y = h * base;
	for (let i = 0; i <= n; i++) {
		const x = (w * i) / n;
		y += (r() - 0.5) * h * roughness;
		y = Math.min(h * 0.95, Math.max(h * (base - 0.25), y));
		pts.push([x, y]);
	}
	return `M0 ${h}L${pts.map(([x, py]) => `${f(x)} ${f(py)}`).join('L')}L${w} ${h}Z`;
}
