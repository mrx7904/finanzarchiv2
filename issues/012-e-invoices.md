# #12 E-invoices from their XML

**Goal:** An invoice with an embedded ZUGFeRD, Factur-X or XRechnung XML is read from that XML, and its values count as exact.
**Requirements**: FA-17 · **Depends on**: #11

## Done when

- [ ] The attachment is found through PDFium (the embedded files of the document) and parsed with `quick-xml`, version and namespaces of the Cross Industry Invoice checked at their primary source with the date.
- [ ] Invoice number, date, seller, net, VAT, gross and due date come from the XML. The reader is `e-invoice`, the source of each value is the XML element.
- [ ] The e-invoice of the generated corpus is read with every field right. A broken XML falls back to the other readers and says so.
- [ ] A standalone XRechnung XML (no PDF) is out of reach of the intake today. Whether to accept `.xml` files is written down as a decision.

## Out of scope

- Writing e-invoices
