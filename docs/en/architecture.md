# FUDE architecture

[Русский](../architecture.md) · **English**

> Status: everything in the diagram works — the editor with the FUDE theme, the lesson core, the built-in lessons, the Tauri + SvelteKit app with progress in SQLite. More on the app in [app.md](app.md).

## Overview

```text
┌─────────────────────────────────────────────────────┐   JSON command channel ┌──────────────────────────┐
│  The FUDE app (Tauri)                               │  ────────────────────▶ │  The PhotoCraft editor   │
│                                                     │   ui.set, ui.pointer,  │  (a separate process,    │
│  interface (SvelteKit + TypeScript, webview):       │   engine.execute,      │   egui + wgpu,           │
│  home, catalog, lesson, lesson studio,              │   app.open / app.save  │   the FUDE Sumi theme)   │
│  achievements, activity, statistics                 │  ◀──────────────────── │                          │
│                                                     │   ui.inspect: tool,    │  --control <port>        │
│  the fude-core core (Rust):                         │   brush, layers,       │  --automation-read-root  │
│  lessons, checks, .fude packages, guides, hints     │   zoom, history        │  --automation-write-root │
│                                                     │                        │                          │
│  SQLite: progress, activity, statistics             │                        │                          │
└─────────────────────────────────────────────────────┘                        └──────────────────────────┘
          │ reads and writes
          ▼
  lesson folders: built-in (lessons/, read-only) and the user's (%APPDATA%\ru.fude.app\lessons)
```

Everything runs on the student's computer. There are no accounts, no server and no sync.

## Why the editor is a separate process

- PhotoCraft is a native egui/wgpu app. Its rules ([`photocraft/AGENTS.md`](../../photocraft/AGENTS.md)) forbid wrapping it in Tauri or a webview.
- In PhotoCraft every action is a command, and there is a JSON control channel (`--control <port>`, protocol — [`photocraft/docs/control-protocol.md`](../../photocraft/docs/control-protocol.md)). Through it the app:
  - opens the task file (`app.open`) and saves the canvas for checking (`app.save`);
  - picks tools and settings (`ui.set`, `tools.setBrush`);
  - plays a guide: strokes (`paint.stroke`), layers, selections;
  - reads the state (`ui.inspect`): active tool, brush, zoom, layers, selection, history;
  - sets the theme (`ui.set {"theme":"fudeInk"}`) and the interface language (`prefs.set interface.language`).
