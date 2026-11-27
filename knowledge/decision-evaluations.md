# Decision: the evaluations count the bookings, once each

**Decided:** 2026-11-26 (#33, FA-37) · applies to `src-tauri/src/evaluations.rs`, `src-tauri/src/commands/evaluations.rs`

## Decision

- **The bookings are the money.** A year's income and spending come from the bookings of its days, nothing from the documents, so an invoice and the booking that paid it never count twice.
- **Once each.** A booking replaced by its CSV twin (#21) does not count, nor does a move between own accounts (#23, an active row of `booking_pairs`, made halves included).
- **The side of a booking** is its direction, with one turn: money in with a category of spending (its own or its party's) is a refund and lowers that category, money out with a category of income lowers that income (`evaluations::side_of`). It counts in the year the money moved. Without a category, money in is income and money out is spending.
- **By month, by category and by party**, each with its count of bookings, the larger first and the one without a name ("Ohne Kategorie", "Ohne Partei") last. The year before comes with it, so a year compares with the last.
- **Amounts to the window** as whole cents (`i32`) for the charts and as German text for reading, never as a float (`decision-money` of the stack).

## Measured

`evaluations::tests` with invented bookings: a salary, a purchase, its refund, a booking of unknown party, a move between two own accounts, a booking replaced by its CSV twin, a booking of the year before.

On a copy of the owner's archive, counts only (2026-11-26): for each of the seven years 2020 to 2026 income minus spending equals the plain sum of the bookings that count (1,730 bookings in all), and 194 halves of moves are left out. Every booking of spending falls into "Ohne Kategorie": no party of the owner has a category yet.

# The look of Auswertungen (#37)

**Decided:** 2026-11-27 by Fable 5.2 on the owner's behalf (the owner was away and asked for that), the owner can change it · applies to `src/evaluations/charts.tsx` and the view of #38

Three variants were drawn at real size in light and dark with invented numbers (`src/showcase/Evaluations.tsx`, the page "Auswertungen" of the showcase):

| Variant | What it was |
|---|---|
| **A Graphit** | rule 12 to the letter: money in in the ink, money out in a light gray, categories gray, the chosen mark in the accent; one column of sections |
| B Eine Farbe je Reihe | the first two hues of the validated dataviz palette, blue money in and orange money out, categories in blue (validated: all checks pass on #ffffff and #18181b) |
| C Wie 1.0 | the tiles and cards of 1.0's dashboard in Graphit |

**Picked: A**, with three changes: the savings rate under the balance ("36 % der Einnahmen, Januar bis Oktober") instead of a fourth tile; the bars of money out one step lighter in dark (`#52525b`); the line of the balances in the ink, 2 px, with the amount at its start beside its day as at its end.

Why, in short: A belongs to the rest of the app, which has no accent color and color only for a state; black against light gray separates money in and out as clearly as blue against orange. B's orange reads like the amber and red of the states, the "zu bunt" of 1.0. C boxes every section and makes the key figures smaller. A keeps the form "Overview" and its three figures are the largest things on the page.

The chart rules hold for every chart: plain HTML and SVG without a library, one y scale per chart, thin marks with 4 px round ends at the baseline and a 2 px gap, a missing month as "fehlt", text at its size at every width, the value of every mark on hover, and a year compared with the same months of the year before.
