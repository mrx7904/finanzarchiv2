# #01 The shell on Windows and macOS, and the check

**Goal:** An empty Tauri 2 app with a Rust core and a React UI opens its window on Windows and macOS, and `node scripts/check.mjs` runs every step of the repository.
**Requirements**: FA-01, FA-02, FA-03 · **Depends on**: none

## Done when

- [ ] Rust is installed through rustup and pinned in `rust-toolchain.toml` to the stable version of `decision-stack.md` or a newer one checked on the day, with the date in the commit.
- [ ] `src-tauri/` and `src/` exist, created with the current `create-tauri-app` or by hand, with the exact versions recorded in `decision-stack.md`.
- [ ] The window opens on the owner's Windows PC and on the Mac with a German title, seen by the owner or as a screenshot.
- [ ] The capability file of the window allows nothing beyond what the empty app uses, and the CSP is set.
- [ ] `node scripts/check.mjs` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `npm run typecheck`, `npm run lint` and `npm test` in addition to its current steps, and exits with 1 when any of them fails.
- [ ] `node scripts/check.mjs --self-test` shows each step able to fail, the Rust and TypeScript steps included.
- [ ] Whether `tauri-driver` can drive the app on Windows and macOS is checked at the primary source and the result stands in `decision-stack.md`.

## Out of scope

- The database, belongs to #02
- The look, belongs to #07
- Installers, belong to #08
