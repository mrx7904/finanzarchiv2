# #25 Parties and their categories

**Goal:** Who money goes to or comes from is one party however the bank spells it, and the party carries its category.
**Requirements**: FA-30 · **Depends on**: #20

## Done when

- [x] `parties` and `party_aliases`: the other side of a booking and the sender of a document are matched to a party by their folded spelling (legal form, spaces and case left out). A new spelling of a known IBAN joins its party.
- [x] Categories in one list of 20 to 30 everyday ones, defaults in the core, which the owner renames, removes and extends in the app (the owner, 2026-11-26). The core is done here, the dialog comes with the views of #26. A booking takes the category of its party. The owner's choice for one booking can be made the party's ("Für alle von …"), stored as a correction the rule reads.
- [x] The owner can merge two parties or split a spelling off, each undoable.
- [x] Tested with generated spellings. Measured on the owner's copy, counts only: parties, bookings with a category.
