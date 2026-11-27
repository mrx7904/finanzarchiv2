# #40 The start view

**Goal:** The owner sees at first glance what waits and does it with one click.
**Requirements**: FA-43 · **Depends on**: #38, #39, #44, #45

The view follows the map and the rules of [`knowledge/decision-views.md`](../knowledge/decision-views.md), its picture is in `src/entwurf/` (#50).

## Done when

- [x] "Start" first in the sidebar: one sentence on what waits, then "Als Nächstes", the steps most pressing first, each with one button that does it.
- [x] Steps from the archive, derived and never stored: documents in "Prüfen", orders without an invoice, a mailbox or watched folder with a problem, an account with a gap between its statements, the local model out of reach, a savings hint.
- [x] Beside them a few key figures of the running year (income, spending, balance), each opening its evaluation.
- [ ] Seen by the owner on the real archive.

**Status:** built on 2026-11-27, waiting for the owner's look on the real archive.

- `src/start/steps.ts` derives the steps each time, never stored, the most pressing first: the local model out of reach, a folder or a mailbox with a problem, questions in Prüfen, an account whose findings name a missing statement, orders without an invoice (not those that need none), up to two savings hints; tested. Each step has one button to where it is done.
- The key figures of the newest year with bookings, each opening Auswertungen.
- The app still opens with Dokumente: the view it opens with is decided when the owner asks (#49, out of scope).
- Seen in Chromium with the stand-in of the core; `npm run check:views`: 0 findings.
