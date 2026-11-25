# Decision: bookings carry the money, documents are evidence, every value knows where it came from

**Decided:** 2026-11-25 · applies to the schema, every repository module, the readers, matching and the views of money · the reasons are the lessons of 1.0 in `docs/analyse-finanzarchiv-1.0.md`, sections 3.1, 3.2 and 3.6

## Decision

### Money

- A **booking** (`bookings`) is the only source of money. Every movement is one booking on one account: from a bank statement, from cash (the account "Bar"), or typed by the owner for an account without statements.
- A document never counts as money by itself. An invoice, a payslip, a receipt or an Amazon order is **evidence** linked to bookings.
- A **link** (`links`) joins one booking and one document with a role (`invoice`, `receipt`, `payslip`, `order`) and an amount. Many to many, so a partial payment and a collective payment need no special case.
- An invoice without a link is an open liability with a due date, shown under "offen", never in the cash flow.
- A transfer between own accounts is two bookings joined as a pair, and counts in the balance, never as income or expense.
- The balance of an account on a day is its opening balance plus its bookings up to that day. A statement carries the bank's balance, and the difference is shown, never hidden.

In 1.0 the same question had three answers depending on the period (payments typed, invoices counted as payments, inside the statements only bookings) and special cases for "vorgemerkt", cash and PayPal (#114, #119 there).

### Provenance

- A value read from a document is a row of `field_values`: document, field, value, the reader that produced it with its version (`code:2`, `e-invoice:2`, `model:<name>`, `vision:<name>`, and `owner` without one), the place in the document (page, the line of text, a box when known) and the time.
- The checks a value passed or failed are rows of `field_checks`.
- The confirmed values are materialized into typed tables (`invoices`, `payslips`, `statements`) for queries and views. They are derived, the source is `field_values`.
- The history of a value, undo and "Warum?" are queries on these rows, no extra table per feature.

In 1.0 provenance grew piece by piece into `proposals`, `auto_acceptances`, `acceptance_corrections`, `sender_corrections`, `sender_rules`, `category_rules` and more.

### Rules and decisions

- Every automatic step is done by a rule with a stable id (for example `match.invoice.amount-and-name`) and parameters that stand in one module.
- Every automatic decision writes a row of `decisions`: the rule id, the parameters it used, the inputs it saw, what it changed and how to undo it. The UI shows "Warum?" and "Rückgängig" from this row.
- The owner's corrections are rows too, and a rule that learns from them (a category per sender, for example) reads them, never a copy.

### Parties

- A **party** is a sender, an employer or a counterparty of a booking, with aliases (the name on the invoice, the text of the bank, the PayPal merchant). Matching and categories work on parties, not on strings.

### Documents and intake

- Files stay unchanged under their SHA-256 hash, `<archive>/<first two hex>/<hash>.<ext>`, as in 1.0 (`decision-document-storage.md` there).
- Every way in creates one row of `intake_jobs` with its source and a stage (`stored`, `text`, `classified`, `read`, `checked`, `filed` or `waiting`). The queue survives a restart and resumes at the stage it reached.

## Sketch of the tables

`documents`, `intake_jobs`, `pages` (text with positions, rendered image cache), `field_values`, `field_checks`, `parties`, `party_aliases`, `accounts`, `statements`, `bookings`, `booking_pairs`, `links`, `categories`, `invoices`, `payslips`, `recurring`, `rules`, `decisions`, `corrections`, `settings`, later `tax_items`, `tax_assessments`, `chats`, `chat_messages`, `mail_accounts`, `mail_messages`. The columns are fixed in the issue that adds a table, this list is the frame.

## Consequences

- The Kontostand bridge of 1.0 ("vom Kontostand zum Kontostand") becomes a sum over bookings, with no money "in the sums but on no account".
- A document read before its booking arrives waits for nothing. It is filed, and the link follows when the booking comes.
- Version 2.0 does not import the archive of 1.0. The owner reads the original documents and statements in again (decided 2026-11-25), so the schema carries no field of 1.0 for compatibility.
