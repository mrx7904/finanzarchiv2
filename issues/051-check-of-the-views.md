# #51 A check that every view keeps the rules

**Goal:** A command proves for every view, in light and dark and from the narrowest to a wide window, what the rules promise, so a function added later cannot push a view out of shape unseen.
**Requirements**: FA-52 · **Depends on**: #46, #47, #48, #49

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). 1.0 had such a check (`check:controls` there) and found with it a message covering the actions of a row.

## Done when

- [ ] `npm run check:views` opens every view and sub page with invented data in light and dark at 960 × 600, 1440 × 900 and 2560 × 1440 and fails when: something runs out of the window sideways, a control is covered, the content starts below a quarter of the height at 1440 × 900 (rule 4), a top bar holds more than one primary button (rule 2), or a state word is not one of rule 11.
- [ ] It runs the views of the app, not the drawn ones, against a core that is replaced at the typed boundary of #03. To be verified in this issue: how the generated bindings are replaced in a browser without Tauri.
- [ ] Its self-test plants one fault per rule and fails on each, as `scripts/check.mjs --self-test` does. Whether it joins the check of the repository or stays a command of its own is decided by its run time.
- [ ] Empty, loading and an error of every list are among the states it opens.
- [ ] `src/entwurf/` is removed once every view it draws is built, and [`docs/entwurf-ansichten.md`](../docs/entwurf-ansichten.md) says so.

## Out of scope

- The look on the Mac: #08.
