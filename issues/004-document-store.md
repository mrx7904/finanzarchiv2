# #04 Unchanged originals under their hash, in by drop and dialog

**Goal:** A dropped or picked file lies unchanged under its SHA-256 hash in the archive folder and has a row in `documents`, and a second import of the same file stores nothing new.
**Requirements**: FA-07, FA-08 · **Depends on**: #02, #03

## Done when

- [x] Files and whole folders can be dropped anywhere on the window or picked in the dialog of the system. The UI never sends a path the core did not give it. The dialog was driven in the release build on 2026-11-25, dropping is built on the window event and still to be tried by the owner.
- [x] The file is written as `<archive>/<first two hex>/<hash>.<ext>` from the bytes that were hashed. A test compares the hash of the stored file with the source.
- [x] PDF, PNG, JPEG and WebP are accepted. An empty, damaged or password protected PDF is refused with a German reason, tested with generated files.
- [x] Importing the same file twice gives one row and the answer "schon im Archiv", tested.
- [x] The list of documents shows name, kind (still "unbekannt") and the day of import.

## Out of scope

- Reading the text, belongs to #06
- Watched folders and mail, belong to stage 4
