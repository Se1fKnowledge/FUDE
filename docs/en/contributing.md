# How to contribute

[Русский](../contributing.md) · **English**

## Branches

| Branch | What is in it |
|---|---|
| `main` | finished versions only; we never commit to it directly |
| `SelfKnowledge` | development; all changes go here |

A finished version reaches `main` by merging `SelfKnowledge` (preferably through a pull request on GitHub), after all the tests in [testing.md](testing.md) pass. We mark a version with a tag, for example `v0.1.0`, and record it in [CHANGELOG.md](../../CHANGELOG.md).

## Commits

- One commit — one logical change: “theme”, “brand removal”, “tests”, “documentation”.
- Changes in `photocraft/` stay separate from the rest of the project: that makes them easier to carry over to a new PhotoCraft version (see [editor.md](editor.md#how-to-update-photocraft-from-the-original-repository)).
- Commit messages are in Russian; the first line briefly answers "what was done".

## Two languages

- Every interface string goes through `t('…')` with an English entry in `app/src/lib/i18n/en.ts`; `npm test` fails otherwise.
- Backend and core messages use `fude_core::tr!("ru", "en")`.
- A new built-in lesson needs an `en.toml` and a `theory.en.md` (see [lesson-format.md](lesson-format.md#languages)); `cargo test -p fude-core` checks it.
- A document changed in `docs/` is changed in `docs/en/` too.

## Rules for the editor code

`photocraft/` follows the original project's rules ([`photocraft/AGENTS.md`](../../photocraft/AGENTS.md)); the main ones:

- outside tests there is no `unwrap()`, `expect()` or `panic!`: errors are returned through `Result`;
- colours and corner radii come only from `theme::Tokens`, never hard-coded;
- every change comes with a test;
- interface changes are checked with a snapshot (`--example snapshot`);
- before a commit — `cargo fmt --all` and the tests of the affected crates.

## Design

- The source of truth is the Figma mockup ([design-system.md](design-system.md)). If a colour changes in Figma, the editor theme changes too, and vice versa: the `fude_tokens_are_the_fude_design_system_colours` test checks it.
- Text must pass WCAG AA; tests check that too.
