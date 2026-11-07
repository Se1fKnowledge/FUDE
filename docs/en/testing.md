# Tests

[Русский](../testing.md) · **English**

The repository has two sets of tests:

- **the `fude-core` lesson core** ([`core/`](../../core/)) and the built-in lessons ([`lessons/`](../../lessons/)) — our own code;
- **the editor** in [`photocraft/`](../../photocraft/) — PhotoCraft's tests plus the tests we added or changed for FUDE.

## The latest run

November 7, 2026, Windows 11, Rust 1.99.1:

| Set | Command | Result |
|---|---|---|
| lesson core and lessons | `cargo test -p fude-core` (from the root) | 112 passed, 0 failed |
| app backend | `cargo test -p fude-app` | 52 passed, 0 failed |
| guides in the real PhotoCraft | `cargo test -p fude-app --test real_editor -- --ignored` | 12 of 12 guides solve their tasks |
| interface | `cd app && npm test` | 67 passed, 0 failed |
| interface types | `cd app && npm run check` | 0 errors, 0 warnings |
| clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | no warnings |
| format | `cargo fmt --all --check` | no differences |
| editor engine | `cargo test -p photocraft-engine --lib` | 2178 passed, 0 failed, 14 skipped |
| editor interface | `cargo test -p photocraft-ui-egui --lib` | 3809 passed, 0 failed, 18 skipped |
| editor clippy | `cargo clippy -p photocraft-engine -p photocraft-ui-egui --lib --tests -- -D warnings` | no warnings |
| editor control channel | `cargo test -p photocraft-ui-egui --lib control::tests::journal_read` | passes |

