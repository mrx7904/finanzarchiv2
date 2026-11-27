# #64 The filters of "Umsätze": a month, a range, the direction, an amount

**Goal:** The owner narrows the bookings to what a question needs, and a figure of "Auswertungen" opens exactly the bookings it sums.
**Requirements**: FA-64 · **Depends on**: #48

As of 0.7.0 "Umsätze" filters by text, category and a whole year (`BookingFilter` in `src-tauri/src/commands/money.rs`). A month or a range is set only by a jump from "Auswertungen". No filter tells money in from money out, so "Einnahmen 2026" and "Ausgaben 2026" open the same list, and "Ohne Kategorie" opens nothing (`Evaluations.tsx`).

## Done when

- [ ] "Zeitraum" offers every year, every month of the chosen year, and "Von … bis …" with two dates.
- [ ] A filter "Richtung": "Eingänge", "Ausgänge", and "Umbuchungen" for the moves between own accounts.
- [ ] The category filter gains "Ohne Kategorie".
- [ ] The search finds an amount as typed ("34,12", "34") beside party and purpose.
- [ ] A figure of "Auswertungen" opens the bookings it sums: "Einnahmen" the incoming ones without moves, "Ausgaben" the outgoing ones, "Ohne Kategorie" the bookings without one. A test in the core proves for a generated year that the sum of the opened list equals the figure to the cent.
- [ ] Every filter set shows as a mark with "×" above the list, and "Alle Filter löschen" stands beside them when two or more are set.
- [ ] The line of counts and sums follows every filter.
- [ ] "Weitere laden" is replaced by loading while scrolling, and the count says how many of how many show.

## Out of scope

- The filter by document: #63. The jump to one booking: #58.
- Exporting the filtered list: #69.
