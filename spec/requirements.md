# finanzarchiv2 Requirements

What is required and which issue implements it. The why is in `concept.md`, the status only in `../issues/README.md`. Requirements of later stages are written when their stage is cut into issues.

## Stage 1 Foundation

### FA-01 One code base for Windows and macOS

The app builds from one repository into a Tauri 2 desktop app that opens its window on Windows 11 and on macOS, with a Rust core and a React UI.

Issue: #01

### FA-02 One local check

One command runs every check of the repository, fails when any step fails, and proves with a self-test that each step can fail. No step needs GitHub Actions.

Issue: #01

### FA-03 Nothing personal in the public repository

The check refuses an IBAN outside the synthetic fixtures, a mail address other than an example or noreply address, and an absolute path into a home folder, in every tracked file.

Issue: #01

### FA-04 The archive database

The archive is one SQLite file with `STRICT` tables and foreign keys. Migrations are embedded and applied in order, the file is checked before the app works on it, and the app keeps snapshots of it.

Issue: #02

### FA-05 Exact money

Amounts are integer cents from the database to the screen. Parsing a German printed amount and formatting one are tested with generated values.

Issue: #02

### FA-06 A typed boundary from one source

The UI reaches the core only through bindings generated from the Rust commands, and the check fails when the bindings no longer match the core.

Issue: #03

### FA-07 Unchanged originals under their hash

An imported file is stored unchanged under its SHA-256 hash. The same file imported twice is stored once and reported as known.

Issue: #04

### FA-08 Files in by drop and dialog

The owner drops files or folders into the window or picks them in a dialog, without saying what they are.

Issue: #04

### FA-09 One intake queue

Every way in creates one job in a queue kept in the database. A job moves through fixed stages, the queue resumes after a restart at the stage each job reached, can be paused, and works on several jobs at once.

Issue: #05

### FA-10 Text and pages of a PDF

The core takes the text of every page with its position and renders a page as an image for the preview and for a later reading by image.

Issue: #06

### FA-11 A design system chosen by the owner

Colors, type, spacing, surfaces and the base pieces stand as tokens and components, built from the six UI sources, after the owner picked them from variants shown at real size.

Issue: #07

### FA-12 Local releases

A script per system builds the installer on that system and uploads it to a GitHub release, after the check passed.

Issue: #08

### FA-13 A corpus to measure on

Code generates invoices, payslips and bank statements with known values into the repository. One command measures a reader on this corpus and, when it is set, on the owner's private corpus outside the repository.

Issue: #09

## Stage 2 Reading and review

### FA-14 The kind of a document by code

Code decides from the printed signals of the text whether a document is an invoice, a payslip, a receipt (Zahlungsbeleg), a bank statement or something else, and says "unknown" when the signals of two kinds are equally strong. The kind is stored with the signals it rests on.

Issue: #10

### FA-15 Every value with its source

A value read from a document is stored with the reader that produced it, the page and the box of the line it stands on, and the time. The checks it passed or failed are stored beside it. A value the owner types is stored the same way, with the owner as reader.

Issue: #11

### FA-16 Invoices read by code

Code reads the invoice number, the invoice date, the net, the rate and amount of VAT, the gross, the due date, the IBAN and the sender of an invoice from its labelled lines, as printed, and turns amounts and dates into `Cents` and dates in code.

Issue: #11

### FA-17 E-invoices from their XML

An invoice that carries a ZUGFeRD, Factur-X or XRechnung XML is read from that XML, and its values count as exact.

Issue: #12

### FA-18 Payslips read by code

Code reads the employer, the month, the Brutto, each deduction and the Netto of a payslip from its labelled lines.

Issue: #13

### FA-19 The local model reads what code cannot

The local model on Ollama reads the fields code left empty, with a JSON schema for its answer, copying amounts and dates as printed and naming the line each value stands on. It is never asked how sure it is. Without a running Ollama a document waits for it and does not fail. Which model reads is chosen by measurement on the generated and the owner's corpus.

Issue: #14

### FA-20 Reading the page as an image

A scan without text, and a decisive value that no reader found in the text, is read from the image of the page by a local vision model. Its values carry the image as their source.

