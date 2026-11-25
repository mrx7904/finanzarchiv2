# Decision: a Rust core in Tauri 2, the UI in React

**Decided:** 2026-11-25 by the owner (Tauri 2 with React, chosen over Leptos or Dioxus in the webview and over a native UI with Slint or egui) · applies to the whole repository

## Decision

- The app is a Tauri 2 desktop app. The core (`src-tauri/`) is Rust and holds every rule: database, documents, reading, checks, matching, money. The UI (`src/`) is React with TypeScript and Tailwind and only shows and asks.
- Windows and macOS first, Linux later. Tauri supports all three from one code base.
- The local model stays Ollama, reached through one Rust trait `LocalModel`, so an embedded runtime can replace it later without touching the readers.

## Why

- The owner wants Rust and the three systems. Tauri uses the webview of the system (WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux) instead of shipping Chromium like Electron.
- The six UI sources the owner chose (`decision-ui-libraries.md`) are React and Tailwind. A Rust UI in WebAssembly could take their ideas only, never their code.
- Tauri's capabilities deny every command and API a window was not given. In 1.0 the same protection was written by hand in `hardening.ts`.
- Rust gives real threads for the PDF work, which ran on one Node process in 1.0.
- `tauri-specta` generates the TypeScript bindings from the Rust commands. In 1.0 each of the 252 IPC channels was written three times (declaration, preload, zod schema).

## Versions

Checked on 2026-11-25 at crates.io (`/api/v1/crates/<name>`), registry.npmjs.org (`/<name>/latest`) and static.rust-lang.org (`channel-rust-stable.toml`). #01 pins the exact versions it installs.

| Part | Choice | Version |
|---|---|---|
| Toolchain | Rust stable | 1.100.0 (2026-11-09) |
| Shell | `tauri`, `tauri-build`, `@tauri-apps/cli`, `@tauri-apps/api` | 2.12.7, 2.7.6, 2.12.6, 2.12.5 |
| Plugins | `tauri-plugin-dialog`, `-single-instance`, `-opener`, `-log`, `-updater` | 2.8.4, 2.5.5, 2.7.3, 2.10.2, 2.14.1 |
| Typed boundary | `tauri-specta`, `specta`, `specta-typescript` | 2.0.0-rc.29, 2.0.0-rc.28, 0.0.15 (see "The typed boundary of #03") |
| Database | `rusqlite` with the feature `bundled` | 0.43.0 |
| Async runtime | `tokio` | 1.54.1 |
| Errors, logs | `thiserror`, `tracing` | 2.0.21, 0.1.44 |
| Hash | `sha2` | 0.11.2 |
| PDF | `pdfium-render` (PDFium, the engine of Chrome) | 0.9.6 |
| Folder watching | `notify` | 9.0.0 |
| Model client | `ollama-rs` or plain `reqwest` | 0.3.6, 0.13.5 |
| Mail (stage 4) | `async-imap`, `mail-parser`, `oauth2` | 0.12.0, 1.0.2, 5.0.0 |
| Credentials | `keyring` | 4.2.0 |
| CSV, XML | `csv`, `quick-xml` (e-invoices) | 1.4.0, 0.42.0 |
| Tests in Rust | `proptest`, `insta`, `cargo-mutants` | 1.12.0, 1.49.0, 27.1.0 |
| UI | `react`, `vite`, `tailwindcss`, `typescript` | 19.3.1, 8.3.8, 4.3.5, 7.0.2 |
| UI pieces | `motion`, `@number-flow/react`, `clsx`, `cmdk` | 14.14.0, 0.6.2, 2.1.1, 1.1.1 |
| UI tests, lint | `vitest`, `oxlint` | 5.1.2, 1.95.0 |
| Build of the UI, pinned by #01 | `@vitejs/plugin-react`, `@tailwindcss/vite`, `@types/react`, `@types/react-dom`, `@types/node`, `serde_json` | 6.3.0, 4.3.5, 19.3.3, 19.3.0, 26.6.10, 1.0.154 |

## To verify

