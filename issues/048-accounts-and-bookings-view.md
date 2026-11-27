# #48 The views "Konten" and "Umsätze" as designed

**Goal:** The owner sees what is on every account at a glance and reads the bookings of a year in dense rows, each with its detail beside it.
**Requirements**: FA-49 · **Depends on**: #44, #45

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The pictures: `?v=konten`, `?v=umsaetze`, `?v=umsaetze-laden`, and Umsätze at 960 × 600. What a booking's detail holds was settled by the owner's look of 2026-11-26 (#26) and stays.

## Done when

- [ ] Konten: the sum over all accounts as the one large figure, how many accounts agree with the bank, then every account with its balance, the bank's balance with its day and "stimmt", "prüfen" or "ohne Bank". The top bar holds "Alle Umsätze" and "Kontoauszug importieren".
- [ ] Umsätze: dense rows grouped by month with the month's sums, newest first, with the search, the category and the period in the head and one line with the count and the sums. Below 1200 px a row drops purpose and category.
- [ ] The detail of the chosen booking stands beside the list and no longer folds open in the row: who, the amount, the day, the purpose, the category, the document as a card, the party over the year, a move, and the statement as a line that shows its place on demand.
- [ ] Money in carries a "+" and is not green (rule 13).
- [ ] A jump from a figure sets its filter as a mark with "×" (rule 9), ready for #38.
- [ ] The list stays quick with ten thousand bookings, measured as in #26. Tests of `src/money/` pass.
- [ ] Whether "Buchung von Hand" and renaming an account are offered in "…" is decided here with what the core can do.
- [ ] Seen by the owner on the real archive, on Windows.

## Out of scope

- The page "Wiederkehrend": #52. The list of categories: #49.
