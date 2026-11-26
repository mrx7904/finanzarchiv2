# #17 The review as questions

**Goal:** "Prüfen" shows what waits, grouped by reason, mostly as yes or no questions with the page beside, and an answer moves the document on.
**Requirements**: FA-22 · **Depends on**: #16

## Done when

- [x] The app has a way between "Dokumente" and "Prüfen" (a sidebar of the design system), and the path in the top bar reads "Prüfen › <group> › <document>" with each earlier segment a button.
- [x] Groups by reason with their count. A question names the value and where it was read, for example "Ist 49,99 € das Brutto von „Beispiel Stadtwerke GmbH“?", with the line marked on the page.
- [x] "Ja" stores the value as confirmed by the owner, "Nein" opens the field with the readings beside each other. A typed value is parsed by `Cents::parse_de` and refused with its reason.
- [x] After an answer the document runs through the checks again and is filed or waits with what is left.
- [x] Keyboard: J and K move, Enter opens, 1 and 2 pick a reading. Tested in the UI tests.

## Out of scope

- Questions that apply to every document of a sender at once, after the owner has used the view
