# #26 Accounts and bookings in the window

**Goal:** The owner sees every account with its balance and finds every booking with its statement, documents and category.
**Requirements**: FA-31 · **Depends on**: #22, #24, #25

## Done when

- [x] A view "Konten" in the sidebar: each account with its balance, the bank's last balance with its day, and a mark where they differ or a statement is missing.
- [x] "Umsätze" of one account or of all: day, party, purpose, amount, category, documents and statement. They can be filtered by period, party and category, show newest first, and stay quick with ten thousand bookings.
- [x] A booking opens its statement at its page, and its documents. Its category can be changed there, for it alone or for all of its party. Merging parties is in the core (#25), its buttons come with the first look of the owner.
- [ ] The path in the top bar reads "Konten › <Konto> › Umsätze". Built and tested, to be seen by the owner on Windows.
