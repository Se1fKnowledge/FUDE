# Work plan

[Русский](../roadmap.md) · **English**

## Done

- [x] Product description: users, features, limits ([PRODUCT.md](../../PRODUCT.md)).
- [x] Figma mockup: 13 screens, including the first day and errors; the design system and components.
- [x] An impeccable review of the mockup (two independent reviews) and fixes of what was found: contrast, small text, one meaning per colour, consistent data.
- [x] The opening animation on Figma keyframes and the [design/fude-splash.mp4](../../design/fude-splash.mp4) video.
- [x] Editor: PhotoCraft with the FUDE Sumi and FUDE Washi themes as defaults.
- [x] ArtCraft trademarks removed from the editor.
- [x] Theme, settings, menu and brand tests; a full run of the editor tests.
- [x] The repository, the `main` and `SelfKnowledge` branches, documentation.
- [x] Decisions: local storage only (SQLite), SvelteKit + TypeScript, lessons are folders with `lesson.toml`, guides are scripts of PhotoCraft commands and an uploaded video.
- [x] The lesson and `.fude` package format ([lesson-format.md](lesson-format.md)).
- [x] The `fude-core` lesson core: model, library, format checks, guides with a command allowlist, hints, task auto-check, package import and export, lesson editor operations; the `fude-cli` CLI; 112 tests.
- [x] Built-in lessons: «Interface basics» and «Light and shadow», 6 lessons each with theory, questions, guides, hints and checks; a picture and guide generator.
- [x] The Tauri + SvelteKit app: splash, welcome, home, lessons, lesson, practice, result, achievements, activity, statistics, workshop ([app.md](app.md)).
- [x] Starting PhotoCraft with a control channel and a token; the guide plays in the editor; step auto-check and hints with ghost shapes on the canvas.
- [x] Progress in SQLite: experience, kyu/dan ranks, streaks, skills, independence, achievements, journal.
- [x] Lesson editor: categories, description, theory with preview, video, guide script, questions, steps and checks with an area picked with the mouse; `.fude` import and export.
- [x] All 12 built-in guides solve their tasks in the real PhotoCraft.
- [x] An editor with no ties to the PhotoCraft repository: the update check and upstream links are removed.
- [x] The editor in the FUDE window with a slide-out task panel or in its own window; the guide draws right in the lesson player.
- [x] Click-to-launch in development mode: `app/run-dev.cmd` and a shortcut.
- [x] Windows installers (`.exe`, `.msi`) and a portable version with PhotoCraft inside are built by GitHub Actions ([`build.yml`](../../.github/workflows/build.yml)).
- [x] The lesson studio: the task canvas with ghost lines and check areas, the guide on timeline tracks, recording a guide from PhotoCraft, the live check, «Through the student's eyes».
- [x] The dark «Sumi» theme in Figma and in the app; the FUDE icon for the editor.
- [x] Ink from real brush strokes: menu strokes, ensō, mountains, difficulty, the activity year — in the app and in Figma.
- [x] Two languages, Russian and English: the interface, messages, the editor, all built-in lessons, the documentation.

## Next

1. **«Start here» rings over the editor window**: the hint target is shown as text for now (`tool:Brush`); this needs the coordinates of PhotoCraft elements from `ui.inspect` and a transparent window on top.
2. **The daily challenge** from the mockup.
3. **Checking the installer on a clean machine.**
4. **The first finished version** in `main`.
