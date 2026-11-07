# The FUDE design system

[Русский](../design-system.md) · **English**

The source of truth is the Figma mockup: [FUDE 筆 — школа цифрового рисунка](https://www.figma.com/design/dHDrUBG4xsJ8LL2saJHFxs) (page “01 · Дизайн-система”). Colours are set up there as variables of the **FUDE / Цвет** collection, text as styles, shadows as effect styles. This document repeats them as text and explains the rules. The mockup's screens are in Russian; the app itself is in Russian and English.

## The idea

Japanese ink painting (sumi-e). Warm washi paper instead of white, black ink for structure, brush strokes, ensō circles and red hanko seals as living accents. The interface around them stays quiet. The user chose the colours: bluish, red, orange, white, black.

## Colours

### Paper (light surfaces)

| Token | HEX | Where |
|---|---|---|
| `paper/base` | `#F3EEE4` | the app background |
| `paper/raised` | `#FBF8F2` | cards |
| `paper/sunken` | `#E9E2D4` | sunken fields, chips, progress tracks |
| `paper/line` | `#DDD4C3` | lines and borders |

### Ink (text and dark)

| Token | HEX | Where |
|---|---|---|
| `ink/900` | `#16161A` | main text, the sidebar |
| `ink/700` | `#3B3A40` | secondary text |
| `ink/500` | `#625E57` | captions, meta information (contrast ≥ 5:1 on all paper) |
| `ink/300` | `#ADA79C` | decorative elements only, not text |

### Indigo · ai-iro (progress, growth, hints)

| Token | HEX |
|---|---|
| `ai/900` | `#16304F` |
| `ai/700` | `#1F4470` |
| `ai/500` | `#2F5D8C` |
| `ai/300` | `#8FB2D6` |
| `ai/100` | `#DCE7F1` |

### Vermilion · shu-iro (the brand and the main action)

| Token | HEX | Where |
|---|---|---|
| `shu/700` | `#A92F1E` | text on a light red background |
| `shu/500` | `#BC3823` | the main button, the active menu item, seals |
| `shu/100` | `#F6D9D2` | a light backing |

### Persimmon and gold (streaks, experience, rewards)

| Token | HEX | Where |
|---|---|---|
| `kaki/700` | `#8F4E16` | text on `kaki/100` |
| `kaki/500` | `#E8893B` | day streaks, experience, warnings |
| `kaki/100` | `#F7E1CB` | backing |
| `kin/700` | `#76591F` | text on `kin/100` |
| `kin/500` | `#C29A4E` | rare rewards, “a day without hints” |
| `kin/100` | `#F1E6CC` | backing |

### The editor's night

| Token | HEX | Where |
|---|---|---|
| `sumi/bg` | `#111216` | the editor canvas |
| `sumi/panel` | `#1A1C21` | panels |
| `sumi/raised` | `#23262D` | hover, raised cards |
| `sumi/line` | `#2F333B` | lines |
| `sumi/text` | `#E8E4DC` | text |
| `sumi/muted` | `#A3A7AF` | captions |

### Colour rules

- **Vermilion is only for the main action and seals.** One red button per screen.
- **Indigo — progress, growth, done steps, hints.**
- **Persimmon — attention and streaks.** Errors in a task are marked with ochre, not red: it is “almost there”, not an alarm.
- **Colour is never the only carrier of meaning.** Difficulty always has a label (“Easy”, “Master”), a status has an icon.
- Text always passes WCAG AA: 4.5:1 for normal text, 3:1 for large.

### The dark “Sumi” theme

The second interface theme is the second mode, “Тушь (тёмная)”, of the “FUDE / Цвет” variable collection (in Figma every screen is bound to the variables; the dark copies are on page “05 · Тёмная тема”). Paper and ink swap places, accent fills become deep, and accent text gets lighter to read on dark.

| Token | Light “Washi” | Dark “Sumi” |
|---|---|---|
| `paper/base` · `raised` · `sunken` · `line` | `#F3EEE4` · `#FBF8F2` · `#E9E2D4` · `#DDD4C3` | `#131418` · `#1C1E24` · `#24272E` · `#30343C` |
| `ink/900` · `700` · `500` · `300` | `#16161A` · `#3B3A40` · `#625E57` · `#ADA79C` | `#EDE7DB` · `#C8C2B6` · `#9C978E` · `#5E5B57` |
| `ai/900` · `700` · `500` · `300` · `100` | as above | `#0F2033` · `#9BBDE2` · `#4A7DB4` · `#37608A` · `#1B2B3E` |
| `shu/700` · `500` · `100` | as above | `#F2A897` · `#CD4C34` · `#3B1E19` |
| `kaki/700` · `100` | as above | `#F3C292` · `#3A2A18` |
| `kin/700` · `500` · `100` | as above | `#E5CD92` · `#CBA660` · `#332A17` |
| `night/plate` · `night/text` | `#16161A` · `#FBF8F2` | `#0E0F12` · `#ECE6DA` |

- `night/plate` and `night/text` are a dark plate in both themes and the text on it: the sidebar, the splash, “Sensei's advice”, dark buttons and selected chapters. Such plates used to be filled with `ink/900`, which turns light in the dark theme.
- The “editor's night” colours (`sumi/*`) are the same in both themes: the lesson studio and practice are always dark and keep the light theme's accents (the `.sumi-scope` class in the code).
- Task pictures, references and diagrams stay light: they are drawings, not interface (in Figma such frames have the light mode set explicitly).
- In the app the theme is chosen with the button next to the profile in the sidebar: light → dark → as in the system. The default is light; the choice is kept in `localStorage` and applied before the first paint, without a white flash.

## Fonts

| Style | Font | Size / line height | Purpose |
|---|---|---|---|
| Display/XL | Cormorant Garamond SemiBold | 68 / 102% | large headings |
| Display/L | Cormorant Garamond SemiBold | 46 / 108% | screen headings |
| Display/Italic | Cormorant Garamond SemiBold Italic | 46 / 108% | accents |
| Heading/H1 | Cormorant Garamond Bold | 32 / 115% | card headings |
| Heading/H2 | Manrope Bold | 20 / 130% | subheadings |
| Heading/H3 | Manrope SemiBold | 16 / 135% | lesson titles |
| Body/M | Manrope Regular | 15 / 160% | main text |
| Body/S | Manrope Regular | 13 / 150% | secondary text |
| Label/M | Manrope SemiBold | 13 / 125% | buttons, captions |
| Label/S | Manrope SemiBold | 12 / 120% | small captions (12 px minimum) |
| Label/Caps | Manrope Bold, CAPITALS | 12, 8% letter spacing | group headings |
| Number/XL, Number/L | **Cormorant Infant** Bold | 56 and 36 | numbers |
| Kanji/XL, Kanji/M | Yuji Syuku | 120 and 28 | category and seal kanji |
| Mono/S | JetBrains Mono Medium | 12 | shortcuts, time |

Numbers are set in **Cormorant Infant**, not Garamond: Garamond has old-style figures, and “1” reads as “I” (“I4 days”). Infant has regular figures with the same design. Infant does not suit words: its Cyrillic “ю” looks like “lo”.

## Ink and forms

The strokes, ensō and mountains are made from real ink strokes, not drawn in code: the “Sumi Strokes” brush set (Denny Tang, SparkleStock), from which [`design/ink-assets.py`](../../design/ink-assets.py) cuts out a few strokes and saves them as luminance masks (white is ink) in [`app/src/lib/assets/ink/`](../../app/src/lib/assets/ink/ATTRIBUTION.md). The components fill the mask with a token colour, so the same stroke works on washi and in the dark theme, and “drawing in” stays a clipping animation. These pictures are not under MIT; the terms are in the `ATTRIBUTION.md` next to them.

In Figma the ink works the same way. All masks are uploaded once: the “Тушь · маски” frame on page “01 · Дизайн-система”. Every piece of ink in the components and screens is a group of two layers:
- “маска · мазок туши”: a rectangle with the stroke picture, an alpha mask;
- “тушь”: a rectangle on top of it, filled with a colour variable (`ink/900`, `shu/500`, `ai/*`, `kin/500`…).

So the strokes recolour themselves in the “Тушь (тёмная)” mode. To replace a stroke, change the mask picture; to recolour it, change the variable of the “тушь” layer.

- The days of “A year in ink strokes” are brush taps.
- The week bars are strokes stood upright.
- The “Independence” gauge is a real ensō clipped to its upper half so that exactly 86% and the dry tail show.
- The opening animation: the brush paints the ensō itself. An arc-shaped “drawing” mask with a round end lies over the stroke. It grows around the circle after the brush (path trim 0 → 100%, 0.62–1.5 s) from the loaded start at the top right, clockwise, so you see one ensō with its gap at the top. The ensō is drawn the same way in the app. The old even line is hidden.
- The cover: the sun inside the ensō is a vermilion blot, not an even circle. It is a real stroke with a ragged edge, a short run, splashes and two flicks to one side (the “клякса” mask in the “Тушь · маски” frame, colour `shu/500`).

- **Brush stroke** — a horizontal stroke with a loaded start and a dry tail, 8 variants, a "seed" picks the number. The backing of the active menu item (drawn in from left to right), the stroke on a lesson cover.
- **Ensō** — a real stroke bent into a circle: 4 variants, starting 70° above "three o'clock" and running clockwise, like the drawing animation. The splash, the sidebar background, reward frames, the lesson result.
- **Small ink** — the five short strokes of the difficulty scale, the brush "taps" in “A year in ink strokes” (each day has its own, by date) and the week bars on “Activity” — a stroke stood upright: a loaded top, a dry foot. These masks keep the ink in the alpha channel and are set with the CSS `mask-image` property. In the dark theme the year has its own scale: the more tasks, the lighter the ink.
- **Hanko seal** — a red square with a kanji: the 筆 logo, the “done” mark 済, the lesson result 完 (done) and 再 (“again”, in ink).
- **Mountains** — three layers (far, middle, near): a dry stroke along the ridge and a wash below it melting into mist. Each layer is tinted with its own indigo token; a "seed" picks a stretch of ridges, so neighbouring cards do not repeat. On the left the mountains dissolve instead of ending at the edge of the block.

## Components (page “01 · Дизайн-система”)

| Component | Variants | Notes |
|---|---|---|
| Button | Main, Accent, Outline, Quiet, Hint | “Accent” (vermilion) — one per screen |
| Difficulty | Level 1–5 | five real ink strokes + a label |
| Chip | — | filters, 32 px high |
| Seal/Hanko | the “Знак” property | the kanji changes |
| Ink/Ensō, Ink/Stroke, Ink/Mountains | — | masks from real ink strokes, colour from a token |
| Sidebar | 5 sections | the active item is underlined with a vermilion stroke, search at the top |
| Lesson card | properties Title, Number, Duration, Sign | the category in a chip on the cover |
| Achievement | Received / Locked | a seal in a gold ensō, a locked one shows progress |
| Switch | — | a period or a mode |
| Hint | Step, Action, Explanation, Key | the number matches the task step |
| Task item | Done / Current / Ahead | for the editor panel |
| Progress | — | indigo |

Cards: 16 px corners, a soft shadow **without a border** (a border plus a shadow gives a "ghost card" effect). Small elements — 12 and 8 px, pills — 999.

## Mockup screens (page “02 · Экраны”, the dark ones — “05 · Тёмная тема”)

**Main sections:** 01 Home · 02 Lessons · 03 Achievements · 04 Activity · 05 Statistics.

**The lesson path:** 06 Lesson (video guide and theory) · 07 PhotoCraft editor (task and hint) · 08 Lesson result.

**States:** 09 Home — first day · 10 Editor — auto-check failed · 11 Result — try again · 12 Lesson — the video did not load · 13 Editor — did not start.

All screens show one consistent student state (Sunday, October 25, 2026: a 14-day streak, lesson 3 “A sphere's cast shadow” in progress), so the numbers do not contradict each other.

## Motion

The opening animation (page “03 · Анимация открытия”, video — [design/fude-splash.mp4](../../design/fude-splash.mp4)), 3.6 s:

| Time, s | Stage | Curve |
|---|---|---|
| 0.00–0.42 | a drop of ink falls | ease-in |
| 0.42–1.50 | the ink spreads over the paper | cubic-bezier(.16, 1, .3, 1) |
| 0.62–1.50 | the brush paints the ensō (path trim 0 → 100%) | cubic-bezier(.65, 0, .35, 1) |
| 1.50–2.10 | 筆 emerges (blur 22 → 0) | ease-out |
| 2.00–2.72 | the FUDE name and the subtitle | ease-out, 180 ms delay |
| 2.60–3.05 | the 学 seal strikes with a turn and a bounce | ease-in → ease-out |
| 3.05–3.60 | the transition to home | spring |

With the system setting "reduce motion" the splash shortens to 0.6 s. The interface micro-animations (the stroke under a menu item, screens appearing, the heat map filling, the seal striking on a reward, a hint "breathing") are described on the same mockup page.

## The editor theme

The PhotoCraft editor got two themes on these tokens: **FUDE Sumi** (dark, `sumi/*`) and **FUDE Washi** (light, `paper/*`). The mapping of tokens to theme fields and the tests that check it are described in [editor.md](editor.md).
