# #19 A snapshot after a large import

**Goal:** After an import of many documents the archive is saved at once, not only at the next start.
**Requirements**: FA-24 · **Depends on**: none

## Done when

- [ ] When the queue becomes empty after at least a threshold of new documents since the last snapshot (one value in the core), a snapshot with the reason `nach-import` is written. Found on 2026-11-25: the owner's first 132 documents had no snapshot until the second start.
- [ ] Tested with a temporary folder: below the threshold none, at the threshold one, the newest ten kept as before.

## Out of scope

- Restoring a snapshot from the window, belongs to stage 6
