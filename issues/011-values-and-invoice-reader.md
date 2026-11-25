# #11 Values with their source, and invoices read by code

**Goal:** Every value read from a document is a row with its reader, its line and its checks, and the code reader fills these rows for invoices.
**Requirements**: FA-15, FA-16 · **Depends on**: #10

## Done when

- [x] `field_values` and `field_checks` exist as in `decision-data-model.md`: document, field, value as printed, normalized value, reader, page, line box, time. A migration and a test that a value without a reader is refused.
- [x] A `Reader` trait in the core, so code, e-invoice, model and image readers stand side by side and are measured the same way.
- [x] The code reader for invoices finds invoice number, invoice date, net, VAT rate, VAT, gross, due date, IBAN and sender from labelled lines, amounts through `Cents::parse_de`, dates in code.
- [x] `npm run measure -- --reader code` on the generated corpus: every field of the invoices with text right, the result in `measurements/`.
- [x] Measured once on the owner's documents of kind invoice, counts only, the counts in the commit.
- [x] The stage `read` of the intake runs the readers that fit the kind.

## Out of scope

- E-invoices, belong to #12
- Payslips, belong to #13
- The model, belongs to #14
- Deciding what is filed, belongs to #16
