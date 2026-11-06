# Changes

[Русский](CHANGELOG.md) · **English**

Format: newest first. Versions will appear once the first finished build reaches `main`.

## Unreleased (the `SelfKnowledge` branch)

### Added

- Two languages — Russian and English — in everything there is: the interface (an RU/EN button next to the profile and a choice on the welcome screen), check messages and errors, ranks, achievements, dates, the PhotoCraft editor, all 12 built-in lessons with their theory and guides (`en.toml`, `theory.en.md`), the documentation (`docs/en/`, `README.en.md`), the CLI (`FUDE_LANG=en`). Tests require a complete translation of the interface and the built-in lessons.
- Ink from real strokes: the menu underline, the stroke on a lesson cover, the ensō (splash, sidebar, home, rewards, result) and the mountains are now made from the «Sumi Strokes» brushes instead of strokes drawn in code. The strokes are stored as masks and tinted with theme tokens; the drawing-in of the ensō and the menu stroke stayed. The difficulty scale, the days of «A year in ink strokes» and the week bars are real strokes too; in the dark theme the year gets lighter with the number of tasks instead of darker. The generator — [`design/ink-assets.py`](design/ink-assets.py), the pictures' licence — [`ATTRIBUTION.md`](app/src/lib/assets/ink/ATTRIBUTION.md).
- The dark «Sumi» theme: in Figma — the second mode of the «FUDE / Цвет» variables and the «05 · Тёмная тема» page with all screens; in the app — a switch next to the profile in the sidebar (light → dark → as in the system), the choice is saved and applied without a flash. Dark plates got their own tokens, `night/plate` and `night/text`.
- Editor: a FUDE icon (筆 on indigo with a seal) instead of the PhotoCraft logo — in the title, the taskbar, the `.exe` and the packages.
- The lesson studio instead of the former lesson editor ([docs/app.md](docs/en/app.md#the-lesson-studio), mockup — page «04 · Редактор уроков» in Figma): three modes, «Lesson», «Guide», «Task»; a canvas with the task picture where the hint's ghost lines (freehand, straight, arrow, oval, rectangle; colour, width, opacity, dashing; numbers on the canvas and in the list) and check areas are drawn; the reference over the task; an interface map for «where to click»; a live check of every check on the reference and the untouched task with a suggested threshold; lesson readiness; the guide on timeline tracks (captions, highlight, actions, strokes) with playback on the canvas and a demo brush; recording a guide while the author draws in PhotoCraft; «Show the hint in PhotoCraft», «Play in PhotoCraft», «Through the student's eyes» without recording progress.
- The lesson studio: undo and redo of edits, canvas zoom and the «Hand», end and corner handles on shapes, «drawn in» lines, the «Task steps» track, splitting and reordering guide steps, recording settings (pauses, undos), live check thumbnails, jumps from «Lesson readiness» to the problems, tool icons on the interface map, a «What the check sees» panel in «Through the student's eyes» mode.
- Lesson format: `style.reveal` for hint shapes and `task_step` for guide steps.
- Lesson format: the `arrow` hint shape and a `style` table (`color`, `width`, `opacity`, `dashed`) for any shape ([docs/lesson-format.md](docs/en/lesson-format.md#a-steps-hint)).
- Core: `assess::prove` (the live check), `record::to_guide` (the author's recording → guide steps).
- Editor: the `journal.read` control channel method — the journal of executed commands for recording a guide.
- Building on GitHub Actions: [`ci.yml`](.github/workflows/ci.yml) — type checks, interface and Rust tests, `cargo fmt`, clippy on every push and pull request; [`build.yml`](.github/workflows/build.yml) — Windows installers (`.exe`, `.msi`) and a portable `.zip` with PhotoCraft inside, a release on a `v*` tag. The [`tauri.bundle.conf.json`](app/src-tauri/tauri.bundle.conf.json) config puts `photocraft.exe` next to `fude.exe`.
- `FUDE_DATA_DIR` — a data folder of your own instead of `%APPDATA%\ru.fude.app` (for checks).
- Tests: starting the editor with a fake PhotoCraft process (success, channel refusal, exit at start, timeout), finding the editor, the database, pictures, helper windows, passing focus; in the frontend — practice keys, the place for the embedded editor, error texts. Backend: 26 → 45 tests, interface: 24 → 35.
- Click-to-launch in development mode: [`app/run-dev.cmd`](app/run-dev.cmd) (`npm run tauri dev` in a minimised console) — for a shortcut with the FUDE icon.
- The editor in the FUDE window: on Windows PhotoCraft is embedded in the FUDE window (the default mode) with a slide-out task panel; the «Separate window» button returns it to its own window, «Embed in FUDE window» brings it back. The lesson guide draws right in the player on the lesson page. The choice is remembered.
- Editor: the `ui.set {"hostEmbedded": bool}` control channel field — inside FUDE the editor does not draw its window buttons and does not move itself.
- The FUDE app ([app/](app/), [docs/app.md](docs/en/app.md)): Tauri 2 + SvelteKit 3 + Svelte 5. A splash as in the mockup, a first-day welcome, home, the lesson catalog, the lesson page (a guide in the editor or a video, theory, reference, questions), practice with an auto-check every 4 seconds and hints, the lesson result, achievements with the rank path, activity («a year in ink strokes», streaks, the week, the journal), statistics (skill profile, growth, experience by week, independence, sensei's advice), the workshop with the lesson editor and `.fude` import/export.
- The app backend: progress in SQLite, experience and kyu/dan ranks, streaks, skills, achievements; starting PhotoCraft with a control channel and a token; playing guides; practice with hints on a separate layer.
- The `real_editor` test: all 12 built-in guides solve their tasks in the real PhotoCraft.
- The app icon: the 筆 seal.
- The format of lessons, checks, guides and `.fude` packages ([docs/lesson-format.md](docs/en/lesson-format.md)).
- The `fude-core` lesson core ([core/](core/)): the lesson model, a library of built-in and own lessons, format checks, guides of PhotoCraft commands with an allowlist, hint ghost shapes, step auto-check from the editor state and a canvas snapshot, `.fude` import and export with zip-slip protection and size limits, lesson editor operations (create, save, copy a built-in lesson, upload videos and pictures); the `fude` CLI; 93 tests.
- Built-in lessons ([lessons/](lessons/)): «Interface basics» and «Light and shadow», 6 lessons each with theory, questions, guides, hints and checks; the `core/examples/lesson_art.rs` picture and guide generator.
- Decisions in [PRODUCT.md](PRODUCT.en.md) and [docs/architecture.md](docs/en/architecture.md): local storage only (SQLite), SvelteKit + TypeScript, own lessons and categories, import from a file, video upload.
- The product description [PRODUCT.md](PRODUCT.en.md): users, features, open questions.
- The Figma mockup: 13 screens (the main sections, the lesson path, first-day and error states), the design system, the opening animation; the splash video [design/fude-splash.mp4](design/fude-splash.mp4).
- Editor: a copy of PhotoCraft (storytold/photocraft@c65b5d2) in [`photocraft/`](photocraft/).
- Editor: the **FUDE Sumi** and **FUDE Washi** themes as defaults; the `window.theme.fudeInk` and `window.theme.fudeWashi` commands.
- Tests of the FUDE themes (tokens, contrast, surface order, installing into egui), settings, menu commands and the absence of the ArtCraft brand.
- Documentation in [`docs/`](docs/en/).
- The MIT licence for FUDE's own code and materials ([LICENSE](LICENSE)); the editor stays under MIT OR Apache-2.0.

### Changed

- Rust updated to the latest version: FUDE needs Rust 1.99 (was 1.85); Tauri 2.12 and the dialog plugin 2.8 — in step with the npm packages; dependencies updated.
- The lesson core CLI renamed to `fude-cli` so it does not clash with the app's `fude.exe`.
- The «Layers» lesson guide hides the background, not the sun layer; `layer.selectTop` and `layer.selectBottom` added to the guide allowlist.
- Editor: «Report an issue» leads to FUDE Issues; the «About» window and the start screen have «PhotoCraft on GitHub · FUDE» links.
- Editor: the addresses in the Windows and Linux installers point to the FUDE repository.

### Removed

- Editor: the whole update check (the request to api.github.com, the «PhotoCraft updates» window, the `help.checkForUpdates` command, the automatic check setting, the `updates` and `updatesOpen` control channel fields, the `ureq` dependency) — the editor no longer uses the network.
- Editor: links to the PhotoCraft repository — «PhotoCraft on GitHub» became «FUDE on GitHub», the authors' names in «About» are text without links.
- Editor: the ArtCraft name, logos and links (the `docs/brand/` logos, «Help» menu items, the Discord button, translation strings, links in the documentation) — under the brand terms for forks.

### Fixed

- FUDE reconnects to PhotoCraft if the editor closed an idle connection (before, for example, recording after showing a hint failed with «os error 10053»).
- When a guide plays, a step without a caption keeps the previous caption on screen.
- The embedded editor did not get the keyboard: sometimes winit's invisible helper window got embedded while the real editor stayed apart. Now only the application window is taken, and a mouse or pen click on the editor gives it focus — PhotoCraft shortcuts work inside FUDE.
- The H key switched on both the «Hand» in the editor and the FUDE hint. The hint is now on F1 (it works with focus in the editor too, while the FUDE window is active), the check on Ctrl+Enter.
- «Port busy» when going from the guide to practice: FUDE no longer picks the port itself; the editor opens the channel on a free port (`--control 0`) and reports it; editor start errors show at once.
- Editor: the initial interface theme was hard-coded as ProMedium, so a saved theme might not apply at start; now the default theme is taken.
- Editor: panel docking did not count the new themes as the Pro layout, and tab dragging broke.
