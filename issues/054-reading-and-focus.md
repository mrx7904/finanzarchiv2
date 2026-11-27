# #54 Reading a document large, and the focus of "Prüfen"

**Goal:** The owner reads every page of a document at a size that can be read, and answers questions with nothing else in the window.
**Requirements**: FA-54 · **Depends on**: #46, #47

Rule 26: [`knowledge/decision-views.md`](../knowledge/decision-views.md), "Fitting a view". The pictures: `?v=dokumente` (the tools under the page), `?v=dokument-lesen`, `?v=pruefen-fokus`. Pages come from the core as images of a width (#06, `PageImage`).

## Done when

- [x] Under the page of every document stand its tools: the page and the count with back and forward, smaller and larger with "Ganze Seite" as the start, and "Groß lesen". They show wherever a page of a document shows, in "Prüfen" too. A document of one page shows no paging.
- [x] "Groß lesen" (the button, or the space bar on the chosen row) shows the document over the whole window without the sidebar: the pages as small images at the left, the page at its zoom, the tools in the top bar. Esc and "Schließen" lead back to the row it came from. The arrow keys and the page keys turn pages, Strg with the wheel zooms.
- [x] A page is asked from the core at the width it is shown, so it stays sharp at 200 %. Measured: the time until a page of the owner's largest document stands at 100 % and at 200 %.
- [x] In "Prüfen" the focus (its button, or F) hides the sidebar and leaves the page, the question and the progress. Esc ends it. It stays on from question to question and ends by itself when nothing is left to ask.
- [x] Tests cover the paging at the first and the last page, the limits of the zoom and the way back.
- [ ] Seen by the owner on the real archive, on Windows.

## Out of scope

- Searching inside a document and selecting its text: a requirement of its own when the owner asks for it.
- A full screen of the system: the window stays a window.

**Status:** built on 2026-11-27, waiting for the owner's look on Windows.

- `PageView` and `PageTools` (`src/documents/PageView.tsx`) under the page in Dokumente and in Prüfen: paging for more than one page, smaller and larger from "Ganze Seite" (50 to 300 %), "Groß lesen". `ReadLarge` over the whole window with the pages at the left; Esc, "Schließen", arrow and page keys, Strg with the wheel. The space bar on the chosen row of Dokumente opens it.
- The core renders a page at 160, 640, 1280, 1920 or 2560 px (`previews::WIDTHS`, two widths new), the window asks for the width it shows (`requestWidth`, the screen's pixel ratio included).
- Measured on the owner's largest document (8 pages, `examples/measure.rs --reader page-timing --files <Archiv>`, without the cache): the first page at 1280 px in 30 ms, at 2560 px in 31 ms.
- Prüfen: "Fokus" in the top bar and F hide the sidebar; Esc ends it, it ends by itself when no question is open. In Prüfen the page starts at the column's width, so the marked place stays on it. The page count of a question is not known to the view yet, so a question shows the zoom without paging.
- Tests: `src/documents/pages.test.ts` (paging at the ends, the limits of the zoom, the width asked for). Seen in Chromium with the stand-in of the core; `npm run check:views`: 0 findings.
