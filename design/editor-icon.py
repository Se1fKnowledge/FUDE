"""The FUDE editor icon for PhotoCraft: 筆 in the Yuji Syuku brush face, washi white on an
indigo tile, with a small vermilion seal. Writes every icon file PhotoCraft ships.

Run from anywhere: python design/editor-icon.py (needs Pillow and fontTools, and app/node_modules
installed for the font)."""
import os

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont
from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# The Yuji Syuku subset that holds 筆 (npm package @fontsource/yuji-syuku, SIL OFL 1.1).
FONT = os.path.join(ROOT, 'app', 'node_modules', '@fontsource', 'yuji-syuku', 'files', 'yuji-syuku-101-400-normal.woff')
OUT = os.path.join(ROOT, 'photocraft', 'assets', 'app-icon')
INDIGO = (31, 68, 112)
INDIGO_DEEP = (22, 48, 79)
WASHI = (243, 238, 228)
SHU = (188, 56, 35)
CHAR = '筆'


def tile(size: int) -> Image.Image:
    """The icon at `size` px, drawn 4× larger and scaled down for clean edges."""
    s = size * 4
    img = Image.new('RGBA', (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    radius = round(s * 112 / 512)
    # The field: indigo, a little deeper towards the bottom.
    field = Image.new('RGBA', (s, s))
    fd = ImageDraw.Draw(field)
    for y in range(s):
        k = y / (s - 1)
        fd.line([(0, y), (s, y)], fill=tuple(round(a + (b - a) * k * 0.8) for a, b in zip(INDIGO, INDIGO_DEEP)) + (255,))
    mask = Image.new('L', (s, s), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, s - 1, s - 1], radius=radius, fill=255)
    img.paste(field, (0, 0), mask)
    # The brush character.
    font = ImageFont.truetype(FONT, round(s * 0.74))
    box = d.textbbox((0, 0), CHAR, font=font)
    w, h = box[2] - box[0], box[3] - box[1]
    x = (s - w) / 2 - box[0] - s * 0.03
    y = (s - h) / 2 - box[1] - s * 0.02
    d.text((x, y), CHAR, font=font, fill=WASHI + (255,))
    # The seal, bottom right.
    if size >= 32:
        q = s * 0.15
        sx, sy = s * 0.78, s * 0.76
        d.rounded_rectangle([sx, sy, sx + q, sy + q], radius=q * 0.18, fill=SHU + (255,))
        inner = q * 0.16
        d.rounded_rectangle([sx + inner, sy + inner, sx + q - inner, sy + q - inner], radius=q * 0.1, outline=WASHI + (255,), width=max(1, round(q * 0.07)))
    return img.resize((size, size), Image.LANCZOS)


def glyph_path(view: float) -> tuple[str, float, float]:
    """筆 as an SVG path in a `view`-unit tile, with its position."""
    font = TTFont(FONT)
    gs = font.getGlyphSet()
    name = font.getBestCmap()[ord(CHAR)]
    upm = font['head'].unitsPerEm
    scale = view * 0.74 / upm
    pen = SVGPathPen(gs)
    # Font units grow up; SVG grows down. Place the em box like the raster version.
    ascent = font['hhea'].ascent
    left = view * 0.13 - view * 0.03
    top = view * 0.13 - view * 0.02
    tp = TransformPen(pen, (scale, 0, 0, -scale, left, top + ascent * scale))
    gs[name].draw(tp)
    return pen.getCommands(), left, top


def svg(view: int = 512) -> str:
    d, _, _ = glyph_path(view)
    q = view * 0.15
    sx, sy = view * 0.78, view * 0.76
    inner = q * 0.16
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{view}" height="{view}" viewBox="0 0 {view} {view}">
  <!-- FUDE editor icon: 筆 (Yuji Syuku, SIL OFL) on indigo with a vermilion seal. -->
  <defs>
    <clipPath id="tile"><rect width="{view}" height="{view}" rx="112"/></clipPath>
    <linearGradient id="field" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#1F4470"/>
      <stop offset="1" stop-color="#1B3A60"/>
    </linearGradient>
  </defs>
  <g clip-path="url(#tile)">
    <rect width="{view}" height="{view}" fill="url(#field)"/>
    <path d="{d}" fill="#F3EEE4"/>
    <rect x="{sx:.1f}" y="{sy:.1f}" width="{q:.1f}" height="{q:.1f}" rx="{q * 0.18:.1f}" fill="#BC3823"/>
    <rect x="{sx + inner:.1f}" y="{sy + inner:.1f}" width="{q - 2 * inner:.1f}" height="{q - 2 * inner:.1f}" rx="{q * 0.1:.1f}" fill="none" stroke="#F3EEE4" stroke-width="{q * 0.07:.1f}"/>
  </g>
</svg>
'''


def main():
    for size in [16, 24, 32, 48, 64, 128, 256, 512]:
        path = os.path.join(OUT, 'hicolor', f'{size}x{size}', 'apps', 'ai.storyteller.photocraft.png')
        tile(size).save(path, optimize=True)
    # macOS: the tile on Apple's 824/1024 grid.
    mac = Image.new('RGBA', (1024, 1024), (0, 0, 0, 0))
    mac.paste(tile(824), (100, 100))
    mac.save(os.path.join(OUT, 'photocraft-1024.png'), optimize=True)
    big = tile(256)
    big.save(os.path.join(OUT, 'photocraft.ico'), sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    mac.save(os.path.join(OUT, 'photocraft.icns'))
    text = svg()
    for name in ['photocraft.svg', 'photocraft-small.svg', os.path.join('hicolor', 'scalable', 'apps', 'ai.storyteller.photocraft.svg')]:
        with open(os.path.join(OUT, name), 'w', encoding='utf-8', newline='\n') as f:
            f.write(text)
    print('ok')


main()
