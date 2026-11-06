# The editor: PhotoCraft in the FUDE style

[Русский](../editor.md) · **English**

The [`photocraft/`](../../photocraft/) folder is a copy of [storytold/photocraft](https://github.com/storytold/photocraft) at commit `c65b5d2`, with FUDE's changes on top. PhotoCraft is an open-source image editor in Rust (egui + wgpu) that follows Photoshop: layers, masks, adjustment layers, styles, text, vectors, brushes and PSD. The licence is MIT OR Apache-2.0.

The git history is split: first a commit with untouched PhotoCraft, then separate FUDE commits. So `git log -- photocraft` and a `git diff` from the first commit show exactly what we changed.

## What changed

### 1. The FUDE Sumi and FUDE Washi themes

In PhotoCraft every interface colour comes from one set of tokens — [`photocraft/crates/ui-egui/src/theme.rs`](../../photocraft/crates/ui-egui/src/theme.rs). We did not repaint other themes; we added two of our own and made them the defaults:

| Theme | Identifier | Family | Based on |
|---|---|---|---|
| FUDE Sumi (`FUDE Ink`) | `fudeInk` | dark, default | `sumi/*` tokens |
| FUDE Washi (`FUDE Washi`) | `fudeWashi` | light, default | `paper/*` tokens |

Both are built on the **Pro** layout (as in Photoshop: flat panels, tabs), so only colours and corner radii changed (4 / 6 / 8 px instead of 3 / 4 / 6). PhotoCraft's other 14 themes stay in the menu.

How theme fields map to FUDE tokens:

| `Tokens` field | FUDE Sumi | FUDE Washi |
|---|---|---|
| `canvas` (around the document) | `sumi/bg` `#111216` | `#DED6C6` (the darkest paper) |
| `chrome`, `card` (panels) | `sumi/panel` `#1A1C21` | `paper/base` `#F3EEE4`, `paper/raised` `#FBF8F2` |
| `dock` | `#15161A` | `paper/sunken` `#E9E2D4` |
| `hover` / `pressed` | `sumi/raised` `#23262D` / `#2C3038` | `#ECE6DA` / `#E2DACC` |
| `text` | `sumi/text` `#E8E4DC` | `ink/900` `#16161A` |
| `accent` (selection) | indigo `#3F6E9F` | `ai/500` `#2F5D8C` |
| `row_selected` | `#29384F` | `ai/100` `#DCE7F1` |
| `primary_bg` (main button) | `shu/500` `#BC3823` | `shu/500` `#BC3823` |
| `warning` | persimmon `#EC944A` | `#844814` |
| `caption_close` (close the window) | `shu/500` | `shu/500` |

The theme is wired in everywhere PhotoCraft lists themes:

- engine settings: `Theme`, `DarkTheme`, `LightTheme` and the defaults ([`crates/engine/src/prefs.rs`](../../photocraft/crates/engine/src/prefs.rs));
- migration of old settings and `prefs.set interface.theme`;
- theme cards in the settings window ([`crates/ui-egui/src/prefs_ui.rs`](../../photocraft/crates/ui-egui/src/prefs_ui.rs));
- the `window.theme.fudeInk` and `window.theme.fudeWashi` menu commands ([`crates/engine/src/shell_commands.rs`](../../photocraft/crates/engine/src/shell_commands.rs), [`crates/ui-egui/src/menus.rs`](../../photocraft/crates/ui-egui/src/menus.rs));
- panel docking and the control channel, which enabled the Pro layout only for `Pro | ProMedium` ([`panel_docking.rs`](../../photocraft/crates/ui-egui/src/panel_docking.rs), [`control.rs`](../../photocraft/crates/ui-egui/src/control.rs));
- the initial interface theme ([`state.rs`](../../photocraft/crates/ui-egui/src/state.rs)) and the web build ([`apps/photocraft-web/src/web.rs`](../../photocraft/apps/photocraft-web/src/web.rs)).

Two fixes along the way are worth knowing:

- **The initial interface theme** was hard-coded as ProMedium. With a different default theme the screen and the state disagreed: a saved ProMedium theme might not apply at start. Now the initial theme comes from `ThemeKind::default()`.
- **Panel docking** detected the Pro layout from the list `Pro | ProMedium`, while drawing used a token flag. For new themes this broke tab dragging. The FUDE themes were added to those checks; the original themes behave as before.

### 2. ArtCraft trademarks

PhotoCraft's brand terms require forks and modified versions to remove the ArtCraft name, logos and links. Done:

- the `docs/brand/` logos are removed;
- the «Help» menu lost «Join the ArtCraft Discord…», «ArtCraft Website» and «PhotoCraft Website» (it led to getartcraft.com); «Report an issue» leads to [FUDE Issues](https://github.com/Se1fKnowledge/FUDE/issues), «PhotoCraft on GitHub» to the source code;
- the Discord button is gone from the start screen and the «About» window; the «PhotoCraft on GitHub · FUDE» links remain;
- unused ArtCraft and Discord strings are removed from all 21 translation catalogs;
- in the Windows (`.wxs`) and Linux (`nfpm.yaml`, `metainfo.xml.in`) installers the addresses point to the FUDE repository;
- the editor README, user documentation, `ATTRIBUTION.md`, `NOTICE` and the book are updated.

Kept on purpose:

- the copyright line «ArtCraft Team and the PhotoCraft contributors» in `LICENSE-MIT`, `NOTICE` and the README — the MIT and Apache licences require it;
- credits and the description of origin (the icon "drawn in ArtCraft", the list of authors) — that is attribution, not use of the mark;
- PhotoCraft's internal developer instructions (`AGENTS.md`, `docs/user/AGENTS.md`) — they describe the original project's processes.

Tests make sure the brand does not come back (see [testing.md](testing.md)).

### 3. No ties to the PhotoCraft repository

The editor in FUDE downloads nothing and opens no pages of the original project:

- **The update check is removed entirely.** At start the editor used to contact `api.github.com` and show a «PhotoCraft updates» window with a link to the download page. Removed: the network module (`apps/photocraft/src/updates.rs` and the `ureq` dependency), the engine commands (`updates_cmds.rs`, `help.checkForUpdates`), the window (`updates_ui.rs`), the «Check for updates automatically» setting and the `updates` / `updatesOpen` fields of the control channel. The FUDE app will update the editor itself.
- **Links lead only to FUDE.** «PhotoCraft on GitHub» in the «Help» menu and on the start screen became «FUDE on GitHub» (`help.fude`); «Report an issue» leads to FUDE Issues.
- **PhotoCraft's authors in the «About» window are listed as text**, without links to their profiles.

What remains and why: copyright lines and licence files (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, `ATTRIBUTION.md`) — the MIT and Apache-2.0 licences require them and they cannot be removed. Mentions of the original repository's issues in code comments are development history, not links in the program.

Tests: `no_runtime_link_to_the_upstream_repository` checks that no interface, engine or app source file has a link to `github.com/storytold` or `api.github.com` (outside comments); `every_link_goes_to_the_fude_repository` — that all menu links lead to FUDE; the «Help» menu and command tests check that `help.github` and `help.checkForUpdates` no longer exist.

### 4. The "inside the FUDE window" mode

On Windows FUDE embeds the editor window in its own (see [app.md](app.md#the-editor-in-the-fude-window-or-on-its-own)). So the embedded editor stays out of the way, the control channel got a `ui.set {"hostEmbedded": bool}` field ([`control.rs`](../../photocraft/crates/ui-egui/src/control.rs)). While it is `true`:

- the «minimise / maximise / close» window buttons and their space in the top bar are not drawn ([`titlebar.rs`](../../photocraft/crates/ui-egui/src/titlebar.rs), [`panels.rs`](../../photocraft/crates/ui-egui/src/panels.rs));
- the top bar does not drag or maximise the window, and there are no edge resize zones ([`lib.rs`](../../photocraft/crates/ui-egui/src/lib.rs));
- `ui.inspect` reports `hostEmbedded`.

FUDE sets `true` when it embeds the window and `false` when it detaches it. Test: `host_embedding_hides_the_window_controls`.

### 5. Reading the command journal to record a guide

The FUDE lesson studio records a guide while the author draws in the real editor. For that the control channel got a `journal.read {from, limit}` method ([`control.rs`](../../photocraft/crates/ui-egui/src/control.rs), described in [`control-protocol.md`](../../photocraft/docs/control-protocol.md)). It returns the entries of the journal of executed engine commands starting at `from` (strokes with points and pressure, brush settings, layers, selections) and the number to read on from. PhotoCraft had the journal before — operation recording uses it; the new method only lets you read it. FUDE takes the active tool from `ui.inspect`.

Test: `control::tests::journal_read_returns_executed_commands_from_a_mark`.

### 6. The FUDE editor icon

The PhotoCraft logo (a kitsune) is replaced with a FUDE icon: the kanji 筆 in the Yuji Syuku brush font, white on an indigo tile, with a small vermilion seal — the same character and font as the FUDE app's seal, but indigo to tell the editor apart on the taskbar. The icon is in the window title, the taskbar, the `.exe`, `.ico`, `.icns` and the hicolor set ([`assets/app-icon/`](../../photocraft/assets/app-icon/README.md)); the file names are unchanged, so building and packaging did not change. The character outline comes from Yuji Syuku (SIL OFL 1.1); the icon is under the PhotoCraft licence (MIT OR Apache-2.0).

Tests: `brand::tests::*` (a 128 px tile with rounded corners, one texture), `app_icon::tests::*` (the `.ico` sizes), importing `photocraft.svg` in `svg::tests`.

### 7. The interface language follows FUDE

PhotoCraft is translated into 22 languages and keeps the choice in the `interface.language` setting. Every time FUDE starts the editor and every time its own language changes, it sends `prefs.set {"path": "interface.language", "value": "ru" | "en"}` through the control channel, so the editor's menus and panels are always in FUDE's language. The editor's code did not need to change for this.

## How to update PhotoCraft from the original repository

FUDE's changes are separate commits, so they can be carried over to a new version:

```bash
# see what we changed
git diff <import commit>..HEAD -- photocraft

# save our changes as a patch
git diff <import commit>..HEAD -- photocraft > fude.patch

# replace photocraft/ with a fresh storytold/photocraft (without its .git),
# commit «Импорт PhotoCraft <version>», then
git apply --3way fude.patch
```

After an update — a full test run (see [testing.md](testing.md)): the theme and brand tests show at once if the new version has new places that list themes, or if ArtCraft links came back.

## What to know about PhotoCraft

- **The project's rules** are in [`photocraft/AGENTS.md`](../../photocraft/AGENTS.md): no `unwrap` or `panic` outside tests, every action is a command, colours only from `theme::Tokens`, interface changes are checked with a snapshot.
- The original project's rules say plainly that PhotoCraft must not be wrapped in Tauri or a webview: it is a native app. So in FUDE the editor runs as **a separate process**, and the lessons app drives it through the JSON command channel (see [architecture.md](architecture.md)).
- Windowless interface snapshots: `cargo run --release -p photocraft-ui-egui --example snapshot -- --out ui.png`. The screenshots in [`design/`](../../design/) were taken this way, with the `--safe-gpu` flag: the Windows software video adapter the snapshot draws on cannot compile one shader of the GPU compositor, so the canvas is drawn on the CPU.
