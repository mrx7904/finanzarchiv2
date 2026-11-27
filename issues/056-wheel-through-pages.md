# #56 Through the pages of a document by the wheel

**Goal:** The owner reads a document of several pages by turning the wheel, as in every PDF reader.
**Requirements**: FA-56 · **Depends on**: #55

Asked by the owner on 2026-11-27 ("Kann man PDF Seiten mit Mausrad scrollen? Das noch dazu bitte"). As of 0.7.0 the wheel moves inside one zoomed page only. At "Ganze Seite" it does nothing, and no wheel leads from one page to the next: pages turn by the buttons and, in "Groß lesen", by the keys (`src/documents/PageView.tsx`).

## Done when

- [ ] In "Groß lesen" the pages of a document stand under each other in one column that scrolls by the wheel, the touchpad and the scroll bar. The count "n von m" and the marked small image at the left follow the page that fills most of the view.
- [ ] A click on a small image, the buttons back and forward, and the arrow and page keys scroll to that page.
- [ ] The page column of Dokumente and of a question in "Prüfen" scrolls the same way through the pages. At "Ganze Seite" one turn of the wheel leads to the next or the previous page.
- [ ] Strg with the wheel (⌘ on the Mac) zooms in all three places and keeps the point under the pointer where it is. Without Strg the wheel never zooms.
- [ ] Only the pages in view and one before and after are asked from the core. Measured on a document of at least 30 pages of the generated corpus: the calls of `page_image` after opening are at most 4, and scrolling to the last page asks no page twice at one width.
- [ ] Tests cover which page counts as the current one while scrolling, and the wheel at the first and the last page.
- [ ] Seen by the owner on the largest document of the archive.

## Out of scope

- Searching inside a document and selecting its text: a requirement of its own when the owner asks for it, as in #54.
- Two pages side by side.
