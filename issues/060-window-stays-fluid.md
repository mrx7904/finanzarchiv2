# #60 The window stays fluid while the archive works

**Goal:** Measured on the owner's archive, no click waits for the core for more than 100 ms, also during an import and while the model reads.
**Requirements**: FA-60 · **Depends on**: #55

Found by reading the code on 2026-11-27, nothing of it measured yet. Every command is a plain `fn` and takes the one `Mutex<Opened>` that the intake, the mail reader and the categories take too. A page travels as a PNG in base64 inside a string, up to 2560 px wide. `list_documents` asks the pair of every row by itself and is loaded whole on every change of the intake, by Dokumente, by "Start" and by the empty "Prüfen".

To verify: a Tauri 2 command without `async` runs on the main thread and so holds the window (v2.tauri.app, "Calling Rust from the frontend", read on the day of the work).

## Done when

- [ ] Measured first, with `examples/measure.rs` on a copy of the owner's archive, counts and times only: the time of every command the views call at start, and the longest time a command waited for the lock while 200 files are imported. The numbers stand in this file.
- [ ] No command that reads a file, renders a page or asks the model runs on the main thread. What was verified above is recorded in `knowledge/decision-stack.md`.
- [ ] Reading commands do not wait for a writer: decided here and recorded in `knowledge/decision-archive-file.md` (for example a reading connection of its own in WAL mode).
- [ ] A page reaches the window without base64 inside JSON. To verify: the asset protocol or a custom protocol of Tauri 2 serving the cached file, with the CSP of `tauri.conf.json` and the capability it needs.
- [ ] `list_documents` runs one query for the pairs. "Start" and "Prüfen" ask the core for their counts and the last filed documents, not for the whole list.
- [ ] Dokumente with 5,000 generated rows opens in under 300 ms and scrolls without a frame over 50 ms, measured in the check of the views.
- [ ] The start work of `lib.rs` that rereads and rechecks older documents does not delay the window: the time from the start of the process to the first view is measured before and after.
- [ ] After the change, measured again: no command called by a view takes more than 100 ms during the import of 200 files.

## Out of scope

- The flicker of the page: #55.
