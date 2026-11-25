# Decision: the six UI sources of 1.0, a design system before the views

**Decided:** 2026-11-25 · applies to `src/`, every view, component, motion and frontend package

## Decision

- The six sources the owner named on 2026-11-11 for 1.0 apply to 2.0 too: pick-ui-library, Beautiful UI, RareUI, Magic UI, Aceternity UI and 21st.dev. Their links, licences and the rules for taking a piece are the vault skill `ui-libraries` (`agentos:ui-libraries`), which serves every project. Read it before any UI work.
- Code may be adapted from Magic UI, Beautiful UI and RareUI (MIT), with a header comment naming source and licence. From Aceternity UI and 21st.dev only the idea is taken, because this repository is public (`decision-public-repository.md`).
- A package comes only from the list of pick-ui-library, pinned exactly. The start set is the one 1.0 settled on: `motion` (through `LazyMotion` and `m`), `@number-flow/react`, `clsx`, `cmdk`. Sonner, base-ui, recharts, Virtuoso, cva, zustand and next-themes were left out in 1.0, and that stays until a piece needs one.
- Every motion stops under reduced motion of the system, and nothing covers a control.
- **The design system comes first** (#07). Colors, type, spacing, the surfaces and the base pieces are shown to the owner as variants at real size, the owner picks, and only then are views built.

## Why

- In 1.0 the look was rebuilt several times after views existed (#33, #34, #78, #107 there), and the owner found one version "zu bunt" and wanted "clean und elegant". 1.0 settled on the Apple look with system grays, the owner's accent and four surfaces (`decision-app-layout.md` there). That is the starting variant for #07, not a given.

## Not decided here

- The accent color, the fonts and whether 2.0 keeps the Apple look on Windows. The owner chooses in #07.
