# #52 The page "Wiederkehrend" under Konten

**Goal:** The owner sees every recurring payment with what it costs in the month and what changed.
**Requirements**: FA-49 · **Depends on**: #34, #48

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The picture: `?v=wiederkehrend`. What a recurring payment is: #34.

## Done when

- [ ] "Konten › Wiederkehrend": the sum in the month and in the year, then every payment with its rhythm, its next day, its amount and "läuft", "gestiegen", "doppelt?" or "beendet". "gestiegen" joins the vocabulary of rule 11 with this change.
- [ ] Beside the list the chosen payment: its amounts over the last twelve months, the next expected day, the sum in the year, its category, "Buchungen zeigen" as a jump to Umsätze, and "Ist keine wiederkehrende Zahlung", which stays (#34).
- [ ] Konten leads to the page with the count and the sum in the month.
- [ ] Tests cover what the page derives for a row. Seen by the owner on the real archive.

## Out of scope

- The section in Auswertungen: #38. Savings hints from it: #39.