The FUDE rows (core, backend, interface, clippy, format) are repeated on every push by the GitHub Actions workflow [`ci.yml`](../../.github/workflows/ci.yml), see [app.md](app.md#building-on-github).

The skipped tests (`#[ignore]`) are skipped by PhotoCraft itself too: long performance checks and tests that need external file corpora.

## The lesson core

From the repository root:

```bash
cargo test -p fude-core                       # everything: modules, library, packages, lesson editor, built-in lessons
cargo test -p fude-core --test lessons        # only the built-in lessons
cargo clippy -p fude-core --all-targets -- -D warnings
cargo run -p fude-core --bin fude-cli -- validate lessons
```

The set runs in under two seconds after the build.

### Modules — `core/src/*.rs`

| Module | Tests | What they check |
|---|---|---|
| `paths` | 5 | ordinary relative paths pass; `..`, `./`, absolute paths, drive letters, `\`, double `/`, a colon and control characters are rejected; extensions are case-insensitive |
| `model` | 9 | a lesson reads with defaults and survives the TOML → model → TOML round trip; unknown fields and check kinds are errors with a line number; ranges and their Russian wording (“от 30 до 80”, “не меньше 0,25”); `#rrggbb` colours |
| `validate` | 12 | a model lesson has no remarks; a lesson must lie in its category; errors name the TOML key; exactly one of `open` and `canvas`; unique steps; ranges and areas within bounds; hint targets and tools as in PhotoCraft; ghost line style (colour, width, opacity) within bounds |
| `guide` | 7 | a script reads and survives the JSON round trip; files, saving, menus, shortcuts, unknown commands and `ui.set` fields are rejected; allowed calls pass; pauses are limited |
| `hint` | 9 | ghost shapes land on a separate translucent layer and return the student to their layer; each shape has its own colour, width and opacity; an arrow has its head at the tip; dashing cuts a line into dashes and goes round corners; a “drawn in” line is drawn in ten pieces with pauses and reaches the end, the pieces cover the line without gaps; an ellipse is closed and stays inside its frame; hint layers are found inside groups too |
| `record` | 4 | the author's recording becomes guide steps: a tool — a highlight, a brush — actions, each stroke separately with its brush and pressure, long pauses are shortened, forbidden commands are skipped and named; the colour follows the foreground; a long stroke is thinned to the guide limit keeping its end; undos are not recorded by default, and pauses can be kept as they are |
| `control` | 4 | `engine.execute` calls; a task with a picture opens through the read root; a blank canvas is created and tinted |
| `assess` | 20 | the `ui.inspect` reply is parsed; each state and canvas check passes and fails with a clear message; transparent counts as white; a task picture of another size is scaled; large areas are sampled; a passed step stays passed, steps go in order; **the live check** (`prove`): a check that passes on the untouched task too is told from a working one and gets a threshold between the two readings; a canvas check cannot be proven without a reference |
| `authoring` | 3 | Russian titles are transliterated into ids; repeats are numbered; the new lesson template is valid |
| `i18n` | 2 | `tr!` picks the text by language and fills in values; the language changes for one thread without disturbing tests next to it; the codes `ru`, `en`, `en-US` |
| `locale` | 3 | a translation replaces the texts that have an entry and leaves the rest; it finds translation entries that are not in the lesson; `en.toml` and `theory.en.md` are read from the lesson folder, Russian needs no translation |

### Integration — `core/tests/`

| File | Tests | What they check |
|---|---|---|
| `library.rs` | 6 | the catalog is sorted by order; a broken lesson goes to “problems” instead of breaking the catalog; own lessons join built-in categories but cannot replace a built-in lesson; dependencies unlock lessons; hidden folders are skipped |
| `package.rs` | 12 | export and import bring back lessons with videos and pictures, without `.exe` and junk; a repeated import needs replace; built-in lessons are not replaced; **zip-slip** (`../`, absolute paths, `C:`, `\`), links, foreign file types, duplicates, too many files and too large sizes are rejected entirely, leaving nothing on disk |
| `authoring.rs` | 6 | a new category and lesson show up in the library at once; a draft with errors is saved but marked; theory, guide, video and pictures are saved into the lesson without overwriting; a built-in lesson is edited as a copy; deleting is allowed only inside own lessons |
| `lessons.rs` | 10 | the built-in lessons have no errors or warnings; two categories of six lessons, difficulty grows; lessons unlock in a chain from the first; each has theory with sensei's advice, questions, a guide of at least 5 seconds, hints and checks on every step; all six skill areas are trained; **canvas checks pass on the reference and fail on the untouched task**; hints are drawn with allowed commands; categories pack into packages; **every lesson and category is fully translated into English** (`en.toml` with nothing missing and nothing extra, `theory.en.md`), and in the English interface the library gives lessons without Russian text; tool names in check messages are in the interface language |

## The app

### Backend — `app/src-tauri/`

| Module | Tests | What they check |
|---|---|---|
| `lang` | 3 | journal entries written in one language are re-worded in the other, unknown ones are left alone; names in quotes and whole known strings (lessons, achievements) are translated; the language is kept in the database |
| `db` | 11 | an empty first-day profile, a daily goal of 1 to 10; time there and back, garbage rejected; attempts, steps (each once and only in its own attempt), hints, outcome; an unfinished attempt without an outcome; attempts ordered by start; the journal newest first with a limit; achievements; lesson stages, removing a missing stage; the database file survives a restart |
| `progress` | 10 | experience for difficulty, independence and repeats; ranks 10 kyu → 1 dan; streaks from today or yesterday; days, minutes, days without hints; skills and their value at the start of a period; independence; experience by week; lesson locks; achievements from facts; a greeting by the hour |
| `editor` | 12 | the channel client passes `auth`, matches replies by `id` and skips others'; a wrong token is rejected; tokens are random; a clear error if there is no editor; the editor is looked for next to `fude.exe` and in the `photocraft/` folder; the channel line from stderr gives the port or the reason of a refusal; **starting with a fake editor** (the test binary restarted in the role of PhotoCraft): connecting to the port the editor reported, and calls; a refusal to open the channel — at once, without waiting; exiting at start; a silent editor — by timeout; a dropped connection (PhotoCraft closes an idle one) is reopened once, while the editor's own refusals are not retried |
| `author` | 3 | guide recording: a tool change and new journal entries are read once, the start mark goes after what was already recorded; the brush and colour come from `ui.inspect`; an old editor without `journal.read` is named clearly |
| `practice` | 2 | an attempt against a fake editor: a check, a hint (counted once, the hint layer and the return to the student's layer), removing the hint layer, the outcome with experience and similarity, the journal; manual steps and no extra snapshots |
| `learn` | 3 | the Russian date; sensei's advice points to a lesson with hints or to a weak skill; the first day |
| `state` | 3 | lesson pictures become `data:` addresses, including from subfolders, paths do not leave the lesson folder; types by extension, case-insensitive |
| `window` | 5 | tiny rectangles for the editor are not used; a process without windows has no main window; helper windows (tool windows, transparent, non-activatable — like the winit window) are not taken for the editor; a press of any mouse or pen button gives the editor the keyboard; F1 and focus are safe without a window |
| `tests/real_editor.rs` | 1 (`--ignored`) | starts the real PhotoCraft and goes through each built-in lesson with its own guide, checking after every step |

### Interface — `app/src/lib/*.test.ts`

| File | Tests | What they check |
|---|---|---|
| `format.test.ts` | 16 | Russian plurals (1 урок, 2 урока, 11 уроков, 21 урок…), durations as in the mockup, clock time, percents, thousands, dates without a time-zone shift |
| `ink.test.ts` | 6 | all ink pictures are in place: strokes, ensō, mountains, small masks; a seed picks a picture repeatably; each day has its own brush tap; the mountain window covers the block, different seeds show different hills, a wide block stretches the picture |
| `checks.test.ts` | 4 | each check kind has a valid initial value; areas do not share one array; Russian descriptions; new step numbers do not collide |
| `shortcuts.test.ts` | 4 | the hint is F1, not H (H is “Hand” in PhotoCraft) and not its Russian layout key; F1 with modifiers and auto-repeat do not count; the check is Ctrl+Enter and Cmd+Enter; keys do not fire in input fields |
| `editor-slot.test.ts` | 5 | by default the editor is in the FUDE window, the choice is remembered and survives blocked storage; the rectangle in physical pixels with rounding; the editor place is sent once, then only on change, and cleared when leaving the page; a backend refusal does not break the page |
| `studio/studio.test.ts` | 13 | ghost lines: simplifying a freehand line, shapes from dragging, areas within the canvas, the default style and saving only differences, hit testing and moving, the arrowhead, a guide stroke → a ghost line; guide tracks: step times, captions until the next one, stroke and action clips, the frame and caption at the playhead, edits (pause, order, insert, delete), Russian names of commands and targets, parsing `guide.json`, clock time; the interface map: searching targets by name and id |
| `studio/studio2.test.ts` | 7 | edit history: undo and redo, a new edit dropping redo, the history limit; shape handles: the ends of a straight line and an arrow, the corners of an oval from the opposite corner, a freehand line has no handles; “drawn in” in the style; the task steps track: neighbouring guide steps merge into a clip, the start time of a task step; splitting: a stroke splits in two with a shared point, another step into actions and a pause, a too-short one is not split |
| `i18n.test.ts` | 7 | every `t(…)` and `tk(…)` string has an English entry with the same placeholders; every word in `count()` has English forms; the dictionary has no extra entries; no Russian text outside `t()`; placeholders and the Russian fallback |
| `api.test.ts` | 2 | the error text from a Rust string, from an `Error` or a fallback; commands are called by their Rust names |

### Manual checks

On October 26, 2026 the app was run in full (`npm run tauri dev`) with the real PhotoCraft. The whole path was gone through: welcome → home → catalog → the “Canvas and zoom” lesson → the guide in the editor (step captions in sync) → practice (the auto-check “Hand is selected now”, the hint drew the “FUDE · hint” layer) → the result with experience and the “First stroke” achievement → achievements, activity, statistics → the workshop: a category, a lesson, a check area with the mouse, saving to `lesson.toml`. Found and fixed: an invisible new achievement on the result, a `structuredClone` error in the lesson editor, overlapping month labels. Not checked by hand: import and export through the system file dialog (the logic is covered by core tests) and video upload.

Editor embedding was checked there too, with window screenshots: practice with PhotoCraft inside the FUDE window and the task panel; the “Task” button hides the panel and the editor takes the full width; “Separate window” makes PhotoCraft its own window again in its former place; “Embed in FUDE window” brings it back; the lesson guide draws right in the lesson page player. The “PhotoCraft updates” window is gone. A shortcut to `app/run-dev.cmd` opens FUDE in about 10 seconds, and closing the window stops the console too.

On November 5, 2026 the embedding fixes were checked in the real app (with `FUDE_DATA_DIR`, so as not to touch real progress): the real editor window is embedded, not winit's helper window; after a click on the canvas the editor has focus, and the Z key switches on “Zoom” — the practice step counted; F1 with focus in the editor shows the hint, H no longer adds a hint; “To practice” after the guide opens the task in the same editor without a port error; going home closes the editor, a new practice starts it again. Installers with `photocraft.exe` inside were built locally (`tauri.bundle.conf.json`).

On November 7, 2026 the English interface was checked in the running app: home, catalog, a lesson with English theory, guide captions and steps, activity with a journal re-worded into English, and the switch back to Russian.

## The editor

From the `photocraft/` folder:

```bash
# everything FUDE touches: themes, settings, menus, brand
cargo test -p photocraft-ui-egui --lib theme::
cargo test -p photocraft-ui-egui --lib prefs_ui
cargo test -p photocraft-ui-egui --lib links
cargo test -p photocraft-engine --lib prefs
cargo test -p photocraft-engine --lib shell_commands

# the full sets
cargo test -p photocraft-engine --lib
cargo test -p photocraft-ui-egui --lib

# before a commit
cargo fmt --all
cargo clippy -p photocraft-engine -p photocraft-ui-egui --lib --tests -- -D warnings
```

The full interface set runs in about 45 seconds (after the build), the engine in about 15.

## FUDE's tests in the editor

### Themes — `crates/ui-egui/src/theme.rs`

| Test | What it checks |
|---|---|
| `fude_themes_are_the_default_parse_and_hold_text_contrast` | FUDE Sumi is the default theme; both themes are found by name (“FUDE Ink”, `fude_ink`, “sumi”, “washi”…); built on the Pro layout; main, secondary and faint text hold 4.5:1 on all surfaces, including hover, pressed and the selected row; status colours — 4.5:1 on calm surfaces and 3:1 on active ones; white text on the indigo selection and on the red button — 4.5:1 |
| `fude_tokens_are_the_fude_design_system_colours` | the theme colours match the Figma tokens (`sumi/*`, `paper/*`, `ink/*`, `ai/*`, `shu/500`) to the byte |
| `fude_surfaces_step_in_depth_and_keep_their_tint` | surfaces go by brightness in the right order (canvas → dock → panels → hover → pressed); ink leans blue, paper leans warm, none is neutral grey; corner radii 4 / 6 / 8 |
| `applying_a_fude_theme_installs_its_palette_into_egui` | after `apply`, standard egui widgets draw with the FUDE palette: panel, window and field backgrounds, text, links, errors, warnings, selection, window corners, the Pro font size |
| `theme_ids_and_labels_are_unique_and_round_trip` | all 16 themes have unique ids and labels, each id is found back; each theme is in exactly one family (dark or light); cycling goes through both FUDE themes |
| `field_selection_is_grey_only_for_the_monochrome_pro_themes` (original, still passes) | a grey field selection only for the monochrome Pro themes; FUDE's is indigo |

### Settings — `crates/engine/src/prefs/tests.rs`

| Test | What it checks |
|---|---|
| `appearance_defaults_and_legacy_theme_migrate` (updated) | a new install: dark mode, FUDE Sumi and FUDE Washi; an old file with one theme moves into its mode |
| `fude_themes_are_choices_and_a_legacy_fude_theme_selects_its_family` | `fudeInk` is first in the dark themes list, `fudeWashi` is in the light themes list; an old file with a FUDE theme opens in its mode |
| `setting_the_legacy_theme_to_a_fude_theme_fixes_its_mode` | `prefs.set interface.theme` with a FUDE theme switches on its mode and leaves the choice in the other family alone; a light theme cannot be written into the dark slot and vice versa |

### Menu commands — `crates/engine/src/shell_commands.rs`

| Test | What it checks |
|---|---|
| `fude_theme_commands_lead_their_theme_submenus` | the `window.theme.fudeInk` and `window.theme.fudeWashi` commands exist, have the right labels, come first in the “Dark Theme” and “Light Theme” submenus and have no shortcuts |
| `no_command_label_carries_the_artcraft_brand` | no command label has ArtCraft or Discord; the removed “Help” items are gone |

### The settings window and switching — `crates/ui-egui/src/prefs_ui.rs`

| Test | What it checks |
|---|---|
| `fude_theme_menu_commands_select_and_save_their_slot` | a new install opens in FUDE Sumi; Window › Theme › FUDE Washi and FUDE Sumi show the theme, switch on its mode and save it, leaving the other family alone |
| `theme_cards_offer_fude_first_and_every_option_is_a_saved_theme` | FUDE comes first in the theme cards; each option is a saveable theme of the same family; favourite FUDE themes go into the appearance button menu |
| `appearance_resolves_both_saved_themes_and_follows_system`, `auto_uses_native_system_theme_when_egui_does_not_report_one`, `darken_and_lighten_interface_…`, `auto_appearance_replaces_a_restored_egui_dark_pin` (updated) | the "as in the system" mode picks FUDE Sumi or FUDE Washi by default; “Darken/lighten interface” starts from a FUDE theme |
| `auto_appearance_follows_live_changes_without_saving_resolved_palettes` (original) | caught a real bug: the initial interface theme was hard-coded ProMedium; fixed in `state.rs` |

### Brand and links — `crates/ui-egui/src/links.rs`

| Test | What it checks |
|---|---|
| `help_menu_lists_links_then_separator_then_system_info_and_about` (updated) | the “Help” menu: FUDE on GitHub, Report an issue, a separator, System info, About — without an update check |
| `link_commands_open_their_urls` (updated) | links lead to the FUDE repository and Issues |
| `control::tests::host_embedding_hides_the_window_controls` | `ui.set {"hostEmbedded": true}` removes the window buttons and their space, `false` brings them back; a non-boolean value is rejected |
| `links_open_through_the_platform_service` (updated) | links open through the system browser |
| `no_link_or_help_command_carries_the_artcraft_brand` | no link leads to ArtCraft or Discord; the removed items are unknown commands |
| `catalogs_and_repository_carry_no_artcraft_brand` | none of the 21 translation catalogs has ArtCraft strings or links to the ArtCraft Discord; there is no `docs/brand` logo folder |
| `every_link_goes_to_the_fude_repository` | all editor links lead to the FUDE repository, none to PhotoCraft |
| `no_runtime_link_to_the_upstream_repository` | no source file of the editor's interface, engine or app has a link to `github.com/storytold` or `api.github.com` (comments do not count) |

### Other tests that had to be updated

| Test | Why |
|---|---|
| `canvas::tests::start_screen_buttons_stay_inside_a_narrow_canvas` | the Discord button is gone; the test checks the PhotoCraft on GitHub link and that Discord does not appear |
| `status_bar_tests::appearance_toggle_preserves_pro_layout_and_has_no_fit_button` | the default light theme is now FUDE Washi |
| panel docking tests (`panel_docking_tests.rs`) | the bench starts in the default theme, like the app itself |

## Design review

The Figma mockup was checked with the impeccable method: an independent design review and an automatic measurement of layers (WCAG contrast, text smaller than 12 px, clipping, overlaps, clickable area sizes, glows). After the fixes, contrast and small text are clean; the remaining marks are intentional (watermarks, faint signs of locked achievements, a dimmed non-working editor on the error screen).
