# Decision: PDFium reads every PDF, shipped beside the app

**Decided:** 2026-11-25 (#04, the binary part of #06 taken forward) · applies to `src-tauri/src/pdf.rs`, `scripts/fetch-pdfium.mjs`, `src-tauri/tauri.*.conf.json`

## Decision

- PDFs are opened with PDFium, the engine of Chrome, through `pdfium-render` 0.9.6. Its default feature `pdfium_latest` is `pdfium_8125`, so the library is release `chromium/8125` of `bblanchon/pdfium-binaries` (published 2026-10-26), not the newest `chromium/8179`, whose API the crate was not built against.
- `node scripts/fetch-pdfium.mjs` downloads the archive of this system (`--all` for every one), checks its SHA-256 against the digest GitHub reported on 2026-11-25 and unpacks it to `src-tauri/pdfium/<platform>/`, which git ignores. The check runs it before the Rust tests, a second run does nothing.
- The library and its licences ship as resources of the app: `tauri.windows.conf.json` and `tauri.macos.conf.json` map them to `pdfium/` in the resource folder, Apple silicon only on macOS for now. The app binds it from there at start, a debug build also from `src-tauri/pdfium/`. Without it the app starts, and a PDF is refused with "PDFs können gerade nicht geprüft werden".
- PDFium binds once per process (`OnceLock` in `pdf.rs`, `pdfium-render` keeps its bindings in a static as well).
- An import asks PDFium before it stores anything: a document that fails to open is refused as damaged, one that asks for a password is refused as protected (`FPDF_ERR_PASSWORD`). The type of a file comes from its first bytes, never from its name.

## Why

- A wrong guess at import costs more than a refusal: 1.0 guessed from the file and stored damaged or protected files that every later step then failed on. PDFium is the engine that renders the PDFs for the preview and reads their text in #06, so the check at import uses the same reader that later works on the file.
- `lopdf` would need no shared library, but it fails on PDFs that viewers open, and a valid invoice must never be refused. It stays a dev dependency that writes test PDFs, the encrypted one included.

## Licences

Read on 2026-11-25 in the unpacked archive: PDFium is under the BSD style licence of "The PDFium Authors" (`licenses/pdfium.txt`), which asks that binary redistributions reproduce the notice. The build scripts of `pdfium-binaries` are MIT (GitHub reports MIT for the repository). The archive carries the licences of the bundled libraries (FreeType, ICU, libjpeg-turbo, libpng, zlib, OpenJPEG, LCMS and others) in `licenses/`. All of them ship with the app. The window that lists them comes with the settings.

## Evidence

On 2026-11-25 on Windows 11: the Rust tests refuse an empty file, a text file, a truncated PDF and a PDF encrypted with a user password, and store a generated PDF under its hash. In the release build a folder import through the system dialog (driven by UI Automation) took a PDF and a PNG from a folder with a subfolder and refused a damaged PDF and a text file, each with its reason. Not verified: dropping files on the window, and the library on the Mac.

## Text, pages and previews (#06)

**Decided:** 2026-11-25

- The stage `text` of the intake reads every page of a PDF through PDFium: its size in points and its lines, each with its box in PDF points from the lower left corner (`pages`, `page_lines`, migration 0004). PDFium gives runs of text (`text().segments()`). Runs whose middle lies within the height of a line form one line, left to right, with a space where the gap is wider than a fifth of the line height (`pages::group_lines`).
- `documents.text_state` says what the intake found: `text`, `scan` (a PDF without any text on any page, to be read as an image in stage 2) or `image` (PNG, JPEG, WebP).
- Pages are rendered by PDFium, image files are scaled by `image` 0.25.10 (PNG, JPEG, WebP only, no default features). The window asks through `page_image`, which answers a PNG as a data URL, so the CSP needs no asset protocol. The width is rounded up to 160, 640 or 1280 pixels, and each image is kept in `Vorschau/<hash>-<page>-<width>.png` beside the archive. The name carries the hash of the original, so a cached image is never stale. The window asks for a page only once it is in view.
- Evidence of 2026-11-25: a generated invoice gives its six lines in order, each amount with its line at the left margin, "Fällig" with its umlaut. A page with a drawing only is a scan. In the release build three invented documents (an invoice with text, a PDF with a drawing only, a PNG) ended as `text` with 7 lines, `scan` and `image`, the list showed the first page of each and the preview the chosen one, four images lay in `Vorschau/`.
