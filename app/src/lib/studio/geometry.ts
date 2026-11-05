// Shapes on the studio canvas. Everything is in fractions of the task picture (0–1), like the
// lesson format; pixels appear only when drawing on screen or talking to PhotoCraft.
import type { Point, Region, Shape, ShapeStyle } from '../api.ts';

export const GHOST_COLOR = '#BC3823';
export const GHOST_OPACITY = 0.35;

export const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
export const round3 = (v: number) => Math.round(v * 1000) / 1000;
const pt = ([x, y]: Point): Point => [round3(clamp01(x)), round3(clamp01(y))];

/** The default stroke width PhotoCraft uses for a ghost on a `w`×`h` picture. */
export function defaultWidth(w: number, h: number): number {
	return Math.max(2, Math.round(Math.min(w, h) * 0.006));
}

/** A shape's look with the defaults filled in. */
export function styleOf(shape: Shape, w: number, h: number): Required<ShapeStyle> {
	const s = shape.style ?? {};
	return { color: s.color ?? GHOST_COLOR, width: s.width ?? defaultWidth(w, h), opacity: s.opacity ?? GHOST_OPACITY, dashed: s.dashed ?? false };
}

/** Drops the style keys that equal the defaults, so lesson files stay short. */
export function cleanStyle(style: ShapeStyle, w: number, h: number): ShapeStyle | undefined {
	const out: ShapeStyle = {};
	if (style.color && style.color.toUpperCase() !== GHOST_COLOR) out.color = style.color.toUpperCase();
	if (style.width !== undefined && style.width !== defaultWidth(w, h)) out.width = style.width;
	if (style.opacity !== undefined && Math.abs(style.opacity - GHOST_OPACITY) > 1e-9) out.opacity = style.opacity;
	if (style.dashed) out.dashed = true;
	return Object.keys(out).length ? out : undefined;
}

/** Ramer–Douglas–Peucker: fewer points along the same line, keeping both ends. */
export function simplify(points: Point[], tolerance = 0.002): Point[] {
	if (points.length <= 2) return points.slice();
	const [a, b] = [points[0], points[points.length - 1]];
	let worst = 0;
	let index = 0;
	for (let i = 1; i < points.length - 1; i++) {
		const d = distToSegment(points[i], a, b);
		if (d > worst) {
			worst = d;
			index = i;
		}
	}
	if (worst <= tolerance) return [a, b];
	const left = simplify(points.slice(0, index + 1), tolerance);
	const right = simplify(points.slice(index), tolerance);
	return [...left.slice(0, -1), ...right];
}

export function distToSegment(p: Point, a: Point, b: Point): number {
	const [dx, dy] = [b[0] - a[0], b[1] - a[1]];
	const len2 = dx * dx + dy * dy;
	const t = len2 === 0 ? 0 : Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2));
	return Math.hypot(p[0] - (a[0] + t * dx), p[1] - (a[1] + t * dy));
}

/** A rectangle from two corners, at least `min` on each side and inside the picture. */
export function regionFromDrag(a: Point, b: Point, min = 0.01): Region {
	const x = clamp01(Math.min(a[0], b[0]));
	const y = clamp01(Math.min(a[1], b[1]));
	const w = Math.max(min, Math.abs(a[0] - b[0]));
	const h = Math.max(min, Math.abs(a[1] - b[1]));
	return [round3(Math.min(x, 1 - w)), round3(Math.min(y, 1 - h)), round3(Math.min(w, 1)), round3(Math.min(h, 1))];
}

export type DrawTool = 'path' | 'line' | 'arrow' | 'ellipse' | 'rect';

/** The shape a drag with `tool` makes; `trail` is every pointer position for the free line. */
export function shapeFromDrag(tool: DrawTool, a: Point, b: Point, trail: Point[] = []): Shape | null {
	if (tool === 'path') {
		const points = simplify(trail.map(pt));
		return points.length >= 2 ? { kind: 'path', points } : null;
	}
	if (Math.hypot(a[0] - b[0], a[1] - b[1]) < 0.005) return null;
	if (tool === 'line' || tool === 'arrow') return { kind: tool, from: pt(a), to: pt(b) };
	return { kind: tool, rect: regionFromDrag(a, b) };
}

const ELLIPSE_SEGMENTS = 48;

