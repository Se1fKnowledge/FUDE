# The FUDE lesson format

[Русский](../lesson-format.md) · **English**

Format version: **1**. It is read and checked by the [`fude-core`](../../core/) core; a folder can be checked with:

```bash
cargo run -p fude-core --bin fude-cli -- validate lessons
```

## The library

A library is one or more roots with categories:

- the built-in lessons — the [`lessons/`](../../lessons/) folder, shipped with the app, read-only;
- your own and imported lessons — `%APPDATA%\ru.fude.app\lessons`, created by the lesson editor and by import.

```text
lessons/
└── light/                      a category (folder name = category id)
    ├── category.toml
    └── 03-sphere-shadow/       a lesson (any folder name, usually NN-slug)
        ├── lesson.toml         the lesson description, task, checks
        ├── theory.md           the theory (Markdown)
        ├── guide.json          the animated guide (optional)
        ├── media/guide.mp4     a video (optional)
        ├── en.toml             English translation (optional, see “Languages”)
        └── assets/             task pictures, the reference
```

If two roots have a category with the same id, their lessons are merged. If a lesson id repeats, the first root (built-in) wins and the second lesson goes to the library's list of problems.

## category.toml

```toml
id = "light"                 # a-z, 0-9, “-”; up to 48 characters; matches the folder name
title = "Свет и тень"
kanji = "影"                 # optional: the kanji on the card
description = "Как свет лепит форму: от источника до рефлекса."
order = 2                    # order in the catalog
accent = "shu"               # the category colour: ai | shu | kaki | kin
author = "FUDE"              # optional
```

## lesson.toml

```toml
format = 1
id = "light.sphere-shadow"   # <category id>.<slug>; unique in the whole library
title = "Падающая тень от сферы"
summary = "Тень на полу: где она начинается, куда ложится и почему темнее у основания."
difficulty = 3               # 1 — first day … 5 — master
minutes = 20                 # approximate duration
order = 3                    # order within the category
skills = ["light", "form"]   # skill areas for the statistics
requires = ["light.core-shadow"]  # optional: lessons that unlock this one
author = "FUDE"              # optional

[theory]
file = "theory.md"

[guide]                      # at least one of script and video
script = "guide.json"
video = "media/guide.mp4"    # mp4 or webm
poster = "media/poster.png"  # optional: the frame before the video starts

[[quiz]]                     # optional: questions after the theory
question = "Где падающая тень темнее всего?"
options = ["У дальнего края", "У точки касания", "Везде одинаково"]
answer = 1                   # the index of the right option, from zero
explain = "У точки касания свет почти не попадает под предмет."

[task]
brief = "Нарисуйте падающую тень от сферы на полу."
open = "assets/task.png"     # the picture the task starts from
reference = "assets/reference.png"  # optional: the reference for the similar check

[[task.steps]]
id = "layer"                 # unique within the lesson
text = "Создайте новый слой для тени."
  [task.steps.hint]
  text = "Кнопка “Новый слой” внизу панели “Слои”."
  target = "panel:Layers"
  [[task.steps.checks]]
  kind = "layers"
  count = { min = 2 }
```

Instead of `open` you can set a blank canvas:

```toml
[task.canvas]
width = 1200
height = 800
background = "#FFFFFF"
```

Exactly one of `task.open` and `task.canvas` is required.

### Skill areas

`tools`, `line`, `form`, `light` (light and shadow), `color`, `composition`.

### Paths

All paths are relative, with `/`, inside the lesson folder. Absolute paths, `..`, `\` and drive letters are forbidden. Allowed extensions: `md`, `json`, `toml`, `txt`, `png`, `jpg`, `jpeg`, `webp`, `mp4`, `webm`.

## A step's hint

```toml
[task.steps.hint]
text = "Возьмите мягкую кисть и проведите от точки касания вправо."
target = "tool:Brush"        # where to point the “start here” ring

[[task.steps.hint.shapes]]   # ghost shapes on the canvas
kind = "ellipse"             # ellipse | rect | line | path
rect = [0.42, 0.72, 0.36, 0.08]

[[task.steps.hint.shapes]]
kind = "path"
points = [[0.2, 0.5], [0.4, 0.45], [0.6, 0.5]]
style = { color = "#2F5D8C", width = 8, opacity = 0.5, dashed = true }

