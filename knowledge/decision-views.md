# Decision: one map of the views and rules every view follows

**Decided:** 2026-11-27 by the owner, on pictures at real size · applies to `src/App.tsx`, every view under `src/`, the sidebar, every dialog and menu, and `src/entwurf/`

## Decision

The views are designed as a whole before the frontend brings the functions together. The owner, in their words: "Wenn das soweit ist möchte ich aber, das alles passt." The draft, its map and its rules in German, is [`docs/entwurf-ansichten.md`](../docs/entwurf-ansichten.md). This file is what holds.

### The map

- **Eight views in the sidebar**, the same in every view, in groups parted by a gap and without a title: Start, Prüfen (with its count), then Dokumente, Konten, then Auswertungen, Steuern, Chat. At the foot stand the intake and Einstellungen. Above the groups stands the quick search (Strg+K, ⌘K on the Mac).
- **"Rechnungen" and "Zahlungen" of the showcase are no views.** An invoice is a document of the kind invoice, so a filter in Dokumente. Payments are the bookings in Konten.
- **Sub pages lie below a view and show in the path:** Prüfen › reason › document, Dokumente › Bestellungen, Konten › account, Konten › Wiederkehrend, Einstellungen › page.
- **Einstellungen is a view with pages**, a list at the left and the chosen page at the right: Ordner, Postfächer, Namen, Kategorien, Modell, Archiv, Darstellung, Über. What is set up once stands there and nowhere else. The four dialogs that hang on buttons in Dokumente and Konten today (Ordner, Postfächer, Namensvorlagen, Kategorien) move there. Stage 6 gets pages here, no view of its own.
- **The intake is seen from every view:** one quiet line at the foot of the sidebar, shown only while something is read, paused or refused. A click opens the jobs with their stage, the pause, and what was refused with its reason.
- **A healthy archive says nothing.** The line "Das Archiv ist bereit." and the version leave the views, the version stands in Einstellungen › Über. A damaged or missing archive replaces every view, sidebar included, by one page with the sentence and the snapshots.

### How one moves

- The sidebar changes the view. A view keeps where the owner was in it (filters, the chosen row) while the app runs.
- The path shows the place inside a view and leads back by a click. It never shows the way.
- A jump leads from a thing to the thing that belongs to it, and the target opens in its own view: a step on Start to where it is done, a figure to its bookings (filtered), a booking to its document and its statement, a document to the booking that paid it, a line of Steuern to its invoice or booking, a source under an answer of the chat to its document or booking, a problem of a folder or mailbox to its page in Einstellungen.
- After a jump an arrow at the left of the path, and Alt+Left, lead back to where the owner came from.
- **No routing package.** The app keeps its own navigation state: the view, the place inside it per view, and a stack of the places a jump left. This settles "No router yet" of [`decision-app-shell.md`](decision-app-shell.md): the app has one window, no address the owner sees and no page that loads, so a router would add a package for what a reducer does. `cmdk`, `clsx` and `motion` stay the only UI packages.

### The rules

The numbers are those of the draft, so a review can name one.

**The frame**

1. Every view stands in `AppShell`. No view draws a header, a title or a second navigation of its own.
2. The top bar carries the actions of the view: at most one primary button, at most one more, everything else in a menu behind "…". The buttons fit the bar of 47 px, and none stands there to fill it. In the content stand only actions of a row or of the chosen piece.
3. What is set up once stands in Einstellungen, never as a button in a view. A view may lead there when something is missing.
4. Content first. The content starts at the top left under the bar. At 1440 × 900 the list, the sum or the document starts after a quarter of the window's height at the latest, at 960 × 600 it is in sight without scrolling. Nothing is centered but an empty state.
5. No sentence above a view that explains it. Text explains only an empty state, an error or a question. The explanation of a figure stands behind a small "i" at the figure. The owner on 2026-11-26: "nicht so textlastig".

**Three forms, no more**

| Form | Layout | Views |
|---|---|---|
| List with detail | a list at the left, the chosen piece at the right | Dokumente, Umsätze, Bestellungen, Wiederkehrend, Prüfen (documents of a reason), Steuern, Chat, Einstellungen |
| Overview | one column of sections, up to about 1120 px wide | Start, Konten, Auswertungen, Prüfen (reasons) |
| Question | the page of the document at the left, the question at the right | Prüfen (question) |

- Three columns (list, page, values) hold when the chosen piece is a document with read values, as in the showcase of #07. Otherwise two.
- The window is at least 960 px wide. Below 1300 px the values move under the page, Start becomes one column, and a row of Umsätze drops purpose and category, which stay in the detail. Nothing is covered and nothing runs out of the window sideways.
- On a large screen lists, tables and the page of a document take the width, running text, forms and the sections of an overview do not. The page of a document reaches the lower edge of the window.
- Every column scrolls by itself, the bar and the sidebar stand still.

**Lists**

