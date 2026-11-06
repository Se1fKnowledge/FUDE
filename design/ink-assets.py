"""Ink art for the app from real sumi brush strokes: the menu and card strokes, the ensō circles
and the mountain washes. Writes luminance masks (white = ink) to app/src/lib/assets/ink/; the
components fill them with a theme colour, so one picture serves both themes.

Source: «Sumi Strokes» Photoshop brushes by Denny Tang (SparkleStock, free download, SparkleStock
Standard License). The brush file itself stays out of the repository; only a few strokes,
downscaled and reworked for the interface, are shipped. See ATTRIBUTION in app/src/lib/assets/ink/.

    python design/ink-assets.py "<path>/Sumi Strokes (1250 pixels).abr"

Needs Pillow and NumPy."""
import math
import os
import random
import struct
import sys

import numpy as np
from PIL import Image, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'app', 'src', 'lib', 'assets', 'ink')

# Horizontal strokes with a heavy start on the left and a dry tail: (tip number, mirror).
STROKES = [(21, False), (22, False), (34, False), (47, False), (61, False), (77, False), (26, True), (42, True)]
# Strokes bent into an ensō: (tip number, mirror).
ENSO = [(21, False), (26, False), (61, False), (77, False)]


# --- ABR reading (Photoshop 6+ sampled brushes) -------------------------------------------------

def packbits(data: bytes, n: int) -> bytes:
    out = bytearray()
    i = 0
    while len(out) < n and i < len(data):
        c = data[i]
        i += 1
        if c > 127:
            out += bytes([data[i]]) * (257 - c)
            i += 1
        else:
            out += data[i:i + c + 1]
            i += c + 1
    return bytes(out[:n])


def read_abr(path: str) -> list[np.ndarray]:
    """Every sampled tip as a float array, 1 = full ink."""
    f = open(path, 'rb').read()
    _version, sub = struct.unpack('>HH', f[:4])
    tips = []
    pos = 4
    while pos + 12 <= len(f):
        sig, key, size = struct.unpack('>4s4sI', f[pos:pos + 12])
        pos += 12
        if sig != b'8BIM':
            break
        end = pos + size
        if key == b'samp':
            p = pos
            while p < end:
                (bsize,) = struct.unpack('>I', f[p:p + 4])
                p += 4
                bend = p + bsize + (-bsize % 4)
                q = p + (47 if sub == 1 else 301)
                top, left, bottom, right = struct.unpack('>iiii', f[q:q + 16])
                depth, comp = struct.unpack('>HB', f[q + 16:q + 19])
                q += 19
                w, h, bpp = right - left, bottom - top, depth // 8
                if comp == 0:
                    raw = f[q:q + w * h * bpp]
                else:
                    rows = struct.unpack(f'>{h}H', f[q:q + 2 * h])
                    q += 2 * h
                    buf = bytearray()
                    for r in rows:
                        buf += packbits(f[q:q + r], w * bpp)
                        q += r
                    raw = bytes(buf)
                a = np.frombuffer(raw.ljust(w * h * bpp, b'\0'), dtype=np.uint8 if bpp == 1 else '>u2')
                tips.append(a.reshape(h, w).astype(np.float32) / (255 if bpp == 1 else 65535))
                p = bend
        pos = end
    return tips


# --- helpers ------------------------------------------------------------------------------------

def crop(a: np.ndarray, flip: bool = False) -> np.ndarray:
    # Rows and columns that carry real ink; stray specks of spatter don't widen the box.
    rows, cols = a.sum(axis=1), a.sum(axis=0)
    ys = np.where(rows > rows.max() * 0.01)[0]
    xs = np.where(cols > cols.max() * 0.01)[0]
    a = a[ys.min():ys.max() + 1, xs.min():xs.max() + 1]
    return a[:, ::-1] if flip else a


def resize(a: np.ndarray, w: int, h: int) -> np.ndarray:
    im = Image.fromarray((np.clip(a, 0, 1) * 255).astype(np.uint8)).resize((w, h), Image.LANCZOS)
    return np.asarray(im, np.float32) / 255


