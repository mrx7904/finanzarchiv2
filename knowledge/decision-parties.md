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
