# #47 The view "Prüfen" as designed

**Goal:** The owner answers what only the owner can decide, one question after the other, with the place on the page in sight.
**Requirements**: FA-49 · **Depends on**: #44, #45

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The pictures: `?v=pruefen-gruende`, `?v=pruefen-liste`, `?v=pruefen`, `?v=pruefen-leer`. What the questions are and how an answer is taken: [`knowledge/decision-checks.md`](../knowledge/decision-checks.md), unchanged.

## Done when

- [x] The reasons as an overview with one sentence on what waits and "Der Reihe nach prüfen" in the top bar, which goes through every question of every reason.
- [x] The documents of a reason as a list with the page of the chosen one beside it.
- [x] The question: the page at the left reaching the lower edge of the window with the place marked, the question at the right as the large sentence, below it what is already sure, and at the foot which document comes next. The top bar says "Frage n von m" and holds "Überspringen".
- [x] Every answer the view has today is kept: Ja, Nein with the readings and a typed value, the kinds, the documents to link, a duplicate with its dialog. The keys of `src/review/keys.ts` hold, and the question names them.
- [x] Empty: "Nichts zu prüfen" with how much was filed (rule 14).
- [x] Tests of `src/review/` pass and cover the order of "Der Reihe nach prüfen".
- [ ] Seen by the owner on the real archive, on Windows.

## Out of scope

- New questions or checks: stage 2 owns them.

**Status:** built on 2026-11-27, waiting for the owner's look on the real archive.

- "Der Reihe nach prüfen" goes through the reasons in their order and asks each document once (`inSequence` in `src/review/keys.ts`, tested); it stands in the path as "Der Reihe nach". The top bar of a question says "Frage n von m" and holds "Überspringen", the foot names the next document.
- Every answer of #17 is kept unchanged: Ja, Nein with the readings and a typed value, the kinds, the documents to link, a duplicate with its dialog, and the keys of `reviewKey`.
- The page of a question is the full page in a column that scrolls, so its marked place stays exact; in the list of a reason the page fits its column.
- Seen in Chromium with the stand-in of the core at 1440 × 900: the overview, "Der Reihe nach prüfen", "Überspringen" to the last question.