[[task.steps.hint.shapes]]
kind = "arrow"               # a line with an arrowhead at point to
from = [0.5, 0.8]
to = [0.75, 0.8]
```

Targets (`target`): `tool:<Tool>` (as in PhotoCraft: `Brush`, `Eraser`, `RectMarquee`, `EllipseMarquee`, `Zoom`, `Hand`, `Move`, `PaintBucket`, …), `panel:<Panel>` (`Layers`, `Color`, `Brushes`, `History`), `menu:<command id>` (`edit.undo`, `select.deselect`, …), `option:<option>` (`brushSize`, `hardness`, `opacity`).

Shape kinds: `ellipse` and `rect` (the `rect` field), `line` and `arrow` (the `from` and `to` fields), `path` (the `points` field, 2 to 2000 points).

Shape coordinates are fractions of the canvas from 0 to 1. The app draws them on a separate “FUDE · hint” layer and removes the layer when the step is passed.

The optional `style` table sets how a shape looks; without it a shape is vermilion (`#BC3823`), 35% opacity, 0.6% of the shorter side of the canvas wide, solid:

| Field | What it sets |
|---|---|
| `color` | a `#rrggbb` colour |
| `width` | the line width in pixels of the task picture, 0.5 to 200 |
| `opacity` | opacity from 0.05 to 1 |
| `dashed` | `true` — dashed (dashes of 4 widths, gaps of 3) |
| `reveal` | `true` — the line appears gradually, as if by hand: drawn in ten pieces with a 45 ms pause (not combined with dashing) |

