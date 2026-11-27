# Decision: savings hints are observations by code, never advice

**Decided:** 2026-11-27 (#39, FA-42) · applies to `src-tauri/src/hints.rs`, `src-tauri/src/commands/hints.rs`, the table `hints_dismissed` (migration 0024)

## Decision

- **Five kinds, all by code**, each with its subject (a party or a category), one short line of what was seen, its figure, and the bookings (and documents) behind it:

  | Kind | Seen | Line |
  |---|---|---|
  | `rose` | a running recurring payment out (#34) whose last amount is above one paid at least twice before | "seit August 2026 2,00 € mehr, 143,88 € im Jahr" |
  | `twice` | two or more running payments out of different parties in "Abos und Medien", "Internet und Telefon" or "Freizeit und Hobby" | "2 laufend: …, zusammen 287,76 € im Jahr" |
  | `fees` | the bookings of "Bankgebühren" in the year of the last day the archive knows | "2026: 8,45 € in 2 Buchungen" |
  | `above` | a category of spending, from January to the last day known, at least a quarter and 100 € above the same days of the year before | "2026 bis März: 250,00 € statt 100,00 €" |
  | `still-runs` | a running payment with payments more than one rhythm after a letter of its party with "Kündigung" in its text, dated by its date read, else by the day it came | "nach der Kündigung vom 15.06.2026 noch 4× gezahlt" |

- **Observations, never advice.** A line says what the archive saw. No verb tells the owner what to do, a test holds that. "Versicherungen", "Kredit und Tilgung", "Sparen und Anlegen" and "Bargeld" are in no hint: investments, loans and insurance cover are no matter of a hint, and cash does not say what it bought.
- **The year compares with the same days** of the year before, because the archive's year is open until its statements come.
- **"Ausblenden" until the numbers change.** A dismissal keeps the hint's key (kind and subject) with the numbers it showed. When the numbers differ, the hint comes back.
- **Categories by name.** The hints name the default categories (`parties::DEFAULT_CATEGORIES`). A category the owner renames is no longer in the lists above. Known and accepted, a renamed category is rare and costs a hint, not a wrong number.
- **To the window** as `HintRow` (`list_hints`, `dismiss_hint`). The hints stand on Start and in Auswertungen (#40, #38).

## Measured on 2026-11-27

`hints::tests` with invented bookings: each kind with its line, figure and bookings, a rise of an insurance that is no hint, a booking of the year before outside the compared days, a dismissal that holds until a new booking changes the numbers, and no line with a word of advice.

A copy of the owner's archive after the categories of #43 (`examples/measure.rs --reader hints`, counts and a sample in `measurements/private/`): 3 hints, one `rose`, one `fees` and one `above`. Few, because most recurring payments of the archive ended with July 2026 (`decision-recurring.md`).

**Open:** the sample checked by the owner (`measurements/private/hints-sample.json`).
