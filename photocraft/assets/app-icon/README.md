# Editor app icon (FUDE build)

The FUDE build of PhotoCraft shows the **FUDE editor icon** instead of PhotoCraft's own: the
character 筆 ("brush", read *fude*) written in the Yuji Syuku brush face, washi white on an indigo
tile, with a small vermilion seal in the corner. It matches the FUDE app's seal icon (the same
character and face) while staying recognisably the editor next to it in the taskbar.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| Indigo (FUDE `ai/700` → `ai/900`) | `#1F4470` → `#1B3A60` | the field |
| Washi (FUDE `paper/base`) | `#F3EEE4` | the character, the seal's frame |
| Vermilion (FUDE `shu/500`) | `#BC3823` | the seal |

## Geometry

A 512-unit tile (`viewBox="0 0 512 512"`), rounded square with `rx=112` as a clip path. The
character takes 74% of the tile; the seal is 15% of the tile at the bottom right and is left out
below 32 px, where it would only be a speck. The macOS render sits on Apple's 824/1024 grid.

## Provenance

Made for FUDE (`https://github.com/Se1fKnowledge/FUDE`). The character's outline comes from Yuji
Syuku by Kinuta Font Factory (SIL Open Font License 1.1). License: see `LICENSE.txt`.

## Files

- `photocraft.svg`: canonical master (the character as a vector path).
- `photocraft-small.svg`: the same drawing; also the hicolor scalable icon.
- `photocraft-1024.png`: 1024 px render on the macOS grid.
- `photocraft.icns`: macOS bundle icon (`CFBundleIconFile`).
- `photocraft.ico`: Windows icon, 16–256 px, embedded in the `.exe` by `apps/photocraft/build.rs`.
- `hicolor/<size>/apps/ai.storyteller.photocraft.png` (16–512) and `hicolor/scalable/...svg`:
  Linux icon theme.

The file names stay PhotoCraft's, so the build scripts and packaging find them unchanged.

The app also sets the window icon and Wayland app ID at runtime (`apps/photocraft/src/app_icon.rs`,
`main.rs`). On Windows the taskbar shows the icon of the Start Menu shortcut that launches the
exe, so the MSI's `<Icon Id>` keeps the `.exe` extension (ICE50); `app_icon.rs` documents the
details and tests the `.ico` sizes and the WiX icon references.

## Regenerate

Replace `photocraft.svg` (and `photocraft-small.svg`), then run `packaging/icons.sh`. It needs
`resvg`, plus `iconutil` on macOS; the `.ico` is packed by `cargo xtask ico`.
