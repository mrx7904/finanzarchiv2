# #13 Payslips read by code

**Goal:** The code reader fills the values of a payslip: employer, month, Brutto, each deduction and Netto.
**Requirements**: FA-18 · **Depends on**: #11

## Done when

- [x] Brutto, Lohnsteuer, Solidaritätszuschlag, Kirchensteuer, the four insurances and Netto (or Auszahlungsbetrag) are read from their labelled lines, the deducted amount and never the rate (lesson of 1.0).
- [x] The month comes from the period line, the employer from the head of the page.
- [x] `npm run measure -- --reader code` reads every field of the 12 generated payslips right.
- [x] Measured once on the owner's payslips, counts only, in the commit.

## Out of scope

- Checking that the deductions add up, belongs to #16
- Payslips as income, belongs to stage 3
