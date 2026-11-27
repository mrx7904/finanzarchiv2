# #41 Taxes for the return

**Goal:** What counts for the owner's own tax return is collected per year, ready to be entered in ELSTER, without advice.
**Requirements**: FA-44 · **Depends on**: #33, #24, #44, #45

The view follows the map and the rules of [`knowledge/decision-views.md`](../knowledge/decision-views.md), its picture is in `src/entwurf/` (#50).

## Done when

- [x] A tax assignment on an invoice or a booking: a tax category, the share used for work, what someone refunded, the labour share for craftsmen. The amount that counts is computed, nothing is entered twice.
- [x] The tax year is the year paid (§ 11 EStG): the day of the booking that paid it, else the invoice date with a hint.
- [x] The values per year (Arbeitnehmer-Pauschbetrag, Entfernungspauschale, Homeoffice-Pauschale, § 35a, Spenden) in one table with their source and the day they were checked. A year without values sums without lump sums and says so.
- [x] The owner's situation as in 1.0 (employed, a part-time master's degree two thirds paid by the employer, the return in ELSTER): only the own third of the study costs counts.
- [x] A view "Steuern" per year: form and section ("Anlage N · Werbungskosten · Fortbildungskosten"), never line numbers; no refund computed; every place with tax content says the owner checks it himself.
- [x] Tests with invented invoices and bookings.

**Status:** the core and the view are built on 2026-11-27 (`knowledge/decision-taxes.md`), waiting for the owner's look on the real archive.

- "Steuern" in the sidebar as a list with detail: the year in the top bar, the notice to check everything and where the values come from in one line, the sections by form and section (never a line number), the entries of a section with what counts and their notes, an overview with the Werbungskosten against the Pauschbetrag, the lump sums, the § 35a reductions and the tax profile of the year. No tax and no refund is computed.
- A document is assigned by "Für die Steuer …" in its menu (dialog `AssignTax` with category, share, refund, labour share, and "Keine Steuer"). A booking is assigned by "Für die Steuer …" in its detail in Umsätze (added on 2026-11-27 before 0.7.0).
- Seen in Chromium with the stand-in of the core; `npm run check:views`: 0 findings.
