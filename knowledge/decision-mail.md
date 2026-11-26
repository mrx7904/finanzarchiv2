# Decision: the mailboxes, read-only

**Decided:** 2026-11-26 (#29, FA-34). Gmail only, the owner's answer of 2026-11-26, so Microsoft 365 (#30) is deferred · applies to `src-tauri/src/mail/`, `src-tauri/src/commands/mail.rs`, the table `mailboxes` (migration 0018), "Postfächer" in "Dokumente"

## Decision

- **Gmail over IMAP with TLS** to `imap.gmail.com:993`, with an app password. The host is a constant in `mail/mod.rs` and the only one the client reaches (row in "Outside connections" of `conventions.md`). TLS is `rustls` with the `ring` provider and the roots of `webpki-roots` (the Mozilla list), so no certificate store or proxy of the system is involved.
- **Read-only on three levels:**
  1. Folders are opened with `EXAMINE`, never `SELECT`.
  2. Bodies are fetched with `BODY.PEEK[…]`, never `BODY[…]` or `RFC822`, which mark a mail as read.
  3. `mail::guard::ReadOnly` sits between the client and the socket. It holds every command line until its end, judges it and passes it on only when it is one of `CAPABILITY`, `NOOP`, `LOGOUT`, `LOGIN`, `ID`, `LIST`, `STATUS`, `EXAMINE`, `SEARCH`, `UID SEARCH`, or a `FETCH` / `UID FETCH` without `BODY[`, `BINARY[` or `RFC822` (except `.SIZE` and `.HEADER`). Anything else fails with `PermissionDenied` before a byte of it reaches the socket. Data of a literal is passed as data, and the line after it is judged again. `IDLE` and `AUTHENTICATE` are held back too, until a piece of work needs them.
- **The app password lies in the keychain of the system** (`keyring`, the service `de.mrx7904.finanzarchiv2.mail`, the address as user): Windows Credential Manager, the macOS keychain. The archive holds only the address. The window sends the password once, to the login and the keychain, and forgets it, whatever the login said. Spaces Gmail shows between the groups of four are removed.
- **"Postfächer"** (from "Dokumente"): add (address and app password, added only once the login worked), "Anmeldung prüfen", "Entfernen" (the password leaves the keychain, documents from the mailbox stay). A login looks into the inbox with `EXAMINE` and logs out. A failed login says why in German, never in the words of the server: not reachable, no answer within 30 s, TLS failed, address or app password wrong, the password of the Google account instead of an app password.
- The IMAP library `async-imap` logs every command at the level `trace`, `LOGIN` with its password. The app has no logger yet. When one comes, it holds `async_imap` below `trace`.
- Reading the mails in the background and taking their invoices is #31.

## Measured

Tests with a fake IMAP server on a local port, which records every command and would accept writing ones (`mail::tests`):
- a login sends exactly `LOGIN`, `EXAMINE`, `LOGOUT`;
- 22 writing calls of `async-imap` (`select`, `store`, `copy`, `mv`, `expunge`, `create`, `delete`, `rename`, `subscribe`, `append`, `close`, `check`, a `fetch` of `BODY[]`, `RFC822`, `RFC822.TEXT`, `BINARY[1]` and more) fail, and none of them reaches the server. `EXAMINE`, `UID SEARCH` and a `UID FETCH` of `BODY.PEEK[]` work after them;
- the test fails when the guard lets every command through;
- the app password is found in no file of the archive.

No login against the real Gmail was made by the agent: the owner types the app password in "Postfächer".