Issue: #15

### FA-21 Checks decide what is filed by itself

A document whose decisive values stand in its text, add up to the cent and agree between two independent readers, and that is no duplicate, is filed without the owner. Every such decision is stored with its rule, the values it saw and how to undo it, and the window shows "Warum?" and "Rückgängig". Everything else waits with its reasons.

Issue: #16

### FA-22 The review as questions

What waits is shown in "Prüfen", grouped by reason, mostly as questions to answer with yes or no, with the page beside the values. A correction is stored as a value of the owner and the document goes on through the checks.

Issue: #17

### FA-23 Names by template

Each kind of document is named by a template the owner can change, filled from its confirmed values when it is filed. A name typed by the owner is kept.

Issue: #18

### FA-24 A snapshot after a large import

After an import of many documents the app writes a snapshot of the archive at once, not only at the next start.

Issue: #19

## Stage 3 Money

### FA-25 Statements read by code, each proved by its balance

A bank statement as PDF is read by code in the layout of its bank: the account (IBAN, bank), the period, the old and the new balance, and every booking with its day, amount, other side and purpose. Old balance plus the bookings is the new balance to the cent, or the statement waits with the gap. Each IBAN is an account, made the first time it appears.

Issue: #20

### FA-26 Statements as CSV, CSV before PDF

A CSV export of a bank is taken in when it reads as a statement of a known layout, proved by its balances like a PDF. Where a CSV and a PDF cover the same days of an account, the bookings are not doubled, and the CSV counts.

Issue: #21

### FA-27 The balance of each account against the bank

The balance of an account on a day is its opening balance plus its bookings up to that day. It is held against every balance the bank printed, and a difference or a gap between two statements is shown, never hidden.

Issue: #22

### FA-28 Moves between own accounts

A booking whose other side is an own account is a move: it changes both balances and counts as neither income nor spending. The bank itself is no own account.

Issue: #23

### FA-29 Documents linked to their bookings

An invoice, receipt or payslip is linked to the booking that paid it by rules with a stable id (amount, number in the purpose, name, days), each link a decision with "Warum?" and "Rückgängig". A link no rule is sure of is a question in "Prüfen".

Issue: #24

### FA-30 Parties and their categories

The other sides of bookings and the senders of documents are parties, each with its spellings. A category belongs to a party, learned from the owner's choices, and a booking takes the category of its party unless the owner gave it another.

Issue: #25

### FA-31 Accounts and bookings in the window

"Konten" shows each account with its balance, the bank's last balance and what does not match. A list shows the bookings of an account or of all, with their statement, their documents and their category, filtered by period, party and category.

Issue: #26

## Stage 4 Coming in by itself

### FA-32 Watched folders and the inbox folder

Folders the owner chooses are watched, and a new file in one of them goes through the intake as if it was dropped. An inbox folder is there from the start. Nothing in a watched folder is moved, renamed or deleted.

Issue: #27

### FA-33 The downloads folder

When the owner switches it on, the downloads folder of the system is watched at its top level, and a file is taken only when it looks like an invoice, a payslip, a receipt or a statement. The files that lay there before are left alone until the owner asks.

Issue: #28

### FA-34 Mailboxes read-only

The owner's mailboxes (Gmail with an app password, Microsoft 365 with OAuth in the system browser) are read in the background and never changed: no mail is moved, deleted, sent or marked as read. Credentials are kept in the keychain of the system, never in a file of the app.

Issue: #29, #30

### FA-35 Invoices from mail

A mail that is an invoice, a receipt or a payslip is found and its PDF attachment, or the mail itself as a PDF without anything it would load from outside, goes through the intake. Every mail is looked at once.

Issue: #31

### FA-36 PayPal and Amazon

A payment mail of PayPal is read by code (payee, amount, day, transaction code) and stands for the payment when no invoice of the seller comes. Two mails of one payment are one. Amazon orders find their invoice where it can be had.

Issue: #32

## Stage 5 Understanding

### FA-37 Income and spending

The archive tells its income and spending per month and per year from the bookings, the only source of money, with the moves between own accounts left out and a refund lowering what it refunds. Every sum opens the bookings behind it, by category and by party.

