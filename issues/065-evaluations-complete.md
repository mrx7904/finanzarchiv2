# #65 "Auswertungen" shows what the core already knows

**Goal:** The parties of a year, the whole salary and every savings hint stand in the window, each leading to its bookings.
**Requirements**: FA-65 · **Depends on**: #38, #39

Found by reading the code on 2026-11-27. The core computes more than the view shows. `evaluation_year` returns the parties of a year and nothing reads them, though FA-37 asks for sums "by category and by party". `salary_years` returns Brutto, every deduction and Netto per month, and the view draws the Netto only. Savings hints show as three lines of text that open nothing, the rest is cut off, and `dismiss_hint` is called nowhere, though FA-42 asks that each hint opens its bookings.

## Done when

- [ ] A section "Größte Empfänger" lists the parties of the year by their sum, and a section "Einnahmen von" those of the income. A row opens the bookings of that party in that year.
- [ ] The section "Gehalt" opens a page "Gehalt" under "Auswertungen": per month Brutto, each deduction and Netto, the sums of the year, a missing month named as missing, and the booking that paid a month with the difference when it differs. A month opens its payslip.
- [ ] A page "Spartipps" under "Auswertungen" lists every hint with its numbers. A hint opens the bookings it rests on, "Ausblenden" dismisses it with "Rückgängig" in the same place, and the section in "Auswertungen" shows the three most pressing with "Alle n".
- [ ] Each page says "keine Finanz- oder Anlageberatung" once where hints are shown (FA-42).
- [ ] The paths read "Auswertungen › Gehalt" and "Auswertungen › Spartipps", and the quick search finds both pages.
- [ ] Tests cover what the pages derive. `npm run check:views` opens both in light and dark with 0 findings.
- [ ] Seen by the owner on the real archive.

## Out of scope

- New kinds of hints: #39 owns them.
- The filters these jumps set: #64.
