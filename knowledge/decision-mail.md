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
## Invoices from mail (#31, FA-35)

**Decided:** 2026-11-26 · applies to `mail/round.rs`, `mail/reader.rs`, `mail/render.rs`, the table `mail_messages` (migration 0019)

- **When:** a round through every mailbox a minute after start, every 15 minutes after, once at once after a mailbox was added, and at "Jetzt lesen" in "Postfächer". One round at a time.
- **Where:** the folder with the attribute `\All` (RFC 6154), "All Mail" in the language of the account, so a mail is found whatever its labels and whether it was archived. Without one, `INBOX`. A mailbox keeps folder, UIDVALIDITY and the last UID looked at. The next round asks for `UID last+1:*`. A changed UIDVALIDITY or folder starts over by date, and the mails known by hash are skipped.
- **How far back:** the first round reads the last 365 days (`round::REACH_DAYS`).
- **Prefilter by code**, from the head (`BODY.PEEK[HEADER]` and `RFC822.SIZE`, 50 mails a command): a word of an invoice in subject or sender (`rechnung`, `invoice`, `beleg`, `quittung`, `receipt`, `zahlung`, `payment`, `abrechnung`, `gutschrift`, `kontoauszug`, `bestellung`, `order`, `billing`, `faktura`), or a mail that is neither plain text nor text with its HTML, so it may carry a PDF. Only those are fetched whole (`BODY.PEEK[]`). A whole mail goes to the model only when it has a PDF attachment (named `.pdf` or `application/pdf`, starting with `%PDF`) or the word. A mail over 30 MB is not fetched.
- **The model** (`round::ask_kind`) gets sender, subject, the names of the PDF attachments and the first 1,500 characters of the text, and answers one of invoice, payslip, receipt, statement, other. It is never asked how sure it is. When it does not answer, the mail and every later one wait for the next round: the last UID stays before them.
- **What is taken:** for a document of money, every PDF attachment. Without one, the mail itself becomes a PDF (`render::mail_pdf`): sender, date and subject in bold on top, then the plain text of the mail in Helvetica on A4, wrapped at 92 characters. Only text goes in, so no image, font or link of the mail is ever loaded. Characters Helvetica cannot show become spaces.
- **Into the intake:** the PDFs wait in `Mail-Eingang` beside the archive, under the first 16 characters of the mail's hash, and go through the intake with the source `mail`. A round removes what the intake took (every job past `queued`).
- **Once:** every mail looked at is a row of `mail_messages` under the SHA-256 of its Message-ID (of sender, date and subject when it has none), with the verdict in German, the count of files and no body. A row outlives its mailbox, so a mailbox added again does not ask about a mail twice. A document the owner deletes stays deleted (`deleted_sources`, #27), and its mail is not looked at again anyway.

## Measured

Tests with a fake IMAP server on a local port, which records every command and would accept writing ones (`mail::tests`):
- a login sends exactly `LOGIN`, `EXAMINE`, `LOGOUT`;
- 22 writing calls of `async-imap` (`select`, `store`, `copy`, `mv`, `expunge`, `create`, `delete`, `rename`, `subscribe`, `append`, `close`, `check`, a `fetch` of `BODY[]`, `RFC822`, `RFC822.TEXT`, `BINARY[1]` and more) fail, and none of them reaches the server. `EXAMINE`, `UID SEARCH` and a `UID FETCH` of `BODY.PEEK[]` work after them;
- the test fails when the guard lets every command through;
- the app password is found in no file of the archive.

No login against the real Gmail was made by the agent: the owner types the app password in "Postfächer".

Tests of #31 against the fake server with five invented mails (`mail::round::tests`):
- a newsletter is judged by its head alone, a photo without PDF and without the word is left by code;
- a PDF attachment and an invoice in the text are taken, a shipping note the model calls other is left, and the model is asked three times, never how sure it is;
- the mail as a PDF carries sender, date and subject on top and its text below, umlauts and the euro sign included;
- the next round finds nothing new, a known mail is not asked about, and without the model the rest waits;
- no command of a round writes.

On the owner's two Gmail mailboxes, read only, counts only (2026-11-26, `measure --reader mail --days 30`, gemma4:12b-it-qat, model unloaded afterwards). Nothing was staged or kept:

| Mails of 30 days | Taken | Left out by code | Left out by the model | Asked the model | Time |
|---|---|---|---|---|---|
| 278 | 10 (4 with PDF attachments, 6 as a PDF of the mail) | 237 | 31 | 41 | 51 s |

Whether the 10 are all invoices and the 268 hold none was not checked mail by mail: the owner sees the 10 in the archive once the version with #31 runs, and what waits in "Prüfen".