Issue: #33

### FA-38 Recurring payments

Payments that come again (subscriptions, contracts, insurances, rent) are found from the bookings by their party, their rhythm and their amount, with the next one expected and a change of the amount named.

Issue: #34

### FA-39 Salary from the payslips

Brutto, Netto and the deductions of every month come from the payslips, a month without a payslip within the archived range is named as missing, never counted as zero.

Issue: #35

### FA-40 Balances over time

The balance of each account and of all of them is told for every day of the archived range, from the bookings and the balances the banks printed.

Issue: #36

### FA-41 The evaluations in the window

A view "Auswertungen" shows the figures of FA-37 to FA-40 for a year the owner picks, in a look the owner chose from variants at real size with the archive's own numbers, light and dark.

Issue: #37, #38

### FA-42 Savings hints

Observations from the archive that can save money: a recurring payment whose amount rose, two recurring payments of the same kind, fees, a category above the year before. Each says what it rests on with its numbers and opens the bookings, and none is financial or investment advice.

Issue: #39

### FA-43 The start view

A view "Start" says in a sentence what waits and lists the next steps, each with one button that does it: documents in "Prüfen", orders without an invoice, a mailbox or a folder with a problem, a statement missing in an account, the local model out of reach. A few key figures of the running year stand beside them.

Issue: #40

### FA-44 Taxes for the return

What is relevant for the owner's own tax return is assigned to invoices and bookings, never copied, and summed per tax year by the day it was paid, with the values the law fixes per year from checked sources. It names form and section, never computes a refund and never presents a matter as settled. Taken over from 1.0 (`decision-tax.md` there, 2026-10-31), the owner's situation unchanged (2026-11-26).

Issue: #41

### FA-45 The chat over the archive

The owner asks the archive in words. The local model reaches data only through fixed tools on a connection that cannot write, code formats and checks every number of an answer, and the sources of an answer are the documents and bookings the tools returned. Taken over from 1.0 (`decision-chat.md` there).

Issue: #42

### FA-46 Categories by the model

Every party without a category gets one from the local model, from its name, its bookings and its documents. A category the owner chose is never changed by the model, and the window shows no explanation of the choice (the owner, 2026-11-26: "nicht so textlastig").

Issue: #43

## The views, brought together

The map and the rules are decided in `knowledge/decision-views.md`. These requirements say what the window owes them.

### FA-47 One frame and one way to move

Every view stands in one frame: a sidebar with the same views everywhere, the path of the place in the top bar, the intake seen from every view, the quick search from every view. A jump leads from a thing to what belongs to it and an arrow leads back. A healthy archive says nothing, a damaged or missing one replaces every view.

Issue: #44

### FA-48 Every piece of a view comes from the design system

No view draws a control of its own. A piece a view needs is added to the design system and shown in the showcase in light and dark before the view uses it.

Issue: #45

### FA-49 Every view has one of three forms

A view is a list with its detail beside it, an overview or a question, with its actions in the top bar, its filters in the head of its list, the states in one vocabulary, and its own look when it is empty, loads or fails. From the narrowest window to a large screen nothing is covered and nothing runs out of the window.

Issue: #46, #47, #48, #52

### FA-50 Settings in one place

What the owner sets up once (folders, mailboxes, name templates, categories, the model, the archive, the appearance) stands in one view "Einstellungen" and nowhere else.

Issue: #49

### FA-51 The views of stage 5 follow the map

Start, Auswertungen, Steuern and Chat are built after the map and the rules, none is designed by itself.

Issue: #50

### FA-52 The rules are checked, not remembered

A command opens every view in light and dark at several window sizes and fails when a rule is broken, and it proves that it can fail.

Issue: #51

### FA-53 The owner fits a view to the work

The line between two columns can be dragged, the sidebar folds to a rail, the values of a document and the detail of a booking fold away, and the whole view has three sizes. The app keeps each over a restart, and nothing gets so narrow that it is covered.

Issue: #53

### FA-54 Reading large and answering in focus

Every page of a document can be turned to and zoomed, and a document can be read over the whole window. In "Prüfen" the owner can hide everything but the page and the question.

Issue: #54
