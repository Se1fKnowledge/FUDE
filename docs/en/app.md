# The FUDE app

[Русский](../app.md) · **English**

A desktop app on **Tauri 2**: the interface is **SvelteKit 3 + Svelte 5 + TypeScript** in a webview, the backend is Rust on top of the [`fude-core`](../../core/) lesson core. The PhotoCraft editor runs as a separate process and is driven through its JSON channel.

## Running

You need Rust 1.99+, Node.js 20+ and a built editor.

```bash
cd photocraft
cargo build --release -p photocraft      # once, ~6 minutes
cd ../app
npm install
npm run tauri dev                        # the app with hot reload of the interface
```

To launch with a click there is [`app/run-dev.cmd`](../../app/run-dev.cmd): it does the same (and `npm install` if the dependencies are not there yet) in a minimised console window. A shortcut to it with the `app/src-tauri/icons/icon.ico` icon is handy. Closing the FUDE window stops the console too.

Building an installer with the editor inside (from `app/`, after a release build of PhotoCraft):

```bash
npm run tauri build -- --config src-tauri/tauri.bundle.conf.json
```

The `.exe` (NSIS) and `.msi` end up in `target/release/bundle/`. The built-in lessons and `photocraft.exe` go into the installer as resources and after installation lie next to `fude.exe`. The extra config [`tauri.bundle.conf.json`](../../app/src-tauri/tauri.bundle.conf.json) keeps `tauri dev` from needing a built editor. The [`build.yml`](../../.github/workflows/build.yml) workflow does the same on GitHub — see [Building on GitHub](#building-on-github).

Where the app looks for PhotoCraft, in order:

1. the `FUDE_PHOTOCRAFT` environment variable — the full path to `photocraft.exe`;
2. next to `fude.exe` (as in the installer);
3. `photocraft/target/release/photocraft.exe` in the repository (development).

Without the editor the app still works, and the practice screen shows «The editor did not start» with an explanation.

## Theme

Light «Washi» or dark «Sumi» — the button next to the profile at the bottom of the sidebar switches light → dark → as in the system ([`theme.svelte.ts`](../../app/src/lib/theme.svelte.ts)). The dark theme tokens are `:root[data-theme='dark']` in [`styles.css`](../../app/src/lib/styles.css), with the values of the second variable mode in Figma ([design-system.md](design-system.md#the-dark-sumi-theme)).

## Language

Russian or English: the RU/EN button next to the profile in the sidebar and the choice on the welcome screen ([`i18n.svelte.ts`](../../app/src/lib/i18n.svelte.ts)). Changing the language reloads the window, so every screen and list is rebuilt in the new language.

- **Interface.** Russian strings stay in the code and serve as keys: `t('Главная')` gives «Home» from the [`i18n/en.ts`](../../app/src/lib/i18n/en.ts) dictionary; `{name}` in a string is a placeholder. Plurals and numbers — `count()` in [`format.ts`](../../app/src/lib/format.ts): the English forms live in `EN_PLURALS` under the Russian word. The [`i18n.test.ts`](../../app/src/lib/i18n.test.ts) test finds every `t(…)` string and requires an English entry with the same placeholders, and treats Russian text outside `t()` as an error.
- **Backend and core.** The language is stored in the database (the `setting` table) and applies from the first call after a start. Check messages, errors, ranks, achievements, greetings and dates are made by the `fude_core::tr!("ru", "en")` macro ([`core/src/i18n.rs`](../../core/src/i18n.rs)).
- **Lessons.** The built-in lessons are translated with `en.toml` and `theory.en.md` files ([lesson-format.md](lesson-format.md#languages)); your own lessons show in the language they are written in.
- **Editor.** PhotoCraft gets FUDE's language through the control channel (`prefs.set interface.language`): its menus and panels use the same language.
- **Journal.** Entries written before the language changed are re-worded when shown: «Пройден урок «…»» ↔ «Lesson done «…»», and lesson and achievement names in quotes follow their translations.
- **CLI.** `fude-cli` speaks Russian; with `FUDE_LANG=en`, English.

## Where the data lives

Everything is local, in `%APPDATA%\ru.fude.app\`:

| What | Where |
|---|---|
| progress (SQLite) | `fude.db` |
| own and imported lessons | `lessons\` |
| canvas snapshots of attempts (for the result screen) | `attempts\attempt-<N>.png` |
| the editor control channel token | `control.token` (new on every editor start) |

The `FUDE_DATA_DIR` environment variable replaces this folder entirely — for checks that must not touch real progress.

## Screens

| Screen | Path | Mockup |
|---|---|---|
| Splash (3.6 s, 0.6 s with "reduce motion") | at start | 03 · Opening animation |
| Welcome: name, daily goal, language | first launch | — |
| Home: current lesson, «Today», recommendations | `/` | 01, 09 |
| Lessons: categories, banner, filters, statuses | `/lessons` | 02 |
| Lesson: guide in the editor or video, theory, reference, questions | `/lesson/<id>` | 06, 12 |
| Practice: steps, auto-check, hints | `/practice/<id>` | 07, 10, 13 |
| Result: the 完 seal, similarity, time, hints, experience, skill growth | `/result/<attempt>` | 08, 11 |
| Achievements: the rank path 10 kyu → 1 dan, seals | `/achievements` | 03 |
| Activity: a year in ink strokes, streaks, the week, the journal | `/activity` | 04 |
| Statistics: skill profile, growth, experience, independence, sensei's advice | `/stats` | 05 |
| Workshop: own categories and lessons, `.fude` import and export | `/studio` | — |
| Lesson studio: texts, the guide on timeline tracks, the task with ghost lines and a live check | `/studio/edit?dir=…` | page «04 · Редактор уроков» |
| Through the student's eyes: a draft lesson as the student sees it, without recording progress | `/practice/<id>?preview=<folder>` | 04 · E6 |

## How practice works

1. «To practice in the editor» → `practice_start`: the backend starts PhotoCraft with `--control 0`, `--control-token-file`, `--automation-read-root <lesson folder>` and `--automation-write-root <attempts>`, opens the task picture (`app.open`) or creates a blank canvas, and starts an attempt in the database. The editor picks the port itself and reports it with the line `photocraft: control server listening on …` on stderr; FUDE reads that line and connects. FUDE used to look for a free port itself and pass it on, and if the port was taken in the meantime, practice after the guide failed with «port busy». If the editor reports that the channel did not open, or closes while starting, FUDE shows the reason at once. The guide and the practice of one lesson use the same running editor.
2. Every 4 seconds (or on «Check») — `practice_check`: `ui.inspect`, if needed a canvas snapshot with `app.save`, then `fude_core::assess`. Steps are passed in order and stay passed.
3. «Hint» (the F1 key) — `practice_hint`: a hint counts once per step; ghost shapes are drawn on the «FUDE · hint» layer («FUDE · подсказка» in Russian), and the layer is removed when the step is passed.
4. When all steps are passed, the attempt closes: experience, time, similarity to the reference, the journal, new achievements; the result screen opens.

## The editor in the FUDE window or on its own

The editor can be kept in two ways — for one small screen and for two monitors:

| Mode | What it looks like | How to switch |
|---|---|---|
| **In the FUDE window** (default) | PhotoCraft fills the FUDE window under the lesson toolbar; the task panel slides out on the right | the «Embed in FUDE window» button |
| **Separate window** | PhotoCraft has its own window, FUDE shows the steps, hints and reference | the «Separate window» button |

In practice the «Task» button hides and shows the task panel on the right, so on a small screen the canvas can have all the room. In the embedded mode the lesson guide draws right in the player on the lesson page. The choice is remembered.

How it works. PhotoCraft stays a separate process: its rules forbid wrapping the editor in a webview. On Windows FUDE finds the main window of the editor process and makes it a frameless child of its own window (`SetParent`, [`window.rs`](../../app/src-tauri/src/window.rs) — the only module with `unsafe`). A page marks the place for the editor with the `use:editorSlot` action ([`editor-slot.ts`](../../app/src/lib/editor-slot.ts)); when sizes change, FUDE passes the rectangle in physical pixels and the editor window follows. «Separate window» gives the editor back its frame and former place. Leaving the lesson pages closes the editor.

Keyboard. The embedded editor window gets the keyboard as soon as it is clicked with the mouse or a pen: the FUDE window catches the click in its child window (`WM_PARENTNOTIFY`) and passes focus to the editor. So all PhotoCraft shortcuts work inside FUDE, H («Hand») included. That is why the FUDE hint sits on F1: that key is free in PhotoCraft. While the practice screen is open and the FUDE window is active, F1 is registered as a hotkey (`RegisterHotKey`), so it works even when focus is inside the editor. The task check is Ctrl+Enter. Only a real application window counts as the editor window: winit's invisible helper window (`Winit Thread Event Target`, a transparent tool window) is skipped. It used to get embedded sometimes, while the editor itself stayed a separate window without a keyboard.

On macOS and Linux embedding is not supported yet: the editor always opens in its own window there, and the mode buttons are hidden.

## The lesson studio

Your own lessons are created and edited in the studio (`/studio/edit`), mockup — page «04 · Редактор уроков» in Figma. The studio is dark like the editor and fills the whole window. On the left is the lesson tree, in the middle the canvas, on the right the inspector of the selection. At the top there are three modes:

| Mode | What it has |
|---|---|
| **Lesson** | description and card (difficulty, minutes, skill areas, «opens after»), the theory in Markdown with an «as the student sees it» view beside it, questions after the theory |
| **Guide** | a canvas with the guide frame and timeline tracks below it: captions, highlight, actions, strokes. Clips are picked with a click, their length changes by dragging the right edge, the playhead plays the guide right on the canvas (Space) |
| **Task** | task steps on the left, the canvas with the task picture, the step inspector: text, hint, checks |

Canvas tools in the task: select and move (V), freehand line (B), straight line (L, Shift — at 45°), arrow (A), oval (O), rectangle (R), check area (M), delete a line (E); Alt while drawing — dashed. Each ghost line gets a number on the canvas and a row with the same number in the inspector, where its colour, opacity, width and dashing are set. The reference can be laid over the task with an opacity slider, and the lines of other steps shown faintly.

- **Where to click** is chosen on the interface map: a diagram of the PhotoCraft window with tools, panels and options, with search by name.
- **Live check.** Half a second after any edit of the task, each canvas check runs on the reference and on the untouched task (`fude_core::assess::prove`). The studio shows whether it passes on each. If a check lets an undone step pass or fails on the reference, it is highlighted and offers a threshold halfway between the two readings.
- **Lesson readiness** at the bottom left lists what is missing: description, theory, guide, steps, working checks.
- **Show in PhotoCraft** opens the task in the real editor (inside the FUDE window) and draws the hint of the selected step on the «FUDE · hint» layer.
- **The demo brush** in «Guide» mode draws a stroke right on the studio canvas; it becomes a clip on the «Strokes» track. A guide stroke moves into a step's hint as a ghost line with one button.
- **Recording in PhotoCraft.** The author draws in the real editor; every 0.3 s FUDE reads the editor's command journal (`journal.read`, see [editor.md](editor.md)) and the active tool. On «Stop» the recording becomes guide steps (`fude_core::record`): a tool change — a highlight and the tool choice, each stroke — a separate step with its brush and pressure, brush settings and layers — actions. Pauses longer than 2 s are shortened; commands a guide may not use (opening a file and so on) are skipped and named.
- **Play in PhotoCraft** saves the lesson and plays the guide in the editor; a step without a caption keeps the previous caption on screen.
- **Through the student's eyes** saves the lesson and opens its practice screen with a «Through the student's eyes» bar: checks, hints and F1 work as for the student, while the attempt is written to a temporary in-memory database and never reaches the progress. A draft with errors can be gone through too.

Working with the canvas and edits:

- **Undo and redo**: Ctrl+Z, Ctrl+Shift+Z (or Ctrl+Y) and the buttons at the bottom left. A history step is a pause in editing, so typing or dragging is undone as a whole; in input fields Ctrl+Z works as usual.
- **Zoom**: the −/+ buttons (50 to 400%), Ctrl+wheel, Ctrl+0 — fit. The «Hand» tool (H) moves the zoomed-in canvas.
- **Handles**: the ends of a selected straight line or arrow can be dragged, the corners of an oval or rectangle. Lines and handles lie above check areas, so clicking them does not select an area.
- **Drawn in**: a ghost line can appear gradually — in PhotoCraft it is drawn in pieces, as if by hand.

The guide:

- **Task steps** are the fifth track: a guide step can say which task step it shows. Neighbouring steps with the same task step merge into one clip, and in «Task» mode the line under the canvas shows from which second the guide shows the selected step.
- **Split** divides the step under the playhead: a single stroke — into two strokes at the playhead, any other step — into its actions and the pause.
- **Earlier / Later** in the inspector reorder a guide step.
- **Recording settings** in the recording bar: «Keep pauses as they are» (do not shorten pauses longer than 2 s) and «Record undos» (by default the author's Ctrl+Z does not go into the guide).

Checks and preview:

- The «Checks» tab has **live check thumbnails**: the reference and the untouched task with the step's areas and marks of how each check behaves on them.
- «Lesson readiness» rows with problems are links: they lead to the section, step or check that needs fixing.
- In «Through the student's eyes» mode a **«What the check sees»** panel under the step lists each check of the current step with its message; the student does not see it.
- The interface map shows PhotoCraft tools as icons.

Ctrl+S saves; leaving the studio with unsaved edits asks for confirmation.

## Experience, ranks, skills

- Experience per task: `40 + 20 × difficulty`, ×1.5 without hints, a repeat gives a quarter.
- Ranks: 10 kyu (0 XP) … 1 kyu (3150 XP), 1 dan (3800 XP).
- Skill: each passed lesson gives its skill areas `10 × difficulty` points (×1.5 without hints); the 0–100 scale is the share of everything the library lessons give.
- Independence is the share of tasks done without hints.
- A streak is the days in a row with at least one task done; yesterday keeps the streak until the end of today.
- Achievements are counted only from the student's real data. The «rarity among students» from the mockup is not shown: a local app has no other students.

## Code

```text
app/
├── src/                     interface (SvelteKit)
│   ├── lib/api.ts           types and calls of backend commands
│   ├── lib/ink.ts           ink pictures from real strokes: strokes, ensō, mountains
│   ├── lib/format.ts        numbers, plurals, dates in the interface language
│   ├── lib/i18n.svelte.ts   the interface language and t(); the dictionary — lib/i18n/en.ts
│   ├── lib/checks.ts        check kinds for the lesson editor
│   ├── lib/shortcuts.ts     practice keys: F1 hint, Ctrl+Enter check
│   ├── lib/editor-slot.ts   the place for the embedded editor, the mode choice
│   ├── lib/studio/          the lesson studio: canvas, guide tracks, interface map, inspectors
│   ├── lib/components/      sidebar, splash, seal, difficulty, area picker…
│   └── routes/              screens
└── src-tauri/               backend (Rust)
    ├── src/db.rs            SQLite: profile, attempts, steps, journal, achievements, settings
    ├── src/progress.rs      experience, ranks, streaks, skills, achievements
    ├── src/editor.rs        starting PhotoCraft and the control channel client
    ├── src/window.rs        embedding the editor window in the FUDE window (Windows)
    ├── src/practice.rs      an attempt: check, hints, result
    ├── src/learn.rs         catalog, lesson and progress commands
    ├── src/play.rs          guide and practice commands
    ├── src/studio.rs        workshop and package commands
    ├── src/author.rs        studio: live check, hint in the editor, guide recording
    ├── src/lang.rs          language: storage, the editor, the journal in the interface language
    └── tests/real_editor.rs all guides in the real PhotoCraft
```

## Tests

```bash
cargo test -p fude-app                                        # backend, 52 tests (with a fake editor)
cargo test -p fude-app --test real_editor -- --ignored        # 12 guides in the real PhotoCraft
cd app && npm test                                            # interface, 67 tests
cd app && npm run check                                       # Svelte and TypeScript types
```

## Building on GitHub

`.github/workflows/` has two GitHub Actions workflows; both run on `windows-latest`:

| Workflow | When | What it does |
|---|---|---|
| [`ci.yml`](../../.github/workflows/ci.yml) | every push to `main` and `SelfKnowledge`, every pull request | `npm run check`, interface tests, interface build, `cargo fmt --check`, clippy with `-D warnings`, `cargo test --workspace` |
| [`build.yml`](../../.github/workflows/build.yml) | a push to `main` and `SelfKnowledge` (a newer push cancels the previous build), `v*` tags, by hand (Actions → Build → Run workflow — once the file is on `main`) | builds PhotoCraft in release and the FUDE installers with the editor inside; uploads the `FUDE-windows-x64` artifact: the `.exe` and `.msi` installers plus the portable `FUDE-portable-windows-x64.zip` (`fude.exe`, `photocraft.exe`, `lessons/`) |

A tag like `v0.1.0` also creates a GitHub release with these files:

```bash
git tag v0.1.0
git push origin v0.1.0
```
