# Decision: one SQLite file, checked before the app works on it, with snapshots beside it

**Decided:** 2026-11-25 (#02) · applies to `src-tauri/src/archive/`, `src-tauri/migrations/`, `src-tauri/src/money.rs`

## Decision

- The archive is `finanzarchiv.db` in the data folder of the app (Tauri `app_data_dir`, the identifier `de.mrx7904.finanzarchiv2`). `rusqlite` 0.43.0 with the features `bundled` and `backup`, so the SQLite of the app is the same on every system.
- WAL, `foreign_keys = ON`, `synchronous = NORMAL`. Every table is `STRICT`, a test fails on a table without it.
- Migrations are `src-tauri/migrations/NNNN_<name>.sql`, listed in `MIGRATIONS` of `archive/migrations.rs` with `include_str!`, numbered from 1 without gaps (tested). `PRAGMA user_version` holds the last one applied. Each runs in a transaction of its own.
- Before the app opens the file for writing, `PRAGMA quick_check` runs on it opened read-only. A file that fails is `Opened::Damaged` with its snapshots and is never written to. A missing file beside snapshots is `Opened::Missing` and no new archive is created over them. Both are lessons of 1.0, where a damaged archive looked empty and a missing one was silently replaced (#79 there).
- Snapshots go to `Sicherungen/` beside the file through SQLite's backup API, written as `.db.part` and renamed, named `finanzarchiv-<UTC YYYYMMDDTHHMMSSZ>-<reason>.db` (`taeglich`, `vor-migration`). One is taken when the newest is older than 24 hours, one before a migration changes an archive that already has a schema, none for a new archive. The newest ten are kept. The age comes from the name, not from the file time, so a copied folder keeps its order.
- Time comes from `jiff` 1.0.3 and is passed in, so the rules are tested without waiting.
- Amounts are `Cents(i64)`. `Cents::parse_de` reads German print ("1.234,56", "-12,50", "12,5", "1234", "49,99 €", the minus sign U+2212) and refuses "12.50" with the suggestion "12,50", because a guess could be off by a factor of a hundred. `Display` prints "1.234,56 €". A property test prints and reads every `i64` back to itself.

## A snapshot after a large import (#19)

**Decided:** 2026-11-25, found in the owner's acceptance of stage 1: the first 132 documents had no snapshot until the second start.

- When the intake queue becomes empty, a worker counts the documents imported since the newest snapshot. From `AFTER_IMPORT_THRESHOLD` = 20 on it writes a snapshot with the reason `nach-import` and keeps the newest ten. The count and the snapshot run under the lock of the archive, so two workers that finish together write one.
- Tested: 19 new documents write none, the twentieth writes one, a second call with nothing new writes none, and the workers write it by themselves after 25 documents.

## Evidence

On 2026-11-25: 17 tests of the core, among them a truncated file that is reported as damaged and stays byte for byte as it was, and twelve days of starts that leave ten snapshots. `cargo mutants --file src/money.rs` first left two survivors. One was a real gap ("12,+5" was read as 12,50 €), now refused and tested. The other was an arbitrary length limit, now replaced by checked arithmetic. The run after the fixes caught 60 of 60.

## Not decided here

- How the UI shows a damaged or missing archive and restores a snapshot. The state is held in the app (`Mutex<Opened>`), #03 brings it to the window.
