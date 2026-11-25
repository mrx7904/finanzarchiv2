# #02 The archive database and exact money

**Goal:** The core opens one SQLite archive file with strict tables, embedded migrations, a check before work and snapshots, and every amount is a `Cents` value.
**Requirements**: FA-04, FA-05 · **Depends on**: #01

## Done when

- [x] `rusqlite` with `bundled` opens the archive in the data folder of the app, WAL on, foreign keys on, every table `STRICT`.
- [x] Migrations are numbered SQL files embedded in the binary. A test applies them to an empty file and a second run changes nothing.
- [x] Before the app works on the file, `PRAGMA quick_check` runs on it. A damaged file is shown as damaged with its snapshots, never as an empty archive (lesson of 1.0, #79 there), proved by a test with a truncated file.
- [x] A snapshot is written with the backup API once a day and before a migration, the newest ten are kept, tested with a temporary folder.
- [x] `Cents(i64)` parses German printed amounts ("1.234,56", "-12,50", "12,5") and formats them. A `proptest` property shows that formatting then parsing returns the same value for generated amounts, and the parser refuses "12.50" with a reason.
- [x] `cargo-mutants` on the money module leaves no survivor, or each one is named as equivalent in the commit.

## Out of scope

- Tables of documents and bookings, they come with the issues that use them (#04, #05 and stage 3)