6. Every list is `List` with `ListRow`.
7. Umsätze use a dense row of 40 px, grouped by month with the month's sums in the group's head. It is a piece of the design system and is shown in the showcase first.
8. The detail of a row stands beside the list and never folds open inside it. Actions of a row appear at its right on hover or focus, and a row never changes its height.
9. Filters stand in the head of the list: one search field, at most two selects, below them one line with the count and the sums. A filter a jump has set stands there as a mark with "×".
10. The arrow keys move the choice, Enter opens, Esc goes one level back, in every list. The focus ring shows only when the owner comes by keyboard.

**States and amounts**

11. One vocabulary of states, the same everywhere and of one width: "prüfen" (amber, waits for the owner), "doppelt?" (red), "fehlt" (red, something expected is not there), "bestätigt", "stimmt" and "vollständig" (green), "gelesen", "läuft" and "beendet" (gray). A new word needs a reason in the change that adds it.
12. Color stands only for a state. No piece, no chart and no category gets a color for decoration. Charts are gray, the chosen bar is the accent.
13. Amounts stand right-aligned in tabular digits, in German notation with "€". Money in carries a "+", money out no sign. Money in is not green. This changes `src/money/Bookings.tsx`, where it is green today.

**Empty, loading, error**

14. Empty: `EmptyState` with one sentence and at most two actions, the first doing the first step.
15. Loading: what comes within 200 ms shows nothing. After that gray rows in the shape of the list stand, never a spinner in the middle of the window.
16. Error: a German sentence where the content would stand, with "Erneut versuchen". An error of a field stands at the field.
17. A problem of the archive replaces every view (see "The map").

**Dialogs, menus, feedback**

18. A dialog asks before something is removed or moved, asks for one short input, or lets the owner pick one thing. Lists are never managed in a dialog. Its title is the question, its button repeats the verb ("Löschen", never "OK"), and what goes and what stays stands below as a short list. Only a button that removes is red.
19. After an action the answer stands where it happened, with "Rückgängig" where there is one. No message opens at the edge of the window and goes by itself.
20. A button that asks something more first ends on " …". One that acts at once does not.
21. One word for one thing, the same in every view. Texts speak of what the app does for the owner, never of how it is built.
22. The quick search runs every action that stands in a top bar or a menu, and finds every page of Einstellungen in the owner's words.

### Fitting a view

Added on 2026-11-27 on the owner's question: "Wie siehts aus mit Usability? Elemente von der Größe her anpassen, ein/ausklappen, Fokusmodus?" Until then the draft said where things stand and almost nothing on how the owner fits a view. Five proposals were made, the owner took all five ("Ja, zeichne 1 bis 5 und trag sie ein").

23. The line between two columns can be dragged, by mouse and by keyboard, and a double click gives the width of the design back. No column gets narrower than its least width.
24. The sidebar folds to a rail of icons of 56 px by its button and by Strg+B. The count of "Prüfen" and the intake stay in sight.
25. The values of a document and the detail of a booking fold away by a button of the top bar, which stays pressed while the column shows. The states of the rows of Dokumente can be hidden too, as the last entry of the filter of the states, "Zustand ausblenden", below a line that parts it from the states to filter by. The owner on 2026-11-27: "Zustand möchte ich auch ausblenden können. Am besten einfach als Punkt beim Filter der Zustände unter Dokumente". While they are hidden the filter reads "Ausgeblendet", a row gives its room to the name, and the line of counts still says how many wait. A filter by one state and hidden states exclude each other, which the one control makes plain.
26. Under the page of every document stand its pages and its zoom, and the document can be read large over the whole window, with its pages at the left. In "Prüfen" a focus hides everything but the page, the question and the progress. Esc ends both.
27. The whole view has three sizes, Klein, Mittel and Groß (90, 100, 115 %), in Einstellungen › Darstellung and by Strg with +, − and 0. It is the zoom of the window, so a view folds by the width it then really has.

What the owner set is kept over a restart, on the computer and not in the archive.

Turning pages and zooming closed a gap of the draft: it showed only the first page of a document at one size, though invoices and statements have several pages.

## What the owner decided with it

The draft asked nine questions, each with a proposal. The owner on 2026-11-27 on the proposals: "Deine Vorschläge gefallen mir aber soweit", then on the nine views drawn with them: "Gefällt mir sehr gut".

| Question | Decided |
|---|---|
| "Konten" or "Geld" | "Konten" |
| Bestellungen under Dokumente or Konten | under Dokumente |
| Kategorien in Einstellungen or in Konten | in Einstellungen. The category of one booking is still chosen at the booking |
| Money in green or with "+" | "+" without color |
| An arrow back after a jump | yes |
| Recurring payments | a page under Konten and a section in Auswertungen |
| Chat as a view or as a field over every view | a view |
| Einstellungen as a view or as a wide dialog, as in 1.0 | a view |
| Gehalt as a section of Auswertungen or as a view, as in 1.0 | a section |

