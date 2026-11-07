# Building and running the editor (Windows)

[Русский](../build.md) · **English**

## What to install

| What | Why | How |
|---|---|---|
| Visual Studio 2022 Build Tools, the “Desktop development with C++” component | the MSVC linker for Rust | `winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"` |
| Rust 1.99 or newer (via rustup; FUDE needs 1.99, the editor 1.95) | the compiler | `winget install Rustlang.Rustup`, then `rustup default stable` |
| Git | the sources | `winget install Git.Git` |

Check:

```bash
rustc --version
cargo --version
```

## Disk space

The editor sources are about 55 MB; the `photocraft/target/` build folder grows to 10–15 GB (debug and release builds together). If the project disk is short of space, use another build folder:

```bash
set CARGO_TARGET_DIR=C:\photocraft-target   # cmd
export CARGO_TARGET_DIR=/c/photocraft-target  # Git Bash
```

## Running

```bash
cd photocraft
cargo run --release -p photocraft
```

The first build takes a few minutes. The editor opens in the **FUDE Sumi** theme. To switch themes: **Window › Theme › Dark Theme / Light Theme** or **Edit › Preferences › Interface**.

Useful launch flags:

| Flag | What it does |
|---|---|
| `--safe-gpu` | draw the canvas on the CPU (if the graphics card or driver cannot cope) |
| `--control <port>` | open the JSON control channel on a port, for example `--control 7878` (this is how FUDE drives the editor) |

## The lesson core

FUDE's own code is a separate Cargo workspace at the repository root (`Cargo.toml`); `photocraft/` is not part of it and builds on its own. From the root:

```bash
cargo build -p fude-core                         # the library and the fude-cli CLI
cargo run -p fude-core --bin fude-cli -- help        # CLI commands
cargo run -p fude-core --example lesson_art -- lessons   # redraw the pictures and guides of the built-in lessons
```

The core is small and builds in a minute; the build lives in `target/` at the root.

To try a lesson's guide in the real editor, start PhotoCraft with the control channel and file roots:

```bash
cd photocraft
cargo run --release -p photocraft -- --control 7878 --control-token-file ../target/control.token --automation-read-root ../lessons/light/03-cast-shadow --automation-write-root ../target/attempt
```

Connect to `127.0.0.1:7878`. Send `{"id":"auth","method":"auth","params":{"token":"<the contents of control.token>"}}` as the first line, then the lines that `fude-cli guide lessons/light/03-cast-shadow` prints. The editor answers each line with a line carrying the same `id`.

## A windowless interface snapshot

```bash
cd photocraft
cargo run --release -p photocraft-ui-egui --example snapshot -- --out ui.png --size 1440x900 --scale 1 --safe-gpu
```

For the light theme add `--script '[["ui.set", {"theme": "fudeWashi"}]]'`. To open a picture on the canvas: `--open path/to/picture.png`.

## Fonts

PhotoCraft can optionally be built with the [craft-fonts](https://github.com/storytold/craft-fonts) fonts (Japanese and other interface fonts): clone them alongside and set `CRAFT_FONTS_DIR`. Without them everything works, and the interface uses the bundled Inter and JetBrains Mono.

## If something does not build

- `link.exe not found` — the C++ component of the Build Tools is not installed.
- `rustc 1.xx is not supported` — update Rust: `rustup update stable`.
- Not enough space — set `CARGO_TARGET_DIR` on another disk (see above).
