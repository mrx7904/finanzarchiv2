# #45 The pieces the views need

**Goal:** The design system holds every piece the drawn views use, each seen in the showcase in light and dark before a view uses it.
**Requirements**: FA-48 · **Depends on**: #07

Found while drawing the views: [`knowledge/decision-views.md`](../knowledge/decision-views.md), "Found in the pieces while drawing". The rule that a piece is shown first: [`knowledge/decision-ui-libraries.md`](../knowledge/decision-ui-libraries.md). Read the vault skill `ui-libraries` before adding one.

## Done when

- [ ] A dense row of 40 px for long lists and a head of a group that stays at the top while its rows scroll (rule 7), with the arrow keys of `List`.
- [ ] A switch and a segmented choice, both reachable and changed by keyboard, with their name for a screen reader.
- [ ] Gray rows in the shape of a list for loading, shown only after 200 ms (rule 15), still under reduced motion.
- [ ] A row of a read value: its name, the value, and where it stands (known, checked, by the sender) as a dot and a word.
- [ ] The list of what goes and what stays for a dialog that removes or moves (rule 18).
- [ ] `Button` can be the trigger of a `Menu`. A dialog puts the focus on its field, or on its safe action when it has none, never on the close button. A long entry of a menu keeps the menu's padding at its right.
- [ ] A filter set by a jump as a mark with "×" (rule 9), and a row whose actions appear on hover or focus without changing its height (rule 8).
- [ ] Every text and surface the pieces add passes the contrast test of `src/ui/tokens.test.ts`, and no stylesheet times a motion without the tokens (`src/ui/motion.test.ts`).
- [ ] Each piece stands in the showcase, and its row in the table "What was taken from where" names its source and licence.

## Out of scope

- Charts: #37 chooses their look.
- A package for long lists (Virtuoso): only when a list is measured as slow, with its own row in the decision.