- PDFium is a shared library that must ship per system. Which prebuilt binaries `pdfium-render` 0.9.6 expects, and their licence (BSD style and Apache 2.0 for PDFium, to be verified at pdfium.googlesource.com). Settled by #06.
- The size of the macOS dmg. The Windows installer of 0.1.0 is 6.2 MB (#08, 2026-11-25), 1.0 shipped more than 100 MB.
- `ollama-rs` 0.3.6 was last released on 2026-07-24. Whether it covers structured output, images and tool calls of the current Ollama, or whether a thin client on `reqwest` is better. Settled in stage 2.
- The keychain prompt after every update on macOS, which hit 1.0 because the app is signed ad hoc. Whether an Apple Developer ID removes it is for the owner to decide (cost).

## End to end tests

Read on 2026-11-25 at v2.tauri.app/develop/tests/webdriver (#01):

- `tauri-driver` alone drives the app on Windows and Linux only, because macOS has no driver for WKWebView.
- The route the page recommends is WebdriverIO with `@wdio/tauri-service`, which runs a WebDriver server inside the app and so works on Windows, Linux and macOS without an outside driver.
- End to end tests of 2.0 take that route when the first view needs one. No package is added before.

## The shell of #01

**Decided:** 2026-11-25

- The installed app is named "Finanzarchiv 2" with the identifier `de.mrx7904.finanzarchiv2`, so it lives beside version 1.0 ("Finanzarchiv", `de.mrx7904.finanzarchiv`) with its own data folder and its own entry in the start menu. The window is titled "Finanzarchiv".
- Prerequisites on Windows, read on 2026-11-25 at v2.tauri.app/start/prerequisites: the workload "Desktop development with C++" of Visual Studio or its Build Tools, WebView2 (part of Windows 11), Rust through rustup with the MSVC toolchain as default. On macOS: the Xcode command line tools (`xcode-select --install`).
- The main window has a capability with no permission at all, and a CSP that allows only the app's own files and the IPC of Tauri. The dev server adds its websocket in `devCsp` only.
- `unwrap` and `expect` are denied in the core and allowed in tests (`src-tauri/clippy.toml`). `main` returns an `ExitCode` instead of the `expect` of the Tauri template.
- The icon is a placeholder until #07 (`src-tauri/icons/icon.svg`, rendered by `tauri icon`). The icons for Android and iOS that the command also writes are not kept.

## The typed boundary of #03

**Decided:** 2026-11-25

- The table above first named `tauri-specta` 1.0.2 and `specta` 1.0.5, the highest stable versions on crates.io. Read again for #03: they belong to Tauri 1. For Tauri 2 there are only release candidates, `tauri-specta` 2.0.0-rc.29 (2026-11-13) with `specta` 2.0.0-rc.28 and `specta-typescript` 0.0.15. Their own docs ask for exact pins during the beta, so all three are pinned with `=`.
- Rejected: `ts-rs` 12.0.1 is stable, but it generates types only. The command names and their arguments would be written by hand in TypeScript and could drift.
- `commands::builder()` lists every command once, for the invoke handler and for the bindings. A command is `#[tauri::command] #[specta::specta]` and returns `Result<T, CommandError>`. `ErrorHandlingMode::Result` makes each binding resolve to `{ status: "ok", data }` or `{ status: "error", error }`, so the window handles a failure without a `try`.
- `src/bindings.ts` is generated by `npm run bindings` (an ignored test that writes the file) and committed. The test `bindings_are_current` regenerates it in memory and fails when it differs. Shown able to fail on 2026-11-25: a new field in `AppStatus` without regenerating made `cargo test` exit with 101.
- App commands need no entry in the capability. The empty capability of #01 stays, checked in the release build: `app_status` answered.

## Test binaries on Windows

**Decided:** 2026-11-25 (#03)

As soon as the core used a type of the Tauri runtime, the test binary of the library ended with `STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139): Tauri needs Common Controls v6, and `tauri-build` embeds that manifest as a resource of the app binary only. `build.rs` now passes `WindowsAttributes::new_without_app_manifest()` and lets the MSVC linker embed `src-tauri/windows-app-manifest.xml` (the default manifest of `tauri-build` 2.7.6) into every binary with `/MANIFEST:EMBED` and `/MANIFESTINPUT`. Only on Windows with MSVC.

## Rejected

- **Electron again.** Works and is known, but the owner wants Rust, and the installers stay above 100 MB.
- **Leptos or Dioxus in the webview.** One language, but no React pieces of the six sources.
- **Slint or egui.** Smallest, but charts, the PDF preview and the look would all be written by hand.
