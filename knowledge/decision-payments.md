# Decision: PayPal payments and Amazon orders

**Decided:** 2026-11-26 (#32, FA-36). Amazon: the owner downloads the invoice, the app never signs in at Amazon (the owner's answer of 2026-11-26) · applies to `src-tauri/src/readers/paypal.rs`, `src-tauri/src/payments.rs`, `src-tauri/src/orders.rs`, the order number in `readers/invoice.rs` and `links.rs`, the tables `receipts_of` and `orders` (migration 0021), "Bestellungen" in "Dokumente"

Taken over from 1.0 (`decision-paypal-payments.md` there, 2026-11-15): only PayPal's own domain counts, the seller's invoice first, two mails of one payment are one.

## PayPal

- **Only the text of a mail from PayPal's own domain** (`paypal.de`, `.com`, `.at`, `.ch` and the others, with any subdomain) is a PayPal payment. A document that only names PayPal, or a sender like `paypal.de.example.com`, goes the way of every document.
- **Code reads it** (`readers::paypal`), measured on the owner's 79 PayPal mails (masked):
  - the payee from "Händler …", else PayPal's sentence ("Sie haben 9,99 € EUR an … gezahlt"), else the subject ("Beleg für Ihre Zahlung an …"), the dots of a cut name dropped;
  - the amount from the sentence, else "Gesamtbetrag";
  - the day from "Transaktionsdatum", else the day of the mail;
  - PayPal's transaction code from "Transaktionscode" or "Transaktions-ID", on its line or the next, stored as the number;
  - the seller's order number from "Bestellnummer";
  - the field `paid_via` = PayPal.
  The reader of invoices does not run on such a mail. In the intake it is a receipt by code (signal `paypal`) when payee, amount and day were read.
- **In the mail round** a PayPal mail is read by code and never asked about. A second mail with a transaction code already taken is not taken ("Eine zweite Mail derselben PayPal-Zahlung, nicht übernommen.", `mail_messages.paypal_code`). Documents of one payment already in the archive are merged at start (`payments::merge_twins`): the first stays, the others go to the Papierkorb, unless the owner did something with them.
- **The seller's invoice first** (`payments::pair`): an invoice with the same gross, its date within 10 days of the payment, and the seller's name (folded, one in the other) or the order number makes the PayPal mail its receipt (`receipts_of`, rule `receipt.seller-invoice:1`). Exactly one must fit, else the PayPal mail stays the payment itself. The booking the PayPal mail was linked to then belongs to the invoice as well (rule `link.receipt:1`). The window names the pair in the preview of both.
- **The money counts once.** The bookings are the money of the archive. A PayPal payment paid from the bank is its booking, with the PayPal mail and the seller's invoice as its documents. A payment from the PayPal balance or a card has no booking of a statement here, and its mail stands for it (for the evaluations of stage 5).

## Amazon

- **An order mail is an order, not a document.** A mail from Amazon's own domain is read for its order numbers (`302-1234567-1234567`), the day of the mail and, where it names one order, its total (`orders::read`). Without a PDF it is not taken ("Eine Bestellung bei Amazon (…), die Rechnung lädst du bei Amazon herunter."). Orders are kept in `orders` by number.
- **The owner downloads the invoice** into the inbox or the downloads. The reader of invoices reads the order number (Amazon's form anywhere, else after "Bestellnummer" or "Bestellnr."), `orders::match_invoices` matches the invoice to its order, and `links` finds its booking by that number in the purpose as it does by an invoice number.
- **"Bestellungen …"** in "Dokumente" lists the orders, newest first, with the invoice or "Rechnung fehlt" and how to get it.
- **The mails looked at before #32** are read once more for their orders: migration 0021 sets every mailbox back to the start of its reach, a known mail is skipped by its hash, and a known mail of Amazon whose orders were not read is fetched once more for them (`mail_messages.orders_read`).

## Reading again keeps a statement's bookings

Found on a copy of the owner's archive: the readers' version 6 re-read every document in one transaction, and re-reading a statement deleted its bookings, which links, moves and the owner's corrections refer to since #23 to #25. The foreign keys refused it and nothing was read again. Now a statement read again as before keeps its bookings and everything on them (`statements::unchanged`). When it reads otherwise, its links, moves and the bookings in corrections are let go first (`statements::detach`), and the rules find the links again.

## Measured

Tests with invented mails, documents and bookings:
- `readers::paypal::tests`: payee, amount, day, code and order of the layout, the payee from the sentence or the subject, the code on the next line, only PayPal's own domains;
- `payments::tests`: the seller's invoice pairs and takes the booking, the order number pairs when the names differ, two fitting invoices pair none, two mails of one payment are one and a held one stays;
- `mail::round::tests`: PayPal by code once, a forged sender to the model, Amazon's orders with total and day, a known Amazon mail read once more for its orders, no command writes;
- `orders::tests`, `statements::tests` (a statement read again keeps its bookings).

On a copy of the owner's archive, 2026-11-26, counts only: 353 documents read again without a broken foreign key. 80 PayPal mails: amount and day read by code in 80, the payee in 71, the transaction code in 62, the order number in 73. 78 filed, 2 waiting (sum, duplicate), 26 linked to a booking. No two of one payment, no seller's invoice that fits yet.

On the owner's two mailboxes, 30 days, read only, counts only (`measure --reader mail --days 30`, gemma4:12b-it-qat, model unloaded afterwards): 281 mails, 14 PayPal payments read by code (all 14 with their transaction code, no second mail of one), 39 Amazon orders told and none taken as a document, 4 mails with PDF attachments and 1 as its text, 207 left out by code, 16 by the model. 21 mails went to the model, 41 did in the same days before #32. 61 s.

## Open

- No total was read of the 39 Amazon orders: their mails print it otherwise than at the start of a line. The order number, which matches invoice and booking, was read in all.
- The payee is missing in 9 of the 80 PayPal mails of the archive, in the text and in the subject.
