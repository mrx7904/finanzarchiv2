# #50 The views of stage 5 in the frame

**Goal:** Start, Auswertungen, Steuern and Chat are built after the map and the rules, so no view of stage 5 is designed by itself.
**Requirements**: FA-51 · **Depends on**: #44, #45, #51

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The pictures: `?v=start`, `?v=auswertungen`, `?v=auswertungen-fehler`, `?v=steuern`, `?v=chat`. This issue adds no view. It binds #38, #40, #41 and #42 to the decision and checks them against it.

## Done when

- [x] #38, #40, #41 and #42 name [`knowledge/decision-views.md`](../knowledge/decision-views.md) and depend on #44 and #45. Done with the cut of this issue.
- [x] Start (#40) is an overview: one sentence on what waits, the steps with one button each, what was filed without the owner, and the figures of the year beside them, below 1300 px under them.
- [x] Auswertungen (#38) is an overview with the year in the top bar, charts in gray with the chosen bar in the accent (rule 12), in the look #37 chose.
- [x] Steuern (#41) is a list by form and section with the documents of the chosen line beside it.
- [x] Chat (#42) is a list of conversations with the conversation beside it, the sources under an answer as jumps.
- [x] Each of the four passes the check of #51.

## Out of scope

- What the four views compute and show: #38, #40, #41, #42.

**Status:** done on 2026-11-27, waiting for the owner's look with the views of #38, #40, #41 and #42.

- Start: the sentence, the steps with one button each, the documents filed in the last seven days (up to five, each opening its document), the figures of the year beside them and under them below 1300 px.
- Auswertungen: an overview in the look #37 chose, the year in the top bar. Steuern: a list by form and section with the entries of the chosen line beside it. Chat: the conversations with the chosen one beside them, the sources as jumps.
- `npm run check:views` covers all four (0 findings over 111 pages).
