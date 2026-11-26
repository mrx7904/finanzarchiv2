# Decision: parties and their categories

**Decided:** 2026-11-26 (#25, FA-30), the list of categories as the owner asked for it · applies to `src-tauri/src/parties.rs`, the tables `parties`, `party_aliases`, `categories`, `corrections` (migration 0013), `bookings.party_id`, `bookings.category_id`, `documents.party_id`

## Decision

- A party is who money goes to or comes from. Each spelling is an alias, matched by its folded form: the legal form left out (`naming::without_legal_form`), lower case, only letters and digits. So "Testhaus Energie AG", "TESTHAUS ENERGIE" and "Testhaus-Energie AG," are one party. A new spelling joins a party by the IBAN beside it: the IBAN of the other side named in the purpose of a booking, or the IBAN of a document. Otherwise a new spelling is a new party, named by its spelling without the legal form.
- Every booking with another side gets its party when its statement is read, and every filed invoice, receipt and payslip gets the party of its sender or employer (`parties::assign`, in the transaction of the stage). `READERS_VERSION` 5 gives the bookings already read their parties.
- **Categories.** The owner asked on 2026-11-26 for "eine sinnvolle Liste mit 20-30 Kategorien, welche man in der App anpassen und erweitern kann". `parties::DEFAULT_CATEGORIES` holds 28 categories:
  - 5 for money in, starting with "Gehalt und Lohn";
  - 23 for money out, from "Miete und Wohnen" to "Sonstige Ausgaben".

  The archive gets them once, a new one and one from before. Renaming, removing and adding are in the core (a name is needed, and the same name twice is refused), the dialog comes with the views of #26. A removed category leaves its parties and bookings without one.
- **A booking takes the category of its party** unless the owner gave the booking its own (`category_of`). "Für alle von …" sets the party's category and lets the booking follow it. Each choice is a row of `corrections`, the record the rule reads.
- **Merging and splitting.** Merging moves the spellings, bookings and documents of one party to another. Splitting gives a spelling its own party with the bookings of that spelling. Each is a correction with what it moved, and `undo` moves it back.

## Measured on 2026-11-26

The owner's archive (a copy, migrated and read again, counts only):

| What | Count |
|---|---|
| Parties | 160 |
| Spellings | 165 |
| Bookings with another side that have a party | 1,885 of 1,894 (the rest have only digits or signs as their other side) |
| Filed documents with a party | 19 of 19 |
| Categories | 28 |
| Bookings with a category | 0, until the owner chooses |

# Decision: the model gives the parties their categories

**Decided:** 2026-11-26 (#43, FA-46), the owner: "Sorge dafür dass das Modell gut die Kategorien zuordnen kann", no explanations in the window · applies to `src-tauri/src/categorize.rs`, `parties::payee_of`, `parties::payees`, `parties.category_by` and `parties.category_asked_at` (migration 0022)

## Decision

- **Asked per party, once.** A party without a category, never asked and with bookings, is asked: its name, the count of bookings in and out, a typical amount, the rhythm (weekly to yearly), the kinds of booking, up to six purposes and the senders of its documents (`categorize::context_of`). The model answers one category of the list or "keine". `category_asked_at` marks it asked, so a "keine" is not asked again.
- **The owner wins.** A category the owner chose, also none, is `category_by = 'owner'` and the model never touches it; categories from before the migration count as the owner's. The model's own are `'model'`.
- **The code checks the model.** A category against the direction of most of the party's bookings (an income category for a party that is paid) is dropped. Payment services (PayPal, Klarna, Amazon Pay and the like) get none without asking: their payee tells.
- **A payment service's payee is the party.** A bank prints the payee into the purpose of a PayPal booking ("… .PP/. Beispiel Musik, Ihr Einkauf bei Beispiel Musik"). `parties::payee_of` reads it, and the booking's party is that payee, when it is read and once at start for the bookings read before (`parties::payees`), unless the booking has a category of its own.
- **In the background.** A pass a minute and a half after start, then every ten minutes, 40 parties at a time, the archive locked only to read and to keep. The window shows only the category, no reason.

## Measured on 2026-11-26

`categorize::tests` with invented parties and a stub model: the owner's choice stays, a "keine" is not asked again, a category against the direction is dropped, a payment service is not asked, a payee is read from both shapes of purpose.

`examples/measure.rs --reader categories`, 31 invented parties of public shops and services, a payment service and a private person: 31 of 31 right with `gemma4:12b-it-qat`.

A copy of the owner's archive, counts only, the model unloaded afterwards:

| What | Count |
|---|---|
| PayPal bookings given their payee as party | 478, 477 of them with a category after the pass |
| Parties asked | 264 in 128 s |
| Parties with a category by the model | 248 |
| Parties without one | 64: 45 only moves between own accounts, 6 payment services, 13 others |
| Counted bookings without a category | 398 of 1,730 (875 before #43) |
