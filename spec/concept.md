# finanzarchiv2 Concept

## Why

The owner's financial records are spread over bank statements, PDF invoices, payslips and mail. Version 1.0 (`mrx7904/finanzarchiv`, Electron, 0.37.0) proved that a local archive with a local model can file them and answer questions about the past. It grew in three weeks to 184 requirements, 51 tables and 252 IPC channels, and learned its model of money in four steps. Version 2.0 starts again from zero with what 1.0 taught, on Rust, so that the foundation fits the whole product from the first day. The lessons are in `docs/analyse-finanzarchiv-1.0.md`.

finanzarchiv2 is an **archive**, not a budget planner. It is filled when documents arrive and answers questions about the past. Daily budgeting stays in `budgetplan` (iOS).

## What it is

A desktop app on the owner's computers, Windows and macOS first and Linux later, built with Tauri 2: a Rust core that holds every rule and a React UI that shows and asks. The archive is one SQLite file and a folder of unchanged original files. A local model on Ollama reads what fixed readers cannot.

It keeps

- **accounts** and their **bookings**, the only source of money. Cash and accounts without statements get bookings too
- **documents** as unchanged originals under their hash: invoices, payslips, receipts, bank statements, orders and others
- the **values** read from each document, each with the reader it came from, the place in the document and the checks it passed
- **links** between bookings and documents, so every booking shows its evidence and every invoice shows whether it was paid
- **parties** (senders, employers, counterparties) with their aliases, **categories** and **rules**, and a record of every automatic **decision**

## How a document becomes data

1. Every way in (drop, folder, inbox folder, downloads, mailbox) creates one job in one intake queue.
2. The file is stored under its hash, its text with positions and its page images are taken in the core.
3. Fixed readers come first: e-invoices (ZUGFeRD, XRechnung), known bank formats, known senders. The local model reads only what they cannot, the page image is the last reader.
4. Code checks every value against the document and the archive. What passes every check is filed automatically, with "Warum?" and "Rückgängig". What fails waits with its reason as a question to the owner.
5. A booking that arrives later can confirm a waiting value, and a document that arrives later finds its booking.

## Principles

- **The archive stays on the computer, data may be fetched.** No content of the archive goes to a third-party service. The model runs locally. Every connection to the outside is listed in `knowledge/conventions.md`. Taken over from 1.0.
- **Two pillars.** A local archive in which the owner sees everything, and automation that files, names, links and checks by itself. The owner acts only where it is really needed.
- **The source stays.** Every value points to its original file and its place there.
- **Code decides what it can verify, the owner decides the rest.** The model is never asked how sure it is.
- **Exact numbers.** Amounts are exact to the cent.
- **One open stage.** A stage is built, then the owner uses it on real documents and accepts it, then the next begins. Version 1.0 built 18 stages and almost none was accepted. The owner may open the next stage while a part of the previous one waits for a machine (2026-11-25: stage 2 began with the Mac part of stage 1 open, everything else of stage 1 accepted on Windows).
- **Measured, not believed.** Every reader is measured on a generated corpus in the repository and on the owner's private corpus outside it.

## What comes when

1. **Foundation.** Shell on Windows and macOS, the check, the database and money, the typed boundary, the document store, the intake queue, PDF text and pages, the design system, local releases, the corpus.
2. **Reading and review.** Fixed readers, the model, the checks, automatic filing, the review as questions, naming by template.
3. **Money.** Accounts, bank statements as PDF and CSV, bookings, links, transfers, the balance against the bank, categories per party.
4. **Coming in by itself.** Watched folders, the downloads, mailboxes (Gmail, Microsoft 365) read-only in the background, invoice links, PayPal and Amazon.
5. **Understanding.** Start with the next steps, evaluations, taxes for the return, the chat over the archive.
6. **Around the archive.** Snapshots and restore, moving the archive, updates, profiles, achievements, styles, feedback and ideas.

## Not in scope

- Importing the archive of 1.0. The owner reads the original documents in again (decided 2026-11-25), which is also the first real test of the automation.
- Sorting the mailbox into folders, as 1.0 did (FA-68, FA-69 there). Left out at the start on the owner's word of 2026-11-25, possible later.
- Daily budget planning, savings goals, forecasts, streaks or reminders.
- Direct bank connections (FinTS, PSD2) until the owner decides on them.
- Several users sharing one archive, or a server.
- Tax advice. The view Steuern prepares, the owner checks.