def blur(a: np.ndarray, r: float) -> np.ndarray:
    im = Image.fromarray((np.clip(a, 0, 1) * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(r))
    return np.asarray(im, np.float32) / 255


def save(a: np.ndarray, name: str) -> None:
    path = os.path.join(OUT, name)
    Image.fromarray((np.clip(a, 0, 1) * 255).astype(np.uint8)).save(path, 'WEBP', quality=84, method=6)
    print(f'{name}: {a.shape[1]}x{a.shape[0]}, {os.path.getsize(path) // 1024} KB')


# --- the pieces ---------------------------------------------------------------------------------

def stroke(t: np.ndarray, width: int = 640) -> np.ndarray:
    h = round(t.shape[0] * width / t.shape[1])
    return resize(t, width, h)


def enso(t: np.ndarray, size: int = 768, sweep: float = 320, start: float = -70, thick: float = 0.11) -> np.ndarray:
    """The stroke bent clockwise around a circle, starting `start`° from 3 o'clock (the angle the
    Enso component's draw animation starts at), radius 0.40 of the size."""
    h, w = t.shape
    yy, xx = np.mgrid[0:size, 0:size].astype(np.float32)
    c = size / 2
    r = np.hypot(xx - c, yy - c)
    ang = (np.degrees(np.arctan2(yy - c, xx - c)) - start) % 360
    band = size * thick
    u = ang / sweep * (w - 1)
    v = (r - (size * 0.40 - band / 2)) / band * (h - 1)
    ok = (ang <= sweep) & (v >= 0) & (v <= h - 1)
    return np.where(ok, t[np.clip(v, 0, h - 1).astype(int), np.clip(u, 0, w - 1).astype(int)], 0)


def ribbon(tips, width: int, band: int, picks) -> np.ndarray:
    """Strokes laid end to end with a little overlap: one long brush line `band` px tall."""
    parts = [resize(crop(tips[n - 1], fl), round(crop(tips[n - 1]).shape[1] * band / crop(tips[n - 1]).shape[0]), band) for n, fl in picks]
    out = np.zeros((band, sum(p.shape[1] for p in parts)), np.float32)
    x = 0
    for p in parts:
        out[:, x:x + p.shape[1]] = np.maximum(out[:, x:x + p.shape[1]], p)
        x += int(p.shape[1] * 0.78)
    return resize(out[:, :x + 1], width, band)


def ridge(width: int, height: int, rnd: random.Random, peaks: int, rise: float, base: float) -> np.ndarray:
    xs = np.arange(width, dtype=np.float32)
    y = np.full(width, height * base, np.float32)
    for _ in range(peaks):
        c = rnd.uniform(-0.05, 1.05) * width
        wl = rnd.uniform(0.07, 0.15) * width
        wr = wl * rnd.uniform(0.7, 1.5)
        h = rnd.uniform(0.55, 1.0) * rise * height
        d = np.where(xs < c, (c - xs) / wl, (xs - c) / wr)
        y = np.minimum(y, height * base - h * np.clip(1 - d, 0, 1) ** 1.6)
    for k in (3, 7, 17, 41):
        y += np.sin(xs / width * math.pi * k * width / 1600 + rnd.uniform(0, 6)) * height * 0.006 * (8 / k) ** 0.5
    return y


def mountains(tips, width: int, height: int, seed: int, peaks: int, rise: float, base: float, band: int, picks, wash: float) -> np.ndarray:
    """One layer: a dry brush line along a ridge, and below it a wash that fades into mist,
    streaked along the ridge by a dry-brush texture."""
    rnd = random.Random(seed)
    y = ridge(width, height, rnd, peaks, rise, base)
    rib = ribbon(tips, width, band, picks)
    yy = np.arange(height, dtype=np.float32)[:, None]
    xi = np.broadcast_to(np.arange(width), (height, width))
    v = (yy - (y[None, :] - band * 0.35)) / band * (band - 1)
    ok = (v >= 0) & (v <= band - 1)
    line = np.where(ok, rib[np.clip(v, 0, band - 1).astype(int), xi], 0)
    press = 0.65 + 0.35 * np.sin(np.arange(width) / width * math.pi * rnd.uniform(3, 6) * width / 1600 + rnd.uniform(0, 6))
    tex = blur(resize(crop(tips[28]), width, height), 0.8)
    streak = tex[np.clip(yy - y[None, :] + band * 0.2, 0, height - 1).astype(int), xi]
    depth = (yy - y[None, :]) / (height * 0.7)
    body = np.where(depth > 0, wash * np.clip(1 - depth, 0, 1) ** 1.8 * (0.35 + 0.65 * streak), 0)
    return np.maximum(line * press[None, :] * 0.95, body)


def main(abr: str) -> None:
    tips = read_abr(abr)
    assert len(tips) == 82, f'expected the 82 Sumi Strokes, got {len(tips)}'
    os.makedirs(OUT, exist_ok=True)
    for i, (n, fl) in enumerate(STROKES, 1):
        save(stroke(crop(tips[n - 1], fl)), f'stroke-{i}.webp')
    for i, (n, fl) in enumerate(ENSO, 1):
        save(enso(crop(tips[n - 1], fl)), f'enso-{i}.webp')
    w, h = 2400, 260
    save(blur(mountains(tips, w, h, 3, 9, 0.62, 0.95, 40, [(22, False), (21, False), (61, False), (77, False), (34, False)], 0.8), 2.5), 'mist-far.webp')
    save(blur(mountains(tips, w, h, 4, 8, 0.42, 0.98, 46, [(61, False), (34, False), (21, False), (47, False), (22, False)], 0.85), 0.8), 'mist-mid.webp')
    save(mountains(tips, w, h, 5, 6, 0.26, 1.02, 54, [(47, False), (22, False), (34, False), (77, False)], 0.9), 'mist-near.webp')


if __name__ == '__main__':
    main(sys.argv[1])
