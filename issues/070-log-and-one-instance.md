# #70 A log file and one instance

**Goal:** What went wrong on the owner's computer can be read afterwards, and two windows never work on one archive.
**Requirements**: FA-70 · **Depends on**: none

As of 0.7.0 the core reports its problems with 16 `eprintln!`, which nobody sees in the installed app, so a fault on the real archive leaves no trace to work from. Nothing keeps a second start from opening the same SQLite file with its own intake, watcher and mail reader. 1.0 had both a log without values of the archive and one instance per data folder (`docs/analyse-finanzarchiv-1.0.md`, "Rahmen").

## Done when

- [ ] The core writes a log file beside the archive: the time, the place in the code and the kind of the error, never a value of the archive, no name of a file or a party, no amount, no text of a document (`knowledge/conventions.md`, "Rust"). A test logs a document with invented values through every logging call and finds none of them in the file.
- [ ] Every `eprintln!` of `src-tauri/src` is replaced, and `clippy` denies `print_stderr` and `print_stdout` outside tests and examples.
- [ ] A panic of a worker is logged with its place, and the queue goes on with the next job. A test makes a worker panic and expects the rest of the files once each.
- [ ] A failure of a command the window shows (#59) is logged with the name of the command.
- [ ] The log is cut at 5 MB with one older file kept. Einstellungen › Archiv has "Protokoll im Ordner zeigen".
- [ ] A second start brings the running window to the front and ends. To verify: the single instance plugin of Tauri 2 and the permission it needs, at v2.tauri.app on the day of the work, with its row in `knowledge/decision-app-shell.md`.
- [ ] Files handed to the second start (for example dropped on the icon) go into the intake of the first.
- [ ] Seen on Windows in the installed app: started twice, one window.

## Out of scope

- Sending a log anywhere: the archive stays on the computer.
- Restoring a snapshot: stage 6.
