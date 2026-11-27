# #54 Reading a document large, and the focus of "Prüfen"

**Goal:** The owner reads every page of a document at a size that can be read, and answers questions with nothing else in the window.
**Requirements**: FA-54 · **Depends on**: #46, #47

Rule 26: [`knowledge/decision-views.md`](../knowledge/decision-views.md), "Fitting a view". The pictures: `?v=dokumente` (the tools under the page), `?v=dokument-lesen`, `?v=pruefen-fokus`. Pages come from the core as images of a width (#06, `PageImage`).

## Done when

- [ ] Under the page of every document stand its tools: the page and the count with back and forward, smaller and larger with "Ganze Seite" as the start, and "Groß lesen". They show wherever a page of a document shows, in "Prüfen" too. A document of one page shows no paging.
- [ ] "Groß lesen" (the button, or the space bar on the chosen row) shows the document over the whole window without the sidebar: the pages as small images at the left, the page at its zoom, the tools in the top bar. Esc and "Schließen" lead back to the row it came from. The arrow keys and the page keys turn pages, Strg with the wheel zooms.
- [ ] A page is asked from the core at the width it is shown, so it stays sharp at 200 %. Measured: the time until a page of the owner's largest document stands at 100 % and at 200 %.
- [ ] In "Prüfen" the focus (its button, or F) hides the sidebar and leaves the page, the question and the progress. Esc ends it. It stays on from question to question and ends by itself when nothing is left to ask.
- [ ] Tests cover the paging at the first and the last page, the limits of the zoom and the way back.
- [ ] Seen by the owner on the real archive, on Windows.

## Out of scope

- Searching inside a document and selecting its text: a requirement of its own when the owner asks for it.
- A full screen of the system: the window stays a window.
