# FUDE built-in lessons

[Русский](README.md) · **English**

Two categories of six lessons, in Russian and English. Format — [docs/en/lesson-format.md](../docs/en/lesson-format.md). To check:

```bash
cargo run -p fude-core --bin fude-cli -- validate lessons
```

## 基 Interface basics (`basics`)

| № | Lesson | Difficulty | What the student does | Checks |
|---|---|---|---|---|
| 1 | Canvas and zoom | 1 | finds a seal in the corner with the magnifier, moves the canvas with the «Hand», shows the whole view again | tools, zoom |
| 2 | Brush: size, hardness, colour | 1 | sets up the brush, lays three soft ink strokes and a vermilion stroke | tool, brush settings, foreground colour, painted areas |
| 3 | Eraser and undo | 1 | erases a blot without touching the circle; undoes a random stroke | brightness where the blot was, the untouched circle, history |
| 4 | Layers | 2 | draws a sun on a separate «Sun» layer, hides the background | layer count, the layer's name and visibility, the painted area |
| 5 | Selection and fill | 2 | fills an indigo rectangle and a vermilion circle through selections | tools, selection size, area colours |
| 6 | Layer opacity and visibility | 2 | lays a band over a stone and makes the layer translucent | layer opacity, the stone showing through the colour, visibility |

## 影 Light and shadow (`light`)

| № | Lesson | Difficulty | What the student does | Checks |
|---|---|---|---|---|
| 1 | Light source and form | 2 | fills a ball with a mid tone and lightens the side towards the sun | layer, the filled ball, light lighter than shadow |
| 2 | Core shadow and halftone | 3 | lays the core shadow and the halftone with a soft brush | brush settings, a dark shadow, the tone order light → halftone → shadow |
| 3 | A sphere's cast shadow | 3 | draws the shadow on the floor, darker at the point of contact | the «Shadow» layer, the shadow on the right and not on the left, the shadow gradient |
| 4 | Reflected light | 4 | adds reflected light at the edge in the shadow | layer, brush, reflected light lighter than the shadow and darker than the light |
| 5 | Light on a complex form: drapery | 4 | models the folds of a cloth with light and shadow | brush, the overall tone, the depth of folds darker than the ridges |
| 6 | Backlight and atmosphere | 5 | draws a halo and a glowing outline around a silhouette | layer, the glow above the ball, a light outline, a dark middle |

## What is in a lesson folder

- `lesson.toml` — description, questions, task, steps, hints, checks;
- `theory.md` — the theory with «Sensei's advice»;
- `guide.json` — the animated guide: PhotoCraft commands with captions;
- `assets/task.png` — what the task starts from (if not a blank canvas);
- `assets/reference.png` — the reference: what the finished task can look like;
- `en.toml` and `theory.en.md` — the lesson in English ([docs/en/lesson-format.md](../docs/en/lesson-format.md#languages)).

The pictures and guides are drawn by the [`core/examples/lesson_art.rs`](../core/examples/lesson_art.rs) generator. The theory, questions and tasks are written by hand.
