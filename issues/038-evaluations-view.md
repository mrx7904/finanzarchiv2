# #38 The view "Auswertungen"

**Goal:** The owner sees in one view what the year brought, and every figure opens what it rests on.
**Requirements**: FA-41 · **Depends on**: #37, #44, #45

The view follows the map and the rules of [`knowledge/decision-views.md`](../knowledge/decision-views.md), its picture is in `src/entwurf/` (#50).

## Done when

- [x] "Auswertungen" in the sidebar, in the look of #37, for the year the owner picks.
- [x] Every bar, slice and figure opens the bookings behind it in "Umsätze", filtered.
- [x] Light and dark, from the narrowest to the widest window, no control covered, every motion stops under reduced motion.
- [ ] Seen by the owner on the real archive.

**Status:** built on 2026-11-27, waiting for the owner's look on the real archive.

- In the look of #37 (variant A): the three key figures, money in and out by month, spending by category, the balance of all accounts, the Netto of the payslips with "fehlt", the recurring payments with up to three savings hints. The year in the top bar.
- The year before is compared over the same months (`sameMonths`), so a year still open does not stand against a whole one. A month after the last booking shows no bar; a month outside the payslips shows nothing, one inside them without a payslip "fehlt" (#35).
- A month opens Umsätze filtered to it, a category to it in that year, a key figure to the year, each as a mark with "×" (`BookingJump`); "Wiederkehrend" opens its page under Konten. The arrow back leads to Auswertungen.
- `src/evaluations/derive.ts` holds what the view derives, tested. Seen in Chromium with the stand-in of the core; `npm run check:views`: 0 findings over 105 pages.