- The editor opens and writes files only inside two roots the app gives it at start: `--automation-read-root` — the lesson folder, `--automation-write-root` — a temporary attempt folder. The editor itself rejects absolute paths and `..`.
- On Windows the editor window is embedded in the FUDE window: FUDE makes the main window of the PhotoCraft process a frameless child of its own window and places it over the spot the page marks; «Separate window» gives it back. The process stays separate, and with `ui.set {"hostEmbedded": true}` the editor hides its window buttons. Details in [app.md](app.md#the-editor-in-the-fude-window-or-on-its-own).
- The channel listens only on `127.0.0.1` and requires a token. The app creates the token file in its data folder, passes it with `--control-token-file` and sends `auth` as the first line of every connection.

## Parts of the app

| Part | What it does | Status |
|---|---|---|
| The `fude-core` core ([`core/`](../../core/)) | lesson model, loading the library, format checks, `.fude` packages, authoring, auto-check, guides, hints, languages | done, tests |
| The `fude-cli` CLI | check a lessons folder, build and unpack a package, check a canvas snapshot | done |
| Built-in lessons ([`lessons/`](../../lessons/)) | «Interface basics» and «Light and shadow», 6 lessons each, in Russian and English | done |
| Shell ([`app/src-tauri/`](../../app/src-tauri/)) | Tauri: the window, starting the editor, commands on top of the core | done, tests |
| Interface ([`app/src/`](../../app/src/)) | SvelteKit 3 + Svelte 5 + TypeScript, the mockup screens | done |
| Progress storage | SQLite: attempts, steps, hints, journal, achievements | done, tests |

## The `fude-core` core

Modules ([`core/src/`](../../core/src/)):

| Module | What it does |
|---|---|
| `model` | the `Category`, `Lesson`, `Task`, `Step`, `Check`, `Hint`, `Quiz` types and reading them from TOML |
| `library` | walks the lesson roots, collects categories and lessons; a bad lesson does not break the catalog but goes to the list of problems |
| `validate` | checks a lesson: fields, difficulty 1–5, paths inside the lesson folder, existing files, allowed extensions, unique steps, dependencies |
| `guide` | reads a guide script and lets through only allowed editor commands |
| `hint` | turns a hint's ghost shapes into editor commands (the «FUDE · hint» layer, translucent strokes) |
| `assess` | checks a task step from the editor state and a canvas snapshot |
| `package` | export and import of `.fude` packages (zip) with zip-slip protection and size limits |
| `authoring` | creates and saves categories and lessons, transliterates titles into folder names, attaches videos and pictures |
| `i18n` | the language of messages (Russian or English) and the `tr!` macro |
| `locale` | lesson translations: `en.toml` and `theory.en.md` next to a lesson |

The lesson and package format is in [lesson-format.md](lesson-format.md).

### Guides

A lesson has two kinds of guide, possibly both:

- **Animated guide** — a `guide.json` script: a sequence of editor commands with captions and pauses. The app plays it in the real PhotoCraft, and the student watches the brush paint on the canvas. A script can call only commands from the allowlist (`guide::ALLOWED_COMMANDS`): drawing, layers, selection, undo, brush settings. Files, saving, scripts and the network are forbidden, so an imported lesson from someone else cannot do harm.
- **Video** — an `mp4` or `webm` file the author uploads into the lesson. The webview plays it with its built-in player.

### Task checks

A task step counts when all its checks pass. There are two kinds of checks:

- from the editor state in `ui.inspect`: tool, brush, zoom, layers, selection, history;
- from a canvas snapshot (`app.save` to a PNG in a temporary folder): the painted share of an area, brightness, «area A darker than B», colour, similarity to the reference.

Areas are given in fractions of the canvas (`[x, y, width, height]` from 0 to 1), so they do not depend on the document size.

### Hints

A step's hint is a text, a target in the interface (`tool:Brush`, `panel:Layers`, `menu:edit.undo`) the app points a «start here» ring at, and ghost shapes. The core turns the shapes into commands: a new «FUDE · hint» layer at 35% opacity, strokes in vermilion. When the step is done, the app removes that layer. Every hint opening goes into the statistics.

## Storage

SQLite in `%APPDATA%\ru.fude.app\fude.db` ([`db.rs`](../../app/src-tauri/src/db.rs)):

- `profile` — name and daily goal;
- `attempt` — lesson, start, end, outcome, hints, time, experience, similarity to the reference;
- `step` — which step is passed and whether with a hint;
- `journal` — events for the home screen and the activity journal;
- `achievement` — when an achievement was received;
- `stage` — at which stage of a lesson the student stopped;
- `setting` — settings, for now just one: the interface language.

Activity by day, streaks, skills and achievements are not stored separately but computed from the attempts ([`progress.rs`](../../app/src-tauri/src/progress.rs)), so they cannot drift apart from them.

## Decisions

- Local storage only, no accounts.
- The interface is SvelteKit + TypeScript inside Tauri.
- Lessons are folders with `lesson.toml`; they travel as `.fude` packages.
- The user can create their own categories and lessons in the built-in lesson editor and import others' from a file.
- A guide is a script of PhotoCraft commands and/or an uploaded video.
- Two languages, Russian and English, in the interface, the lessons and the documentation.
- FUDE's own code is under the MIT licence ([LICENSE](../../LICENSE)).