## Why

- 1.0 rebuilt its look several times after views existed, and functions added on top of a view pushed its content off the screen (`decision-app-layout.md` there, "Content first"). 2.0 has the design system of #07, chosen by the owner, but until now no view was designed as a whole: the three built views put the pieces together while their functions arose, with six buttons in a row above Dokumente and settings as dialogs behind buttons.
- Twelve wishes of the owner on the running 1.0 are rules here from the start. The draft lists each with its rule ("Was 1.0 schon gelernt hat").
- 1.0 is a source of lessons and no limit. The owner on 2026-11-27: "Lass dich nicht zu weit von Version 1.0 eingrenzen."

One point departs from 1.0 knowingly. There the actions of a view stood in the top strip for a few hours, and the owner found the buttons too large for it and placed there only to fill it. For 2.0 the owner decided on 2026-11-25 that the top bar carries the page title and the buttons ([`decision-app-shell.md`](decision-app-shell.md)). Rule 2 follows the newer decision and keeps the lesson.

## What was shown

`src/entwurf/` draws the views from the pieces of `src/ui/` with invented data and without the core, as a page of its own in the dev server (`/src/entwurf/index.html`, without `?v=` it lists every view). Nothing imports it, so it is in no build and no release.

| File | What it draws |
|---|---|
| `Entwurf.tsx` | the nine views: Start, Prüfen (question), Dokumente, Konten, Umsätze, Auswertungen, Steuern, Chat, Einstellungen › Postfächer |
| `Unterseiten.tsx` | Prüfen (reasons, documents of a reason), Bestellungen, Wiederkehrend, the seven other pages of Einstellungen, and the states: empty, loading, error, damaged archive, the intake opened, the quick search, the arrow back |
| `Anpassen.tsx` | rules 23 to 27: a column dragged wider, the rail, a column folded away, the states hidden by the filter, the focus of Prüfen, a document read large, the sizes Groß and Klein |
| `Dialoge.tsx` | seven dialogs: deleting a document, renaming, picking the invoice of an order, restoring a snapshot, moving the archive, removing a mailbox, no longer watching a folder |

Seen by the owner as pictures on 2026-11-27: every view, sub page, state and dialog at 1440 × 900 in light, a part of them in dark, Start, Dokumente and Umsätze at 960 × 600, and the four menus. All of them were rendered in Chromium in light and dark.

**The drawn views are the picture a view is built after, not its code.** They hold no state and call nothing. They stay until the last view is built and go with #51.

Auswertungen is drawn only as a place: its sections and that every figure opens its bookings. How its charts look is chosen by the owner from variants (#37), within rule 12.

## Found in the pieces while drawing

Not changed in the draft, they are the work of #45:

- `Button` takes no `ref`, so it cannot be the trigger of a `Menu`. The draft uses a plain button of the same look.
- A dialog puts the focus on its close button first. It belongs on the safe action or on the field.
- A long entry stands close to the right edge of its menu.
- There is no switch, no segmented choice, no dense row, no row of gray shapes for loading and no row of a read value with its source. The draft draws each for itself.

## Known limits

- **The sizes were not seen as a zoom of a window.** The pictures of Groß and Klein were taken with a smaller and a larger window at a higher and a lower density, which is what a zoom does. How Tauri zooms a webview is to be verified in #53.
- **UNVERIFIED in the app:** everything was seen in Chromium, nothing in the Tauri window, on Windows or on the Mac. WebView2 is Chromium. WKWebView is not, and the Mac is still unseen for the design system too (#08).
- The owner saw pictures, not a running app. A view is accepted when the owner has used it on the real archive, as every stage is (`spec/concept.md`, "One open stage").
- Drawn but not asked of the core: "Neu lesen lassen" at a document, "Buchung von Hand" in Umsätze, "Braucht keine Rechnung" at an order, "Beim Start öffnen" in Darstellung, "Als Tabelle exportieren". Each is named in the issue of its view as something to decide there, none is promised by this file.

## Rejected

- **Building the views first and designing afterwards.** That is what 1.0 did.
- **Einstellungen as a wide dialog**, as in 1.0: the path could not show where the owner is, and rule 18 keeps lists out of dialogs.
- **A routing package** (see "How one moves").
- **Titles over the groups of the sidebar**, as in 1.0 ("Überblick", "Finanzen", "Eingang"): eight views read without them, and a label above content that needs none is what the skill `frontend-design` names as a mark of a templated page.
- **Folding the sections of an overview** (what was filed on Start, the categories of Auswertungen): it makes a calm view restless, and an overview is short enough to scroll.
- **A fold at 1200 px wide:** with the size Groß a window of 1440 px is 1252 px wide, and three columns left the page 145 px. The fold stands at 1300 px, and #53 holds the page at 320 px at least.
- **Toasts** for feedback (Sonner stays out): in 1.0 one covered the actions of a row.
