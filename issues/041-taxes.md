# #41 Taxes for the return

**Goal:** What counts for the owner's own tax return is collected per year, ready to be entered in ELSTER, without advice.
**Requirements**: FA-44 · **Depends on**: #33, #24

## Done when

- [ ] A tax assignment on an invoice or a booking: a tax category, the share used for work, what someone refunded, the labour share for craftsmen. The amount that counts is computed, nothing is entered twice.
- [ ] The tax year is the year paid (§ 11 EStG): the day of the booking that paid it, else the invoice date with a hint.
- [ ] The values per year (Arbeitnehmer-Pauschbetrag, Entfernungspauschale, Homeoffice-Pauschale, § 35a, Spenden) in one table with their source and the day they were checked. A year without values sums without lump sums and says so.
- [ ] The owner's situation as in 1.0 (employed, a part-time master's degree two thirds paid by the employer, the return in ELSTER): only the own third of the study costs counts.
- [ ] A view "Steuern" per year: form and section ("Anlage N · Werbungskosten · Fortbildungskosten"), never line numbers; no refund computed; every place with tax content says the owner checks it himself.
- [ ] Tests with invented invoices and bookings.
