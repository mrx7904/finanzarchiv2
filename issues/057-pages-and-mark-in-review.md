# #57 Every page and the right mark in "Prüfen"

**Goal:** A question shows every page of its document, and the mark stands on the line it means and nowhere else.
**Requirements**: FA-57 · **Depends on**: #55

Found by reading the code on 2026-11-27, not yet seen in the window. `ReviewQuestion` carries no page count, so `Review.tsx` gives the page of the question as the count and no later page can be reached (named as open in #54). The mark is drawn on whatever page is shown. At "Ganze Seite" it is placed by the box around the image, not by the image, which is narrower than its box for a portrait page.

## Done when

- [ ] `ReviewQuestion` carries the page count of its document. The tools under the page of a question page through all of them.
- [ ] The mark shows only on the page its reading stands on. Turning away hides it, turning back shows it again.
- [ ] At every zoom, "Ganze Seite" included, the mark lies on the marked line: a test places a mark at known fractions on a portrait and on a landscape page and compares the box of the mark with the box of the image.
- [ ] A question whose readings stand on different pages opens on the page of the first reading, and choosing a reading with a mark turns to its page.
- [ ] In the list of a reason the page of the chosen row is asked at the width it is shown, not at a fixed 900 px.
- [ ] Seen by the owner on a document of several pages that waits in "Prüfen".

## Out of scope

- The flicker of the page: #55. Scrolling by the wheel: #56.
- New questions or checks: stage 2 owns them.
