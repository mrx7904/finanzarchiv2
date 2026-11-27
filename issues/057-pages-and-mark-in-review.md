# #57 Every page and the right mark in "Prüfen"

**Goal:** A question shows every page of its document, and the mark stands on the line it means and nowhere else.
**Requirements**: FA-57 · **Depends on**: #55

Found by reading the code on 2026-11-27, not yet seen in the window. `ReviewQuestion` carries no page count, so `Review.tsx` gives the page of the question as the count and no later page can be reached (named as open in #54). The mark is drawn on whatever page is shown. At "Ganze Seite" it is placed by the box around the image, not by the image, which is narrower than its box for a portrait page.

## Done when

- [x] `ReviewQuestion` carries the page count of its document. The tools under the page of a question page through all of them.
- [x] The mark shows only on the page its reading stands on. Turning away hides it, turning back shows it again.
- [x] At every zoom, "Ganze Seite" included, the mark lies on the marked line: a test places a mark at known fractions on a portrait and on a landscape page and compares the box of the mark with the box of the image.
- [x] A question whose readings stand on different pages opens on the page of the first reading, and choosing a reading with a mark turns to its page.
- [x] In the list of a reason the page of the chosen row is asked at the width it is shown, not at a fixed 900 px.
- [ ] Seen by the owner on a document of several pages that waits in "Prüfen".

## Evidence

Seen on 2026-11-27 in the browser with the stand-in of the core at 1600 × 900, a question of two pages whose second reading stands on page 2: the tools read "1 von 2" and turn to page 2, where the mark of the first reading is gone and comes back on page 1. At 100 % the image spans 300 to 1061 px and the mark begins at 361 px (0.08 of the page), at "Ganze Seite" the image spans 447 to 929 px and the mark begins at 486 px, where 0.7.0 placed it by the box. Pointing at the second reading after "Nein" turns to page 2 with its mark and keeps the zoom. The core gives each reading the size of its own page (`review::tests`), `mark.test.ts` compares the mark with the image on a portrait and a landscape page. "Choosing" a reading is pointing at it (hover or focus), since a click on it answers the question. Not seen in the Tauri window: UNVERIFIED there.

## Out of scope

- The flicker of the page: #55. Scrolling by the wheel: #56.
- New questions or checks: stage 2 owns them.
