# Decision: taxes as assignments, the year paid, values per year from checked sources

**Decided:** 2026-11-27 (#41, FA-44), taken over from 1.0 (`decision-tax.md` there, 2026-10-31) with the owner's situation unchanged (2026-11-26) · applies to `src-tauri/src/taxes.rs`, `src-tauri/src/commands/taxes.rs`, the tables `tax_assignments` and `tax_profiles` (migration 0025)

## Decision

- **An assignment, not a copy.** A document or a booking gets at most one tax assignment: a category, the share used for work (1 to 100 %), what someone refunded, and for craftsmen the labour share. What counts is `(amount − refunded) × share`, for craftsmen the labour share only, never below zero, rounded to the cent (`taxes::counted`).
- **One payment, one assignment.** A booking whose linked document is assigned is refused, and the other way round ("Die Zahlung ist schon über ihren Beleg zugeordnet."). Changing an assignment replaces it.
- **The tax year is the year paid** (§ 11 EStG): for a document the day of the first booking linked to it (#24), else its invoice date with the note "Jahr nach Rechnungsdatum, keine Zahlung gefunden"; for a booking its day. The amount of a document is its gross that counts (`checks::chosen`).
- **Twenty categories in five groups**, as in 1.0, each with form and section as ELSTER shows them as headings ("Anlage N · Werbungskosten · Fortbildungskosten"), never a line number.
- **Values per year in one place** (`taxes::year_values`) with their source: 2025 and 2026 checked on 2026-10-31 in 1.0, 2023 and 2024 with the values of 2025 and saying so (`taken_from`). A year without values sums without lump sums: no Entfernungspauschale, no Homeoffice, no comparison with the Pauschbetrag.
- **The tax profile per year** holds what no document shows: kilometres and days to work, an own car or not, what the employer paid towards the way, days at home. The Entfernungspauschale follows the year's rates (2025: 0,30 € to km 20 and 0,38 € from km 21, 2026: 0,38 € from km 1), capped at 4.500 € without an own car, less the employer's support, never below zero. The Homeoffice-Pauschale is 6 € a day, at most 1.260 €.
- **Sums, no tax.** Werbungskosten are the documents and the two lump sums, compared with the Arbeitnehmer-Pauschbetrag (what they are above it). Erstausbildung is capped at 6.000 €. The § 35a reductions are 20 % of their sections, capped at 4.000 € and 1.200 €. No refund, no zumutbare Belastung, no tax rate.
- **Notes never block:** a year from the invoice date, a craftsman without a labour share, a donation above 300 € that needs a Zuwendungsbestätigung.
- **The owner's study:** the employer pays two thirds tax free (§ 3 Nr. 19 EStG), so the fee is assigned to "Fortbildung und Studium" with those two thirds as refunded, and only the own third counts.
- **Never presented as settled.** Every tax result carries `taxes::CHECK_NOTICE` ("Keine Steuerberatung, bitte alles selbst prüfen."), and the view shows it in one line.
- **To the window:** `tax_categories`, `tax_years`, `tax_year`, `assign_tax`, `unassign_tax`, `set_tax_profile`. The view "Steuern" is built with the frame (#44, #45), after `decision-views.md`.

Left out of 1.0 for now, each a later step when the owner asks: the proposals of a category by words, the export as a table, the travel of a part-time study and the meal allowances, the Kontoführungspauschale, refunds as bookings that lower a later year (#68 there is still open on that rule).

## Measured on 2026-11-27

`taxes::tests` with invented invoices and bookings: share, refund and labour share, the owner's own third, the year paid against the invoice date, one assignment per payment, the lump sums of 1.0's check by hand (18 km × 150 days × 0,38 € less 120 € = 906,00 €, 60 days at home = 360,00 €), the two rates of 2025 and the cap without a car, a year without values, the § 35a reductions and their caps, a donation above 300 €, Erstausbildung capped, no line number in any category.

Nothing on the owner's archive yet: it holds no tax assignment until the owner makes one in the view.
