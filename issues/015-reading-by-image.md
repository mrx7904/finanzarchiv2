# #15 Reading the page as an image

**Goal:** Scans and decisive values that no reader found in the text are read from the image of the page by a local vision model.
**Requirements**: FA-20 · **Depends on**: #14

## Done when

- [x] The page image of #06 goes to the vision model at a width chosen by measurement. Values carry the reader `image:<model>` and the page as their source.
- [x] A scan gets its kind and its values this way. A document with text uses it only for a decisive value (gross of an invoice, Brutto and Netto of a payslip) that no reader found.
- [x] The five scans of the generated corpus are measured: right fields and seconds per page, under 15 minutes, model unloaded afterwards. The numbers stand in `decision-model.md`.
- [x] Whether local character recognition of the system (Windows.Media.Ocr, macOS Vision) would beat the model is checked at its primary source and written down, built only if the owner asks.

## Out of scope

- Checking the values, belongs to #16
