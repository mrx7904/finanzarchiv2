# #47 The view "Prüfen" as designed

**Goal:** The owner answers what only the owner can decide, one question after the other, with the place on the page in sight.
**Requirements**: FA-49 · **Depends on**: #44, #45

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The pictures: `?v=pruefen-gruende`, `?v=pruefen-liste`, `?v=pruefen`, `?v=pruefen-leer`. What the questions are and how an answer is taken: [`knowledge/decision-checks.md`](../knowledge/decision-checks.md), unchanged.

## Done when

- [ ] The reasons as an overview with one sentence on what waits and "Der Reihe nach prüfen" in the top bar, which goes through every question of every reason.
- [ ] The documents of a reason as a list with the page of the chosen one beside it.
- [ ] The question: the page at the left reaching the lower edge of the window with the place marked, the question at the right as the large sentence, below it what is already sure, and at the foot which document comes next. The top bar says "Frage n von m" and holds "Überspringen".
- [ ] Every answer the view has today is kept: Ja, Nein with the readings and a typed value, the kinds, the documents to link, a duplicate with its dialog. The keys of `src/review/keys.ts` hold, and the question names them.
- [ ] Empty: "Nichts zu prüfen" with how much was filed (rule 14).
- [ ] Tests of `src/review/` pass and cover the order of "Der Reihe nach prüfen".
- [ ] Seen by the owner on the real archive, on Windows.

## Out of scope

- New questions or checks: stage 2 owns them.
