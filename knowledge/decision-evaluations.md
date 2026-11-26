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
