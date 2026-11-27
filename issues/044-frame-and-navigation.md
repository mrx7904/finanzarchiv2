# #44 The frame of the views and how one moves

**Goal:** Every view stands in one frame with the sidebar of eight views, and the owner moves between them by the sidebar, the path, a jump and the arrow back.
**Requirements**: FA-47 · **Depends on**: #07, #26

The map and the rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The picture: `src/entwurf/Entwurf.tsx` (`Frame`), `?v=start`, `?v=dokumente-sprung`, `?v=start-eingang`, `?v=dokumente-suche`.

## Done when

- [ ] The sidebar shows Start, Prüfen with its count, Dokumente, Konten, Auswertungen, Steuern and Chat in three groups without a title, and the intake and Einstellungen at its foot. A view that is not built yet is not shown, so no row leads to an empty page.
- [ ] One navigation state without a routing package: the view, the place inside each view (the chosen row, the filters, the sub page) and a stack of the places a jump left. A view keeps its place when the owner comes back by the sidebar. Tests cover a change of view, a jump, the way back and a jump from a jump.
- [ ] A jump opens its target in the target's own view. After it an arrow at the left of the path, and Alt+Left, lead back, and the arrow is gone when the stack is empty. The jumps that exist today go through it: a booking to its document, a document to its paired receipt.
- [ ] The intake is one line at the foot of the sidebar, shown only while something is read, paused or refused. A click opens the jobs with their stage, the pause, and what was refused with its reason and "Ausblenden". The box "Eingang" above Dokumente is gone.
- [ ] The quick search opens from every view by Strg+K (⌘K) and from its field in the sidebar. It finds views, documents, parties, bookings, the pages of Einstellungen in the owner's words, and runs every action that stands in a top bar or a menu (rule 22).
- [ ] The line "Das Archiv ist bereit." and the version no longer stand above the views. A damaged or missing archive replaces every view, sidebar included, by one page with the sentence and the snapshots, each with "Wiederherstellen …" (`?v=archiv-beschaedigt`).
- [ ] The actions of a view stand in the top bar (rule 2). `src/shell.test.ts` holds what the frame gives a view.

## Out of scope

- The views themselves: #46, #47, #48, #49. The pieces the frame needs: #45.
- Restoring a snapshot from the page of a damaged archive beyond what the core can do today: stage 6.
