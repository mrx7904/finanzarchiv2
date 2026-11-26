# Decision: statements read by code, proved by their balance

**Decided:** 2026-11-26 (#20, FA-25), the lessons of 1.0 taken over without its values · applies to `src-tauri/src/statements/`, the tables `accounts`, `statements`, `bookings` (migration 0011), the check `check:balance`

## Decision

- A statement is read by code in the layout of its bank, never by the model. The balance is its proof: the old balance plus the bookings is the new balance to the cent (`check:balance`). A statement that adds up is filed by `file.checks-passed:1`. One that does not add up, or whose layout no reader knows, waits with the sums or the reason.
- The layouts are named by their form ("Layout A"), never by a bank (`decision-public-repository.md`). They were found from the owner's statements read masked on this computer (every digit a 9, every word outside a vocabulary of banking terms a `w`). Nothing of their content is in the repository.
- **Layout A** is a table with the heads "Bu-Tag Wert Vorgang":
  - A booking starts with its day and day of value ("dd.mm." each), then its kind, then the amount at the right with S (money out) or H (money in).
  - The indented lines below it are the other side (the first) and the purpose (the rest).
  - "alter Kontostand vom" and "neuer Kontostand vom" frame the table, and the bank's text after the table is no part of it.
  - "Übertrag auf/von Blatt" carries the table from page to page, and the head of each page is skipped up to its column heads.
  - The codes of the form at the left edge of the page are no part of it.
  - The year of a "dd.mm." comes from the closing day: a month after the closing month belongs to the year before.
  - The first statement of a new account prints "alter Kontostand" without a day. The balance then holds up to the day before its first booking.
- What is read goes into the tables:
  - `statements`: layout, number, both days, both balances and the sum of the bookings.
  - `bookings`: day, day of value, amount in whole cents with its sign, kind, other side, purpose, and the page and box of its first line.
  - Values for the name and the checks go into `field_values`: the bank as `sender`, the IBAN, the number, the period, both balances.

  Reading again replaces all of it.
- The first statement of an IBAN makes its account (`accounts`), named by the bank without its legal form and the last four characters of the IBAN.
- The statements filed in stage 2 by `file.statement:1` are checked again at the next start (`checks::supersede_statement_rule`): the old decision is marked undone (which `check:undone` does not count), the document waits for the checks, and `READERS_VERSION` 4 reads its bookings.

## Measured on 2026-11-26

- Generated corpus, `npm run measure -- --reader statements`, results in `measurements/synthetic-statements.json`:
  - the 3 statements of layout A have their balances, IBAN and every one of 41 bookings right (day, amount, other side, kind), and all add up;
  - the 3 of an unknown layout are read as none, as they should be;
  - the corpus has a statement over two pages with a carry, and a December booking in January.
- The owner's archive (a copy, counts only):
  - 110 of 110 statements read, 110 of 110 adding up to the cent, 1,924 bookings, 3 accounts;
  - 1,894 of the bookings with another side, the rest are closings and the like;
  - on the run of the start of the new version, all 110 were filed by `check:balance` and their 110 decisions of stage 2 superseded.

## Statements as CSV, CSV before PDF (#21)

- **Layout B** is a CSV export:
  - Cells are split by ";", and a quote opens a quoted cell only at its start.
  - The text is UTF-8 (its BOM dropped) or Windows-1252, as the owner's bank writes it.
  - Above the table stand lines of key and value: "Umsatzanzeige", "IBAN", "Kontoname", "Bank", "Zeitraum" ("dd.mm.yyyy - dd.mm.yyyy") and "Saldo".
  - The table is headed "Buchung;…;Saldo;Währung;Betrag;Währung" and lists the bookings newest first, each with the balance after it. They are stored oldest first.
  - The old balance is the oldest booking's balance after it minus its amount, the new one the "Saldo" of the head. The period starts the day before "Zeitraum" begins, as a PDF's old balance does.
  - Proved like a PDF by `check:balance`.
- The intake takes a `.csv` only when its bytes read as layout B (`FileType::Csv`, detected from the content, never the name). Any other CSV is refused at the door as before.
- Its lines are kept as the lines of one page (`text_state` `table`), so the readers, the checks and the name work on it as on a PDF. The stage `classified` gives it the kind statement by its layout.
- The table `documents` was built anew to allow `csv` (migration 0012). Since then the migration runner turns the foreign keys off while migrations run and checks them with `PRAGMA foreign_key_check` before each commit, as SQLite recommends for a changed CHECK.
- **CSV before PDF:** a booking of another statement of the same account with the same day and amount is the same booking, when it is the only such one of the day on both sides. The CSV's booking counts, the PDF's stays as `replaced_by` it. Reading a statement again, or removing it, lets the bookings it replaced count again.

### Measured on 2026-11-26

- Tests:
  - an invented CSV of layout B is read oldest first and adds up;
  - Windows-1252 and quotes are read;
  - any other CSV is refused;
  - a PDF and a CSV of the same account count their shared booking once, the CSV first, also after reading again, and the PDF's booking counts again without the CSV.
- The owner's two CSV files (copies, counts only): both taken at the door, read, and adding up to the cent, 315 bookings. They belong to accounts without PDF statements, so no booking was joined.
- A copy of the owner's archive migrated from version 10 to 12 with 0 broken foreign keys and its 132 documents.

## The balance of each account against the bank (#22)

- An account has one opening: a balance at the end of a day (`accounts.opening_day`, `opening_cents`). It is taken from the earliest statement of the account and moves back when an earlier one is read (`accounts::refresh_opening`, after every statement stored or removed). An opening the owner typed (`opening_source` `owner`) is never written over.
- `accounts::balance_on(account, day)` is the opening plus the bookings after its day up to the day, those another booking replaced left out. A booking of the earliest statement dated on the day of its old balance counts, its old balance does not hold it.
- `accounts::findings(account)` holds the archive against the bank:
  - for each statement, the counted balance at its end against the new balance it printed (`Differs`);
  - between two statements, a later start or another balance than the one the statement before ended with (`Gap`);
  - an account without an opening (`NoOpening`).

  Each finding has its German sentence. The counted balance at the end of a statement (`counted_through`) adds the bookings of every statement up to it, each with its own statement and not by day: the owner's bank dates some bookings with the closing day of the statement before, and counting them by day put them into that one (79 false differences on one account on the first run).
- Property tests: bookings in any order give the same balance, and a month cut into two statements gives the same as one, without a finding.

### Measured on 2026-11-26

The owner's archive (a copy, migrated and read again, counts only): all 3 accounts without a finding, each one's balance equal to the bank's last.
