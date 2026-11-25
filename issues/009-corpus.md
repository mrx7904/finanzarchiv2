# #09 A generated corpus and a measure command

**Goal:** Code generates invoices, payslips and bank statements with known values into `fixtures/synthetic/`, and one command measures a reader on them and on the owner's private corpus.
**Requirements**: FA-13 · **Depends on**: #06

## Done when

- [x] The generator writes at least 30 invoices (with and without text layer, one e-invoice), 12 payslips and 6 bank statements in two layouts, with invented names and IBANs, and a file of the right values per document.
- [x] Generating twice with the same seed gives the same files, tested.
- [x] The measure command reports per field how many values a reader got right, as counts, and writes the result to `measurements/`.
- [x] With `CORPUS_PRIVATE` set the same command measures the owner's documents and writes only to the ignored folder.
- [x] The privacy step of the check passes over `fixtures/synthetic/`.

## Out of scope

- The readers to measure, they belong to stage 2
