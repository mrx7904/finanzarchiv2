# Decision: a generated corpus in the repository, the owner's documents beside it, one measure command

**Decided:** 2026-11-25 (#09) · applies to `src-tauri/examples/corpus.rs`, `src-tauri/examples/measure.rs`, `fixtures/synthetic/`, `measurements/`

## Decision

- `npm run corpus` (the example `corpus`, seed 2026) writes `fixtures/synthetic/`: 30 invoices (every sixth at 7 %, five as scans whose page is a JPEG, the last an e-invoice with an embedded `factur-x.xml`), 12 payslips of one year and 6 statements of one account in two layouts (A with a sign, B with S and H). Every PDF has a JSON beside it with the right values as printed and the amounts in cents. Every name is invented, every IBAN only carries a valid check sum.
- The generator is a Cargo example, not part of the app. It uses the dev dependency `lopdf` to write PDFs, PDFium to render the page of a scan and `image` to encode it. The same seed gives the same bytes (tested by the example's own tests, which `cargo test` and so the check run).
- `npm run measure` (the example `measure`) runs a reader over the corpus and writes, per kind and field, how many values it got right to `measurements/synthetic-<reader>.json`, which is committed. With `CORPUS_PRIVATE` set it measures the owner's folder too and writes only to `measurements/private/`, which git ignores. A result holds counts, never a value.
- The owner's private corpus is a folder of real documents with a JSON beside each in the same format. It never enters the repository (`decision-public-repository.md`).
- The one reader so far is `text-layer`: a field is right when its printed value stands in the text PDFium reads. It is the floor of stage 2. A value that is not in the text layer must come from reading the page as an image.

## Why

In 1.0 no document had confirmed values when the automation was planned (0.12), so there was nothing to measure against, and measurements on the owner's archive were run by hand and kept nowhere (`decision-automatic-review.md` there). A generated corpus is in the repository from the first reader on, and the owner's corpus is measured with the same command.

## The first measurement

On 2026-11-25, `text-layer` on the synthetic corpus: every field of the 25 invoices with text, the 12 payslips and the 6 statements (32 bookings) right, 0 of 5 for every field of the scans, as expected. The file is `measurements/synthetic-text-layer.json`.
