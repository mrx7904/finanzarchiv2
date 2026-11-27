# #74 A field that chooses from a list looks like the app

**Goal:** The opened list of every filter and every choosing field is drawn by the design system, in light and dark, and no longer by the system.
**Requirements**: FA-74 · **Depends on**: #45

Reported by the owner on 2026-11-27 ("die Dropdowns sehen aktuell noch schrecklich aus bei den Filtern"). The cause: `SelectField` (`src/ui/Field.tsx`) is a native `<select>` with its own box. The closed field follows the tokens, the opened list is drawn by the webview and the system: its own font, row height, colors and corners, a hard edge, and on Windows a look that does not follow the dark scheme of the app. The filter "Zustand" of Dokumente fakes a dividing line with a disabled entry of dashes. The menu of the app (`src/ui/Menu.tsx`) already has the look that is missing here.

## Done when

- [ ] To decide first with the skills `frontend-design` and `ui-libraries` and record in `knowledge/decision-ui-libraries.md`: the list is built on the popover of `Menu.tsx` or on another piece of the six sources. The owner picks from two variants at real size in the showcase, in light and dark.
- [ ] The opened list has the surface, the corners, the shadow, the type and the row height of the menu. The chosen entry carries a check mark, the entry under the pointer or the keys the hover of a row, and a count stands at the right in tabular figures ("prüfen · 12" becomes the name and its count).
- [ ] A real dividing line and a group title are parts of the piece. The entry of dashes in Dokumente is gone.
- [ ] The list opens under its field, as wide as the field at least, turns upward when there is no room below, never leaves the window and scrolls inside when it is longer than 320 px.
- [ ] The keys work as in a native field: Enter, the space bar and the arrow keys open it, the arrows move, typing letters jumps to the entry that begins with them, Enter chooses, Esc closes without a change, Tab closes and moves on. The focus returns to the field.
- [ ] A screen reader hears a combobox with its list, its options and the chosen one (`role` and `aria` as in the ARIA Authoring Practices, "Select-Only Combobox", read on the day of the work).
- [ ] Every use of `SelectField` in `src/` shows the new list without a change to its call: the filters of Dokumente and "Umsätze", the category of a booking, the pages of Einstellungen, "Steuern". No native `<select>` is left in `src/`, and a rule of `npm run check:views` fails on one.
- [ ] The showcase shows the piece closed, open, with a line, with counts, with a disabled entry and with 40 entries, in light and dark.
- [ ] Tests in `src/ui/pieces.test.tsx` cover the keys, the entry found by typing, the disabled entry and the turn upward. `npm run check:views` opens a filter of Dokumente and of "Umsätze" in light and dark with 0 findings.
- [ ] Seen by the owner in the installed app on Windows, light and dark.

## Out of scope

- New filters of "Umsätze": #64, which uses this piece.
- A field that searches while typing (a combobox with text): a piece of its own when a list grows beyond what scrolls well, for example the parties of #66.