Shapes are easier to draw in the lesson studio (see [app.md](app.md#the-lesson-studio)) than to write by hand.

## Checks

Each check is a `[[task.steps.checks]]` table with a `kind` field. A step is passed when all its checks pass. Any check can have a `message` field — what to show the student if it fails; otherwise the core composes the message itself, in the interface language.

Steps are passed in order, and a passed step stays passed (`assess::Progress`): if the student changes the tool at step 3, step 1 “Pick the brush” is not "undone". So a check that must hold to the end (for example, “the light has not darkened”) goes into the same step as the work it guards, not into a separate step. A step without checks counts with the “Done” button.

Ranges are written as `{ min = …, max = … }`; either bound can be left out.

Areas (`region`, `than`) are `[x, y, width, height]` in fractions of the canvas.

### From the editor state

| kind | Fields | What it checks |
|---|---|---|
| `tool` | `tool` | the active tool |
| `brush` | `size` (px), `hardness` (0–1), `opacity` (0–1) | brush settings |
| `zoom` | `zoom` (1 = 100%) | the zoom of the active window |
| `layers` | `count` | the number of layers in the document (groups included) |
| `layer` | `name` (part of the name, case-insensitive), `visible`, `opacity`, `type` (`Pixel`, `Group`, `Type`, `Shape`, …) | a layer matching all conditions exists |
| `selection` | `present` (default `true`), `width`, `height` (fractions of the canvas) | whether there is a selection and its size |
| `history` | `entries`, `can_redo` | the number of history steps; `can_redo = true` — the last step was undone |
| `foreground` | `color` (`#rrggbb`), `tolerance` (default 0.1) | the foreground colour |

### From a canvas snapshot

The app saves the canvas to a PNG (`app.save`) and compares it with the original task picture (or with the blank canvas colour). Transparent pixels are laid on white.

| kind | Fields | What it checks |
|---|---|---|
| `coverage` | `region`, `changed` (0–1) | what share of the area changed (a pixel changed if some channel moved by more than 24 of 255) |
| `luminance` | `region`, `value` (0–1) | the average brightness of the area |
| `darker` | `region`, `than`, `by` (default 0.05) | the area is darker than another by at least `by` |
| `color` | `region`, `color`, `tolerance` (default 0.12) | the average colour of the area is close to the given one |
| `similar` | `reference` (default `task.reference`), `score` (0–1) | similarity to the reference: both pictures are shrunk to 64×64 in greyscale, score = 1 − the average difference |

## guide.json — the animated guide

```json
{
  "format": 1,
  "title": "Падающая тень",
  "open": "assets/task.png",
  "steps": [
    {
      "caption": "Создаём слой для тени",
      "highlight": "panel:Layers",
      "calls": [
        { "method": "engine.execute", "params": { "command": "layer.new.layer", "params": { "name": "Тень" } } }
      ],
      "pause": 900
    },
    {
      "caption": "Мягкая кисть, малая непрозрачность",
      "highlight": "tool:Brush",
      "calls": [
        { "method": "ui.set", "params": { "tool": "Brush" } },
        { "method": "engine.execute", "params": { "command": "tools.setBrush", "params": { "size": 60, "hardness": 0, "opacity": 0.3 } } }
      ]
    }
  ]
}
```

- `open` or `canvas` (as in `task`) — what the guide starts from; can be left out, then the guide runs in the current document.
- `caption` — the caption at the bottom of the screen (an empty one keeps the previous); `highlight` — a target in the interface (the same as for hints); `pause` — the pause after the step in milliseconds (0–15000, default 600).
- `task_step` — optional: the id of the task step this part of the guide shows. Only the author needs it — the studio draws the “Task steps” track from it; the player does not read it.
- Allowed methods: `ui.set` (only `tool`, `zoom`, `center`, `rotation`, `fit`, `brushSize`), `ui.pointer` and `engine.execute`.
- Allowed `engine.execute` commands: `paint.stroke`, `layer.new.layer`, `layer.setProps`, `layer.select`, `layer.selectTop`, `layer.selectBottom`, `layer.delete`, `select.rect`, `select.all`, `select.deselect`, `select.inverse`, `edit.fill`, `edit.undo`, `edit.redo`, `tools.setBrush`, `tools.setColors`, `tools.defaultColors`, `tools.swapColors`, `file.new`.
- Limits: up to 400 steps, up to 2000 calls, up to 5000 points in one stroke.

Nothing else (opening and saving files, scripts, shortcuts, the network) can be called by a script: so an imported lesson from someone else cannot harm the student's computer.

## Languages

A lesson is written in one language (the built-in ones in Russian) and can get a translation without changing its files:

```text
01-canvas/
├── lesson.toml
├── theory.md
├── en.toml          the English translation of the lesson's texts
└── theory.en.md     the theory in English
```

`en.toml` is one table: each original text of the lesson and its translation.

```toml
[text]
"Холст и масштаб" = "Canvas and zoom"
"Выберите инструмент “Масштаб” (Z)." = "Pick the “Zoom” tool (Z)."
"Ctrl+0 — показать холст целиком" = "Ctrl+0 — show the whole canvas"
```

- Every text the student reads is translated: the title, the summary, the task brief, steps, hints, check messages, questions, options, explanations and the step captions of `guide.json`.
- The key is the whole original text (without spaces at the ends). So a translation does not break when steps are reordered, and a string without a translation simply stays as it is.
- Texts without Russian letters (`Ctrl+0`, `#BC3823`) need no translation.
- `theory.en.md` replaces `theory.md` entirely. Without the file the original theory is shown.
- A category is translated the same way: `en.toml` next to `category.toml` (title and description).
- The library applies the translation when it shows lessons to a student in the English interface. The lesson studio always opens the original, so saving never writes English into `lesson.toml`.
- `fude-cli validate` warns about `en.toml` entries that are not in the lesson (a typo, or stale after an edit). The `every_builtin_lesson_and_category_is_fully_in_english` test requires every text of the built-in lessons to be translated.

## The .fude package

A package is a zip archive for sharing lessons.

```text
my-lessons.fude
├── fude-package.toml
└── <category id>/
    ├── category.toml
    └── <lesson folder>/…
```

```toml
# fude-package.toml
format = 1
title = "Мои уроки акварели"
created_by = "FUDE 1.0.0"
categories = ["watercolor"]
```

On import the core:

1. rejects paths with `..`, absolute ones, with `\` and drive letters (zip-slip), and links;
2. limits the package: up to 4000 files, a file up to 1 GiB, all together up to 4 GiB, extensions from the list above;
3. unpacks into a temporary folder and checks each lesson;
4. refuses if a lesson id is already in the library (unless replace is chosen);
5. moves the categories into the user root and returns a report: what was added, what was replaced, warnings.

Export collects the chosen category or separate lessons together with their category's `category.toml`. Translations (`en.toml`, `theory.en.md`) travel with the lessons.

## The `fude-cli` CLI

```bash
cargo run -p fude-core --bin fude-cli -- help
```

Messages are in Russian; set `FUDE_LANG=en` for English.

| Command | What it does |
|---|---|
| `fude-cli validate <root>...` | check lesson folders, print the catalog and all errors |
| `fude-cli pack <category folder> <file.fude> [--title T] [--lesson <lesson folder>]...` | build a package from a category or separate lessons |
| `fude-cli info <file.fude>` | show what is in a package and check it |
| `fude-cli import <file.fude> <own lessons> [--builtin <root>] [--replace]` | import a package |
| `fude-cli new-category <own lessons> <title> [--kanji K] [--accent ai\|shu\|kaki\|kin]` | create a category |
| `fude-cli new-lesson <category folder> <title> [--difficulty 1..5]` | create a lesson from the template |
| `fude-cli guide <lesson folder>` | print the guide commands as JSON, one per line — they can be sent to the PhotoCraft channel |
| `fude-cli assess <lesson folder> <inspect.json> [canvas.png] [--confirm <step>]...` | check a task from a saved `ui.inspect` reply and a canvas snapshot |

## The built-in lessons and their pictures

The task pictures, references and guide scripts of the built-in lessons are drawn by the [`core/examples/lesson_art.rs`](../../core/examples/lesson_art.rs) generator:

```bash
cargo run -p fude-core --example lesson_art -- lessons
```

The geometry (ball, shadow, folds) is set in it once and agrees with the check areas in `lesson.toml`. The `canvas_checks_pass_on_the_reference_and_fail_on_the_untouched_task` test makes sure every canvas check passes on the reference and fails on the untouched task.