/** The shape as a polyline (an arrow without its head). */
export function outline(shape: Shape): Point[] {
	switch (shape.kind) {
		case 'line':
		case 'arrow':
			return [shape.from, shape.to];
		case 'path':
			return shape.points;
		case 'rect': {
			const [x, y, w, h] = shape.rect;
			return [[x, y], [x + w, y], [x + w, y + h], [x, y + h], [x, y]];
		}
		case 'ellipse': {
			const [x, y, w, h] = shape.rect;
			const [cx, cy, rx, ry] = [x + w / 2, y + h / 2, w / 2, h / 2];
			return Array.from({ length: ELLIPSE_SEGMENTS + 1 }, (_, i) => {
				const t = (Math.PI * 2 * i) / ELLIPSE_SEGMENTS;
				return [cx + rx * Math.cos(t), cy + ry * Math.sin(t)] as Point;
			});
		}
	}
}

/** The two short strokes of an arrow head at the tip, in pixels (as PhotoCraft draws it). */
export function arrowHead(from: Point, to: Point, size: number): [Point, Point][] {
	const [dx, dy] = [to[0] - from[0], to[1] - from[1]];
	const len = Math.hypot(dx, dy);
	if (len < 1e-6) return [];
	const head = Math.min(Math.max(size * 4, len * 0.12), len * 0.5);
	const [ux, uy] = [dx / len, dy / len];
	return [1, -1].map((sign) => {
		const [c, s] = [Math.cos(0.5), sign * Math.sin(0.5)];
		const [rx, ry] = [ux * c - uy * s, ux * s + uy * c];
		return [to, [to[0] - rx * head, to[1] - ry * head]] as [Point, Point];
	});
}

/** SVG path data of the shape on a `w`×`h` picture, arrow head included. */
export function svgPath(shape: Shape, w: number, h: number): string {
	const px = ([x, y]: Point): Point => [x * w, y * h];
	const line = (pts: Point[]) => pts.map(([x, y], i) => `${i ? 'L' : 'M'}${x.toFixed(1)} ${y.toFixed(1)}`).join(' ');
	let d = line(outline(shape).map(px));
	if (shape.kind === 'arrow') {
		const size = styleOf(shape, w, h).width;
		for (const seg of arrowHead(px(shape.from), px(shape.to), size)) d += ' ' + line(seg);
	}
	return d;
}

/** Bounding box of a shape. */
export function bounds(shape: Shape): Region {
	const pts = outline(shape);
	const xs = pts.map((p) => p[0]);
	const ys = pts.map((p) => p[1]);
	const [x0, y0] = [Math.min(...xs), Math.min(...ys)];
	return [round3(x0), round3(y0), round3(Math.max(...xs) - x0), round3(Math.max(...ys) - y0)];
}

/** Index of the shape nearest to `p` within `tolerance`, the topmost on ties; -1 for none. */
export function hit(shapes: Shape[], p: Point, tolerance = 0.015): number {
	let best = -1;
	let bestD = tolerance;
	shapes.forEach((shape, i) => {
		const pts = outline(shape);
		for (let k = 1; k < pts.length; k++) {
			const d = distToSegment(p, pts[k - 1], pts[k]);
			if (d <= bestD) {
				bestD = d;
				best = i;
			}
		}
	});
	return best;
}

/** The shape moved by `[dx, dy]`, kept inside the picture. */
export function moveShape(shape: Shape, dx: number, dy: number): Shape {
	const b = bounds(shape);
	const mx = Math.min(Math.max(dx, -b[0]), 1 - (b[0] + b[2]));
	const my = Math.min(Math.max(dy, -b[1]), 1 - (b[1] + b[3]));
	const mv = ([x, y]: Point): Point => [round3(x + mx), round3(y + my)];
	switch (shape.kind) {
		case 'line':
		case 'arrow':
			return { ...shape, from: mv(shape.from), to: mv(shape.to) };
		case 'path':
			return { ...shape, points: shape.points.map(mv) };
		default: {
			const [x, y, w, h] = shape.rect;
			return { ...shape, rect: [round3(x + mx), round3(y + my), w, h] };
		}
	}
}

/** A region moved by `[dx, dy]`, kept inside the picture. */
export function moveRegion([x, y, w, h]: Region, dx: number, dy: number): Region {
	return [round3(Math.min(Math.max(x + dx, 0), 1 - w)), round3(Math.min(Math.max(y + dy, 0), 1 - h)), w, h];
}

/** A guide stroke (pixels of a `w`×`h` picture) as a ghost line for a task step. */
export function strokeToShape(points: number[][], w: number, h: number, width?: number): Shape | null {
	const pts = simplify(points.filter((p) => p.length >= 2).map(([x, y]) => pt([x / w, y / h])), 0.0015);
	if (pts.length < 2) return null;
	return width ? { kind: 'path', points: pts, style: { width: Math.round(width) } } : { kind: 'path', points: pts };
}

export const SHAPE_LABELS: Record<Shape['kind'], string> = {
	path: 'линия от руки',
	line: 'прямая',
	arrow: 'стрелка',
	ellipse: 'овал',
	rect: 'прямоугольник'
};
