# #06 Text and pages of a PDF in the core

**Goal:** The core takes the text of every page with positions and renders pages as images, through PDFium, on Windows and macOS.
**Requirements**: FA-10 · **Depends on**: #05

## Done when

- [x] Which PDFium binaries `pdfium-render` expects, where they come from and their licence is checked at the primary source and stands in `decision-pdf.md` with the date (done with #04).
- [ ] The text of each page is stored with the box of each line in `pages`. A test on a generated invoice finds every printed amount with its line.
- [ ] A page renders as an image for the preview, cached, and the first page shows in the list of #04.
- [ ] A scan without a text layer is marked as such, for the reading by image of stage 2.
- [ ] The installer of #08 carries the library, checked by starting the installed app on Windows and on the Mac.

## Out of scope

- Character recognition of scans, decided in stage 2
