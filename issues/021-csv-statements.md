# #21 Statements as CSV, CSV before PDF

**Goal:** The CSV exports the owner's bank gives are taken in as statements, and the same booking from a CSV and a PDF is one booking.
**Requirements**: FA-26 · **Depends on**: #20

## Done when

- [x] The intake takes a `.csv` only when it reads as a statement of a known layout (IBAN, period, balances), with its encoding detected. Any other CSV is refused at the door as today.
- [x] Its balances prove it like a PDF (`check:balance`).
- [x] A booking of a CSV and a booking of a PDF of the same account are the same when day, direction and amount agree and it is the only such booking of the day. The CSV's booking counts, and a link or category the owner gave stays.
- [x] Tested with generated CSV and PDF of overlapping months. Measured on the owner's CSV files, counts only.
