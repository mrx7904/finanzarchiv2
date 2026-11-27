# #55 The page of a document stands still

**Goal:** A page that is shown stays shown: it never flickers, and it is asked from the core once per size.
**Requirements**: FA-55 · **Depends on**: #54

Reported by the owner on 2026-11-27 ("eine PDF Ansicht unter Prüfen im großen Bild flackert stark"). Reproduced the same day in Chromium with the stand-in of the core at 1600 × 900, a question open in "Prüfen": 75 calls of `page_image` in 3 seconds, the width asked for alternating between 776 and 761 px.

The cause, in `src/documents/PageView.tsx` and `src/documents/PageImage.tsx`: at a zoom other than "Ganze Seite" the page is higher than its box, so a scroll bar appears and takes 15 px of the measured width. The new width is a new request, `PageImage` empties its image before the answer comes, the box loses its height, the scroll bar goes, the width grows back, and the loop starts again. "Prüfen" starts at 100 %, so it shows there at once. The same loop runs in Dokumente and in "Groß lesen" at every zoom but "Ganze Seite".

## Done when

- [ ] The width a page is fitted to does not change when a scroll bar comes or goes.
- [ ] The window asks the core only for the widths the core renders (`previews::WIDTHS`), so two widths that give the same image are one request.
- [ ] While another width or page loads, the image shown before stays, at its place and size. A page never shows an empty box between two images of the same page.
- [ ] A test counts the calls of `page_image` for a page at 100 % in a box lower than the page, over a simulated second: at most 2. It fails on the code of 0.7.0.
- [ ] `npm run check:views` gains the probe: a question of "Prüfen" at 1600 × 900 makes at most 2 calls of `page_image` per page within 3 seconds.
- [ ] A page that cannot be rendered shows "Keine Vorschau" in its box, not an empty one.
- [ ] Seen by the owner in the installed app on a real document in "Prüfen", in Dokumente at 150 % and in "Groß lesen".

## Out of scope

- Scrolling through the pages by the wheel: #56.
- The page count and the mark in "Prüfen": #57.
- How the image travels from the core to the window: #60.
