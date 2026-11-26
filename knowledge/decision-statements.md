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
