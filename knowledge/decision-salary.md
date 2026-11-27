# Decision: the salary comes from the payslips, a month without one is missing

**Decided:** 2026-11-27 (#35, FA-39) · applies to `src-tauri/src/salary.rs`, `src-tauri/src/commands/salary.rs`

## Decision

- **The filed payslips are the source.** Each amount is the value that counts for its field (`checks::chosen`: the owner's above the sender's own data, code, the text model and the image), never a mix of readers.
- **Nine amounts**, in this order and with these names: Brutto, Netto, Lohnsteuer, Solidaritätszuschlag, Kirchensteuer and the four social insurances (`salary::FIELDS`). A field a payslip does not print is absent, not zero.
- **By month and year.** A payslip counts in the month its period names (`YYYY-MM`). Two payslips of one month, two employers, are summed. A payslip without a period is left out of the months.
- **"fehlt", never zero.** Every month from the first payslip's to the last's is shown, a month without one is missing and has no amounts. Months before the first and after the last show nothing. A year counts its missing months.
- **Netto against the booking.** What the bookings linked to the month's payslips brought (#24, active links) stands beside the Netto, and a difference is named as the bookings minus the Netto.
- **To the window** as `SalaryYearRow` (`salary_years`): cents as `i32` and as German text. The section is part of Auswertungen (#38), not a view of its own (`decision-views.md`).

## Measured on 2026-11-27

`salary::tests` with invented payslips: the months of the range with one missing, the sums of a year and its missing count, the Netto against its booking with and without a difference, two employers in one month, a payslip not filed, the owner's value above the readers', no payslip at all.

A copy of the owner's archive (`examples/measure.rs --reader salary`, counts in `measurements/private/`): **no payslip is in the archive yet**, so no month. The money in that comes monthly is found by #34 from the bookings.

**Open:** measured again once the owner has imported payslips.
