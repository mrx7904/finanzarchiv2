# #29 Gmail, read-only

**Goal:** The owner's Gmail is read in the background, and nothing in it ever changes.
**Requirements**: FA-34 · **Depends on**: #05

## Done when

- [x] IMAP over TLS to `imap.gmail.com:993` with an app password. The client library and its version are checked at the primary source and written into `decision-stack.md`. The row in "Outside connections" says what goes there.
- [x] Read-only on three levels:
  - folders opened with `EXAMINE`;
  - bodies fetched with `BODY.PEEK`;
  - a client that refuses every writing command before it reaches the socket.

  A test against a fake IMAP server records every command and fails on any that writes.
- [x] The app password lies in the keychain of the system (`keyring`), never in the database, a file or a log.
- [x] "Postfächer" in the window: add, test the login, remove. A login that fails says why in German.

**Done:** 2026-11-26. Proved against a fake IMAP server ([decision-mail](../knowledge/decision-mail.md)). The first login against the real Gmail is the owner's, in "Postfächer".
