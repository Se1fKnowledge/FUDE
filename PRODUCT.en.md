# Product

[Русский](PRODUCT.md) · **English**

## Platform

web

A desktop app on Tauri: the lessons interface renders in a webview, so the design language is web-like rather than native to one OS.

## Stack

Tauri + Rust (the user's choice). The interface in the webview is SvelteKit + TypeScript. Progress is stored in a local SQLite database. Lessons lie in folders on disk and travel as `.fude` packages (format — [docs/en/lesson-format.md](docs/en/lesson-format.md)).

The editor is the open-source PhotoCraft project (https://github.com/storytold/photocraft): Rust, egui, rendering through wgpu, MIT/Apache-2.0 licence. Every editor action is a command that can be driven through a JSON channel or MCP. Through that channel the app checks task steps, highlights tools and plays guides.

The app works entirely locally: there are no accounts, no server and no sync.

## Users

Two groups on one learning path:
- beginners in digital drawing who have never worked in Photoshop-like editors;
- people who draw traditionally but do not know the editor and want to bring their skill into digital.

The path starts with the interface basics and goes up to advanced techniques. The interface, the lessons and the documentation are in Russian and English (the user's decision); the language is switched next to the profile and chosen on the welcome screen. The Figma mockup is in Russian.

## Product purpose

To teach drawing in a graphics editor, with Photoshop as the model. The student picks a lesson, watches an animated guide, reads the theory and does the task right in the built-in PhotoCraft editor. The app tracks activity and the growth of separate skill areas.

The project is a diploma work (4th year). Success: a working app that takes the student from a lesson to a finished task and shows progress beautifully.

## Positioning

Learning happens inside a real editor, not on a video next to it. The task is done in PhotoCraft, and the app sees every action through the editor's command channel. So a hint can show where to click and draw a ghost shape right on the canvas. Using hints is honestly counted in the statistics.

## Operating context

- The student opens the app on a computer and picks a category and a lesson.
- A lesson: animated video guide → theory → the editor with the task → result.
- Two kinds of video guides (the user wants both):
  - an animated guide: a script of PhotoCraft commands the app plays in the real editor (visualisation + theory);
  - an ordinary video: a file the lesson author uploads into the lesson.
  A lesson can have one of them or both.
- Input: a mouse, possibly a graphics tablet (PhotoCraft supports pen pressure and tilt).
- Screens: people with one small screen work in one window (the editor inside FUDE, the task slides out on the side); people with two monitors, or who like separate windows, move the editor into its own window and keep FUDE with the guide and task beside it (the user's decision).
- The editor has no ties to the original PhotoCraft repository: it does not check for updates or lead to its pages (the user's decision).

## Capabilities and constraints

The required minimum from the original request:
- Lesson categories, each with several lessons on its topic; each lesson's difficulty is visible.
- A home page.
- An achievements tab.
- An activity tab by day, like GitHub's.
- A statistics tab with the growth of different skill areas.
- A lesson = an animated video guide + theory, after which the editor opens with the task.
- Hint mode: counted in the statistics, shows where to click, sometimes draws ghost shapes on the canvas to follow.
- A beautiful opening animation.

Added by the user later:
- Own lessons and categories: an editor for creating and changing lessons right in the app.
- Importing ready lessons from a file (a `.fude` package) and exporting your own.
- Uploading an ordinary video into a lesson, not only an animated guide.
- The lesson studio: the task canvas, ghost lines, the guide on timeline tracks, recording a guide from the editor, the live check.
- The dark «Sumi» theme.
- Ink in the interface from real brush strokes, not drawn in code.
- Two languages: Russian and English, in the whole app, the lessons and the documentation.

Built-in content: two categories of six lessons — «Interface basics» and «Light and shadow».

Suggested in the mockup, not yet confirmed by the user: kyu/dan ranks, the «Independence» measure (hints lower it), task auto-check with comparison to a reference, the «Daily challenge», «Sensei's advice», the lesson result screen.

Decided: local storage only (SQLite), SvelteKit + TypeScript, lessons are folders with `lesson.toml`, guides are scripts of PhotoCraft commands plus an uploaded video.

## Brand commitments

The working name in the mockup is FUDE (筆, «brush»); the user has not approved it.

Visual requirements the user set:
- first a Figma mockup, then the app;
- "very beautiful", the app should give aesthetic pleasure;
- a very beautiful and pleasant opening animation;
- the style of Japanese ink brush strokes;
- colours: bluish, red, orange, white, black.

## Evidence on hand

- The Figma mockup: file `dHDrUBG4xsJ8LL2saJHFxs` (https://www.figma.com/design/dHDrUBG4xsJ8LL2saJHFxs). Pages: cover, design system, the screens, the opening animation on keyframes, the lesson studio, the dark theme.
- The splash video: `design/fude-splash.mp4`.
- The built-in lessons in the `lessons/` folder (theory, tasks, guide scripts) in Russian and English. There are no videos, users, reviews or metrics yet. They must not be invented in the interface as facts.

## Product principles

1. We learn by hand in a real editor: each lesson ends with practice in PhotoCraft, not watching.
2. A hint helps but does not hide: its use shows in the statistics, and independence is valued higher.
3. Progress must be visible and split by skill area, not reduced to one number.
4. Show first, then explain, then let them do it: guide → theory → task.
5. Beauty is part of the product: the learning process should be pleasant in itself.
