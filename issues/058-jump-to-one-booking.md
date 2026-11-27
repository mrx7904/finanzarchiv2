# #58 A jump leads to the one booking

**Goal:** Wherever the window names a booking, a click opens that booking, chosen in "Umsätze" with its detail.
**Requirements**: FA-58 · **Depends on**: #48, #50

Found by reading the code on 2026-11-27. The navigation knows an account, a text and a filter of "Umsätze", but no booking (`src/navigation.ts`). So a source "Buchung" of the chat and an entry of "Steuern" open all bookings (`App.tsx`), the booking that paid a document opens its whole account (`Documents.tsx`), and a booking found in the quick search opens the bookings of its party. Sources of the chat and entries of "Steuern" read only "Dokument" or "Buchung".

## Done when

- [ ] The place of "Umsätze" can hold one booking. A jump to it shows the list around that booking with the row chosen, scrolled into view and its detail open, also when the booking lies beyond the first loaded rows.
- [ ] These lead to their booking: a source of the chat, an entry of "Steuern", the booking under a document in Dokumente, a booking in the quick search, a booking of a savings hint and of "Wiederkehrend".
- [ ] The arrow back leads from the booking to where the jump began.
- [ ] A source of the chat and an entry of "Steuern" show what they are: the name of the document, or the party and the day of the booking.
- [ ] A booking that no longer exists shows "Die Buchung gibt es nicht mehr" in place of the detail.
- [ ] Tests cover the jump in `navigation.test.ts` and the row found beyond the first page.

## Out of scope

- New filters of "Umsätze": #64.
