# #51 A check that every view keeps the rules

**Goal:** A command proves for every view, in light and dark and from the narrowest to a wide window, what the rules promise, so a function added later cannot push a view out of shape unseen.
**Requirements**: FA-52 · **Depends on**: #46, #47, #48, #49

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). 1.0 had such a check (`check:controls` there) and found with it a message covering the actions of a row.

## Done when

- [x] `npm run check:views` opens every view and sub page with invented data in light and dark at 960 × 600, 1440 × 900 and 2560 × 1440 and fails when: something runs out of the window sideways, a control is covered, the content starts below a quarter of the height at 1440 × 900 (rule 4), a top bar holds more than one primary button (rule 2), or a state word is not one of rule 11.
- [x] It runs the views of the app, not the drawn ones, against a core that is replaced at the typed boundary of #03. To be verified in this issue: how the generated bindings are replaced in a browser without Tauri.
- [x] Its self-test plants one fault per rule and fails on each, as `scripts/check.mjs --self-test` does. Whether it joins the check of the repository or stays a command of its own is decided by its run time.
- [x] Empty, loading and an error of every list are among the states it opens.
- [x] `src/entwurf/` is removed once every view it draws is built, and [`docs/entwurf-ansichten.md`](../docs/entwurf-ansichten.md) says so.

## Out of scope

- The look on the Mac: #08.

**Status:** done on 2026-11-27. The check is built, and `src/entwurf/` is removed once every view it drew was built (up to commit c4e63c9 in the history of git).

- The bindings are replaced below themselves: `src/dev/mock.ts` sets `window.__TAURI_INTERNALS__`, which `invoke` of `@tauri-apps/api` calls, so the generated `commands` run unchanged against invented answers. It is loaded with `?mock` in the dev server only; `?mock=empty`, `?mock=error` and `?mock=slow` answer the lists empty, with an error, or after 1.5 s. A view is opened by the URL (`navFromUrl`, dev build only).
- `scripts/check-views.mjs` starts its own dev server and the Edge or Chrome of the system headless and talks to it over the DevTools protocol with the WebSocket of Node, so no package is added. 14 views and sub pages × 3 sizes × light and dark, and 5 lists × empty, error and loading: 99 pages in 93 s.
- **It stays a command of its own** (`npm run check:views`, `-- --self-test`): 93 s would more than double the check of the repository, and it needs a browser of the system. It runs before a view is merged.
- The self-test plants one fault per rule into Dokumente (a block 4000 px wide, a box over a control, a space of 400 px above the content, two more primary buttons, the word "erledigt") and finds each.
- Found on its first run and fixed: the error of Dokumente stood in the middle of the window, rule 16 wants it where the content would stand. An empty state is the one thing in the middle (rule 4) and is left out of the rule of the content's start. "ohne Bank" stands beside the words of rule 11, added by #48 for an account without a balance of its bank.
