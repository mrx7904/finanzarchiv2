# Decision: the six UI sources of 1.0, a design system before the views

**Decided:** 2026-11-25 · applies to `src/`, every view, component, motion and frontend package

## Decision

- The six sources the owner named on 2026-11-11 for 1.0 apply to 2.0 too: pick-ui-library, Beautiful UI, RareUI, Magic UI, Aceternity UI and 21st.dev. Their links, licences and the rules for taking a piece are the vault skill `ui-libraries` (`agentos:ui-libraries`), which serves every project. Read it before any UI work.
- Code may be adapted from Magic UI and Beautiful UI (MIT), with a header comment naming source and licence. From Aceternity UI and 21st.dev only the idea is taken, because this repository is public (`decision-public-repository.md`). RareUI changed its licence in 2026, see "RareUI is idea only" below.
- A package comes only from the list of pick-ui-library, pinned exactly. The start set is the one 1.0 settled on: `motion` (through `LazyMotion` and `m`), `@number-flow/react`, `clsx`, `cmdk`. Sonner, base-ui, recharts, Virtuoso, cva, zustand and next-themes were left out in 1.0, and that stays until a piece needs one.
- Every motion stops under reduced motion of the system, and nothing covers a control.
- **The design system comes first** (#07). Colors, type, spacing, the surfaces and the base pieces are shown to the owner as variants at real size, the owner picks, and only then are views built.

## Why

- In 1.0 the look was rebuilt several times after views existed (#33, #34, #78, #107 there), and the owner found one version "zu bunt" and wanted "clean und elegant". 1.0 settled on the Apple look with system grays, the owner's accent and four surfaces (`decision-app-layout.md` there). That was the starting variant for #07, not a given.

## The look: Graphit

**Decided:** 2026-11-25 by the owner, in the running dev app on Windows (#07)

Four variants were shown at real size in light and dark, each with a sidebar, a list, a preview, a form, a menu, a dialog, an empty state and the quick search, switched by a strip above the window:

| Variant | What it was |
|---|---|
| 1 Apple (1.0) | the system grays of macOS, white panels with a hairline, the blue of Apple, large bold titles, 13 px text |
| 2 Windows 11 | Segoe UI, a gray window like Mica, cards with a line, the blue of Windows, small radii, a blue mark at the chosen row |
| 3 Kontor | calm like a ledger: titles in a serif of the system, flat panels with lines, a deep petrol accent, dense rows |
| **4 Graphit** | no color accent: black buttons in light and white ones in dark, soft large radii, color only where a state needs it |

The owner picked **Graphit**, and answered two questions with it:

- **The same look on Windows and on the Mac.** One token file, only the system font differs (SF Pro on the Mac, Segoe UI on Windows). This settles "whether 2.0 keeps the Apple look on Windows" of the first version of this file: it keeps none of the two systems' looks.
- **Windows is enough for the showing.** The Done item of #07 asked for the variants on Windows and on the Mac. The owner decided that the view on Windows fulfils it and the Mac is looked at with #08 (releases).

Two notes of the owner on the picked look, carried out before the merge:

- The drawn invoice of the preview did not fit the window. It now fits its panel whole, by width or by height, whichever is smaller, with its type scaled in container units.
- The states of a row (prüfen, bestätigt, doppelt?, gelesen) were filled capsules of different widths. They are now one width for every state, neutral with a hairline and a colored dot, "für mehr Ruhe und visuelle Befriedigung", in the owner's words.

What Graphit sets, all in [`src/ui/tokens.css`](../src/ui/tokens.css):

- **Type:** the system font, 14 px body, 12 px small, 15 px headings, titles 24 px semibold with a tight tracking of -0.025 em.
- **Color:** zinc grays. The accent is black (`#18181b`) in light and near white (`#f4f4f5`) in dark, text on it inverts. Green, amber and red appear only for states.
- **Surfaces:** window `#fafafa` / `#0f0f11`, sidebar `#f4f4f5` / `#141417`, panels white / `#18181b`, what floats above white / `#222226`. Depth reads by brightness in dark, as in 1.0.
- **Radii:** control 8, row 10, panel 14, dialog 18 px. **Heights:** control 34, list row 56, sidebar 232 px.
- **Motion:** 140 ms for a color or a pop, 240 ms for something that moves, one ease-out curve.

### Why Graphit

The owner's words were "clean und elegant" in 1.0 and a dislike of "zu bunt". Graphit is the variant with the least color: the eye finds states because they are the only color on the screen. The owner did not give further reasons beyond the two notes above.

## How the tokens and the pieces work

- **One token file.** Every color is one `light-dark()` pair, so a token is one line and follows the color scheme of the element it is used on. `@theme inline` maps the tokens to Tailwind names (`bg-surface`, `text-fg-muted`, `rounded-panel`, `shadow-raised`), so no component picks a gray or a size of its own. `data-theme="light"` or `"dark"` on an element overrides the system's scheme below it.
- **Contrast checked by a test.** `src/ui/tokens.test.ts` reads the token file and checks every pair of a text color and a surface it stands on, in light and dark, for at least 4.5 to 1 (WCAG 2.2): 30 pairs, 60 checks. A text or surface token without its pairs fails the test. While the owner picked, the same test ran over all four variants (314 checks), and three colors were deepened until each passed.
- **Reduced motion stops everything.** `src/ui/motion.tsx` is the only file that imports Motion. Under reduced motion of the system it sets `MotionGlobalConfig.skipAnimations`, which makes every animation of Motion jump to its end, opacity and height included (Motion's own `reducedMotion="user"` stops transforms and layout only). It follows the setting while the app runs. For CSS, the duration tokens go to zero and a rule with `!important` stops every transition and animation, inline ones included. `src/ui/motion.test.ts` checks the gate, its listening to a change, that no other file imports Motion, and that no stylesheet times a transition without the tokens.
- **The dialog and the menu are native.** The dialog is `<dialog>` with `showModal()` (focus kept inside, the rest inert, Escape, focus returned), the menu a `popover`. Both lie in the top layer. Their entry is CSS with `@starting-style`. This is why base-ui stays out: the browser of both systems gives what base-ui would add (WebView2 is Chromium, WKWebView needs Safari 17.5 or newer for `light-dark()` and `@starting-style`).
- **The showcase stays** as `src/showcase/`, opened by "Gestaltung ansehen" in the dev build only, so a new piece is looked at in light and dark before a view uses it.

## What was taken from where

| Piece in `src/ui/` | Source | Licence | How |
|---|---|---|---|
| `motion.tsx`: `MotionRoot`, `spring`, `quick`, `cn` | pick-ui-library (github.com/emilkowalski/skills): motion 14.14.0, clsx 2.1.1 | MIT | packages |
| `QuickSearch.tsx` | pick-ui-library: cmdk 1.1.1. Rows and the empty answer from Beautiful UI "Search List". `Kbd` after the idea of "Kbd" by shadcn on 21st.dev | MIT. Beautiful UI MIT. 21st.dev licence listed as unknown, idea only | package, code adapted, idea |
| `Menu.tsx` | Beautiful UI "Glide Menu" (`components/primitives/GlideMenu.tsx`, fetched): the highlight that glides between rows | MIT | code adapted |
| `List.tsx`: `List`, `ListRow`, `Capsule` | Beautiful UI "Task Rows" (`TaskRows.tsx`, fetched): the row of badge, label, amount and capsule. The capsule redrawn after the owner's note | MIT | code adapted |
| `Field.tsx`: `TextField`, `SelectField` | Beautiful UI "Search List" (`SearchList.tsx`, fetched): the input row with leading icon and the round clear button | MIT | code adapted |
| `EmptyState.tsx` | Beautiful UI "Search List" (its empty answer), Magic UI "Blur Fade" (magicui.design/r/blur-fade.json, fetched, the blur left off), after the idea of Aceternity UI "Empty State With Stacked Cards" | MIT, MIT, Aceternity License (idea only) | code adapted, idea |
| `Dialog.tsx` and `components.css` | after the idea of Aceternity UI "Animated Modal": the backdrop fades in, the panel grows from .98. Its 3D tilt and bounce left out | Aceternity License (idea only) | idea |
| `Button.tsx`: `Button`, `Spinner` | after the idea of Aceternity UI "Stateful Button" (the busy state) and RareUI "Loading Spinner" (a still track and a turning arc) | Aceternity License, RareUI licence below, both idea only | idea |
| `AppShell.tsx`: `AppShell`, `TopBar`, the window buttons | none of the six sources has a window title bar. After the idea of the custom title bar of the Tauri guide (v2.tauri.app/learn/window-customization) and the caption buttons of Windows 11, see `decision-app-shell.md` | Tauri docs MIT, idea only | idea |
| `Path.tsx`: `PathNav`, `collapsePath` | none of the six sources has a path for a title bar. After the idea of the breadcrumbs listed on 21st.dev and the path bar of the Finder, the folded pages in the `Menu` (Beautiful UI "Glide Menu"), see `decision-app-shell.md` | 21st.dev per component, idea only | idea |
| `icons.tsx` | drawn for this app | none | own |

The Beautiful UI files were read at `raw.githubusercontent.com/slev12397/beautiful-ui/main` on 2026-11-25. Its licence: MIT, Copyright (c) 2026 Shane Levine. Each adapted file names it in its header.

The app has no "Über" view yet. The list of pieces and licences moves into the app with the settings view of a later stage, from this table.

## RareUI is idea only

**Found:** 2026-11-25 while taking pieces for #07

The repository of RareUI (`Codewithswappy/RareUI`) was renamed to `Codewithswappy/ShitUI`, and `rareui.in/docs/components/<name>` answers "Not Found". Its licence is now "MIT License + Commons Clause + Attribution": no selling of the components, and a visible credit with the name "ShitUI" and a link to its site in the README or the credits of every project that uses them. The vault skill `ui-libraries` still lists RareUI as MIT.

So no RareUI code enters this repository. Its ideas may still be taken, as with Aceternity UI. The skill in the vault needs the same correction, which belongs to the vault, not to this repository.

## Packages

Versions checked on 2026-11-25 at `registry.npmjs.org/<name>/latest`, pinned exactly in `package.json`:

| Package | Version | Released | Used by |
|---|---|---|---|
| `motion` | 14.14.0 | 2026-11-13 | `EmptyState` through `motion.tsx` |
| `clsx` | 2.1.1 | 2024-04-23 | every piece (`cn`) |
| `cmdk` | 1.1.1 | 2025-03-14 | `QuickSearch` |

`@number-flow/react` 0.6.2 is not installed yet: no piece of #07 shows a figure that changes. It comes with the first view that does (rule 4 of the skill: a package only when a piece needs it).

## Left out

- **base-ui:** the native `<dialog>` and `popover` give focus trap, Escape, light dismiss and the top layer.
- **cmdk's own dialog** (Radix): the quick search lies in the same `<dialog>` frame as every dialog, so there is one.
- **Sonner, recharts, Virtuoso, cva, zustand, next-themes:** no piece of #07 needs them. The variants of a button are four class strings.
- From the sources: the blur of Blur Fade (it smears text), the timed demo sequences and staggered entries of Task Rows, the 3D tilt of the Animated Modal, the neumorphic and glass looks of RareUI.
- The three variants not picked. They were overrides of the tokens in a file of the showcase and were removed with the pick. The table under "The look: Graphit" describes them, the Apple values are those of 1.0 (`decision-app-layout.md` there).
