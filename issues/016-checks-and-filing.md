# #16 Checks decide what is filed by itself

**Goal:** A document that passes every check is filed without the owner, with "Warum?" and "Rückgängig", and every other one waits with its reasons.
**Requirements**: FA-21 · **Depends on**: #12, #13, #14

## Done when

- [x] Checks with a stable id each: the decisive values stand in the text (or the XML), net plus VAT is gross to the cent and the rate fits, a payslip's Brutto minus its deductions is its Netto, two independent readers agree on each decisive amount, no other document has the same sender, number and gross.
- [x] The stage `checked` stores each result in `field_checks` and moves the job to `filed` or `waiting`. A filed document has a row in `decisions` with the rule, the values it saw and how to undo it.
- [x] Property tests over the arithmetic checks. Every check shown able to fail.
- [x] The preview of a filed document shows "Warum?" (the checks it passed) and "Rückgängig" (it goes back to waiting).
- [x] Measured on the owner's 132 documents: how many are filed, how many wait and why, counts only in the commit.

## Out of scope

- The view where waiting documents are answered, belongs to #17
