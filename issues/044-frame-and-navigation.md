# #44 The frame of the views and how one moves

**Goal:** Every view stands in one frame with the sidebar of eight views, and the owner moves between them by the sidebar, the path, a jump and the arrow back.
**Requirements**: FA-47 · **Depends on**: #07, #26

The map and the rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The picture: `src/entwurf/Entwurf.tsx` (`Frame`), `?v=start`, `?v=dokumente-sprung`, `?v=start-eingang`, `?v=dokumente-suche`.

## Done when

- [x] The sidebar shows Start, Prüfen with its count, Dokumente, Konten, Auswertungen, Steuern and Chat in three groups without a title, and the intake and Einstellungen at its foot. A view that is not built yet is not shown, so no row leads to an empty page.
- [x] One navigation state without a routing package: the view, the place inside each view (the chosen row, the filters, the sub page) and a stack of the places a jump left. A view keeps its place when the owner comes back by the sidebar. Tests cover a change of view, a jump, the way back and a jump from a jump.
- [x] A jump opens its target in the target's own view. After it an arrow at the left of the path, and Alt+Left, lead back, and the arrow is gone when the stack is empty. The jumps that exist today go through it: a booking to its document, a document to its paired receipt.
- [x] The intake is one line at the foot of the sidebar, shown only while something is read, paused or refused. A click opens the jobs with their stage, the pause, and what was refused with its reason and "Ausblenden". The box "Eingang" above Dokumente is gone.
- [x] The quick search opens from every view by Strg+K (⌘K) and from its field in the sidebar. It finds views, documents, parties, bookings, the pages of Einstellungen in the owner's words, and runs every action that stands in a top bar or a menu (rule 22).
- [x] The line "Das Archiv ist bereit." and the version no longer stand above the views. A damaged or missing archive replaces every view, sidebar included, by one page with the sentence and the snapshots, each with "Wiederherstellen …" (`?v=archiv-beschaedigt`).
- [x] The actions of a view stand in the top bar (rule 2). `src/shell.test.ts` holds what the frame gives a view.

## Out of scope

- The views themselves: #46, #47, #48, #49. The pieces the frame needs: #45.
- Restoring a snapshot from the page of a damaged archive beyond what the core can do today: stage 6.

**Status:** done on 2026-11-27, with two limits:

- The page of a damaged or missing archive lists the snapshots without "Wiederherstellen …": the core cannot restore one yet (stage 6, see "Out of scope").
- The quick search runs the actions of the frame (back, pause the intake) and finds views, documents, accounts, parties and bookings. The pages of Einstellungen and the actions of the views come with #46 to #49, when those views put their buttons into the top bar.

Tests: `src/navigation.test.ts` (a change of view, a jump, the way back, a jump from a jump, the limit), `src/frame.test.tsx` (only built views, the arrow back, the quick search), `src/intake/describe.test.ts` (the line at the foot). Seen in Chromium: the page of an archive the core cannot open. **Not seen in the Tauri window:** the dev app would open the owner's real archive and migrate it past the installed 0.6.0, so the frame is seen with the next build the owner asks for.
