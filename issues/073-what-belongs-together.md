# #73 What belongs together, as one picture

**Goal:** At a document or a booking the owner sees in one picture everything the archive joined to it, and reaches each part with a click.
**Requirements**: FA-73 · **Depends on**: #58, #66

Asked by the owner on 2026-11-27 (a canvas "wie in Figma", answered with a yes to this picture and to dragging the page in #56). As of 0.7.0 the joins of the archive show one step at a time: a document names the booking that paid it, the booking names its statement and its party, a receipt names its invoice, an order its invoice. Nobody sees the chain, and a wrong join is found only by walking it.

The picture is drawn by the app from the links, it is not a board the owner arranges: nothing is placed, moved or saved by hand. It is a part of the detail, so the three forms of a view stay (`knowledge/decision-views.md`).

## Done when

- [ ] To decide first and record in `knowledge/decision-views.md`: where the picture stands (a section "Zusammenhang" in the detail of a document and of a booking, opened on demand) and how it is laid out. The owner picks from two variants at real size with invented data, as in #37.
- [ ] The core returns the joined things of a document or a booking in one command: the document, its paired receipt or invoice, its order, the bookings that paid it, their statement, account and party, the other half of a move, and the tax assignment, each with the rule that joined it. Nothing is derived in the window.
- [ ] The picture shows each thing as a small card of the design system with its kind, name, day and amount, and a line to what it is joined to. A line shows "Warum?" with the rule, and "Rückgängig" where the join can be undone.
- [ ] A click on a card opens that thing in its view, with the arrow back (#58).
- [ ] The picture is fixed in size and never scrolls sideways: more than 12 things fold into "und n weitere" per kind, which opens the list.
- [ ] A thing that stands alone shows no picture, only "Nichts verknüpft".
- [ ] Every card and line is reached by the keyboard and read by a screen reader as a list of the joins.
- [ ] No new package without its row in `knowledge/decision-ui-libraries.md`. To verify there: whether the lines need more than SVG of the design system.
- [ ] Tests in the core with invented data: an invoice with receipt, order and booking, a move between two accounts, a thing alone. `npm run check:views` opens the picture in light and dark with 0 findings.
- [ ] Seen by the owner on a real invoice paid through PayPal.

## Out of scope

- A free canvas the owner arranges, and a picture of the whole archive.
- A flow of money by category in "Auswertungen": a requirement of its own when the owner asks for it.
