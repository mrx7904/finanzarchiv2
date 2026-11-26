# #28 The downloads folder

**Goal:** Invoices the owner downloads reach the archive by themselves, and nothing else of the downloads does.
**Requirements**: FA-33 · **Depends on**: #27, #14

## Done when

- [ ] Off until the owner switches it on. Then the downloads folder of the system is a watched folder, only at its top level.
- [ ] A file is taken only when it looks like an invoice, a payslip, a receipt or a statement: the signals of `kinds.rs` first, then the local model for a PDF with text that code did not take. A file without text, an image or any other type stays out. The reason is shown with the document.
- [ ] What lay there when the switch was turned on is left alone. "Vorhandene prüfen" looks at it with the same rule.
- [ ] Measured on a copy of the owner's downloads, counts only: files looked at, taken, left out.
