# #07 The design system, chosen by the owner

**Goal:** Tokens and base pieces of the UI exist, built from the six UI sources, in the variant the owner picked at real size.
**Requirements**: FA-11 · **Depends on**: #01

## Done when

- [ ] At least three variants (the Apple look of 1.0 among them) are shown to the owner at real size in light and dark, on Windows and on the Mac, with a sidebar, a list, a preview, a form and a dialog.
- [ ] The owner's pick and the reasons stand in `decision-ui-libraries.md`.
- [ ] Colors, type, spacing, radii and surfaces are tokens in one file. Text has a contrast of at least 4.5 to 1, checked by a test over the tokens.
- [ ] Button, field, list row, dialog, menu, empty state and the quick search exist as components, each piece taken from a source with its header comment.
- [ ] Every motion stops under reduced motion, checked by a test.

## Out of scope

- The views themselves, each comes with its stage
