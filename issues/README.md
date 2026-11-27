# finanzarchiv2 Issues

**This is the only source of status.** The requirements say what is required and which issue implements it, the concept says why. Neither carries status.

Status: `open` · `in progress` · `done` · `deferred`

At most one stage is open. The next stage is cut into issues and begun only after the owner accepted the previous one on real documents (`spec/concept.md`, principle "One open stage").

## Stage 1 Foundation

| # | Title | Requirements | Depends on | Status |
|---|---|---|---|---|
| [#01](001-shell-and-check.md) | The shell on Windows and macOS, and the check | FA-01, FA-02, FA-03 | none | in progress |
| [#02](002-database-and-money.md) | The archive database and exact money | FA-04, FA-05 | #01 | done |
| [#03](003-typed-boundary.md) | A typed boundary between core and UI | FA-06 | #01 | done |
| [#04](004-document-store.md) | Unchanged originals under their hash, in by drop and dialog | FA-07, FA-08 | #02, #03 | done |
| [#05](005-intake-queue.md) | One intake queue that survives a restart | FA-09 | #04 | done |
| [#06](006-pdf-text-and-pages.md) | Text and pages of a PDF in the core | FA-10 | #05 | in progress |
| [#07](007-design-system.md) | The design system, chosen by the owner | FA-11 | #01 | done |
| [#08](008-local-releases.md) | Releases built on the owner's machines | FA-12 | #01, #06 | in progress |
| [#09](009-corpus.md) | A generated corpus and a measure command | FA-13 | #06 | done |

**Stage 1 is done when the owner installs 2.0 on the Windows PC and on the Mac from a release, drops 20 real PDFs, sees each in the list with its first page, quits in the middle of the import, and after the next start finds every file once.**

**Status:** Accepted in part on Windows on 2026-11-25. The owner imported 135 real files through the dialog into the installed 0.1.0 in about 16 seconds, which the owner called fast: 132 PDFs stored once each under their hash (132 distinct hashes, no row without its file, no partial file, no open job), 131 read with text and 1 marked as a scan, 3 refused with their reason (two CSV statements, one spreadsheet). Then the owner started the app again (its first daily snapshot was written) and dropped three single files on the window: two PDFs already in the archive ended as known and were not stored again, a CSV was refused with its reason. Quitting while an import of new files runs was not reached by the drops (each file was done in less than a millisecond). The owner accepted the process kill test of #05 for it on 2026-11-25 (six kills of a real process in 200 files, every file once). Open: everything on the Mac.

Order of work: #01 first. Then #02, #03 and #07 in parallel. Then #04, #05 and #06, then #08 and #09.

## Stage 2 Reading and review

| # | Title | Requirements | Depends on | Status |
|---|---|---|---|---|
| [#10](010-kind-by-code.md) | The kind of a document by code | FA-14 | none | done |
| [#11](011-values-and-invoice-reader.md) | Values with their source, and invoices read by code | FA-15, FA-16 | #10 | done |
| [#12](012-e-invoices.md) | E-invoices from their XML | FA-17 | #11 | done |
| [#13](013-payslips-by-code.md) | Payslips read by code | FA-18 | #11 | done |
| [#14](014-local-model.md) | The local model reads what code cannot | FA-19 | #11 | done |
| [#15](015-reading-by-image.md) | Reading the page as an image | FA-20 | #14 | done |
| [#16](016-checks-and-filing.md) | Checks decide what is filed by itself | FA-21 | #12, #13, #14 | done |
| [#17](017-review-as-questions.md) | The review as questions | FA-22 | #16 | done |
| [#18](018-names-by-template.md) | Names by template | FA-23 | #16 | done |
| [#19](019-snapshot-after-import.md) | A snapshot after a large import | FA-24 | none | done |

**Stage 2 is done when the owner's archive is read, at most 10 % of its documents wait for the owner, and a sample of 30 documents filed without the owner holds no wrong value.** Decided by the owner on 2026-11-25: documents that pass every check are filed automatically, and stage 2 starts while the Mac part of stage 1 is open.

Order of work: #19 and #10 first, then #11, then #12, #13 and #14, then #15 and #16, then #17 and #18.

## Stage 3 Money

| # | Title | Requirements | Depends on | Status |
|---|---|---|---|---|
| [#20](020-statements-by-code.md) | Statements read by code, each proved by its balance | FA-25 | #16 | done |
| [#21](021-csv-statements.md) | Statements as CSV, CSV before PDF | FA-26 | #20 | done |
| [#22](022-balance-against-the-bank.md) | The balance of each account against the bank | FA-27 | #20 | done |
| [#23](023-moves-between-own-accounts.md) | Moves between own accounts | FA-28 | #22 | done |
| [#24](024-documents-linked-to-bookings.md) | Documents linked to their bookings | FA-29 | #20, #16 | done |
| [#25](025-parties-and-categories.md) | Parties and their categories | FA-30 | #20 | done |
| [#26](026-accounts-and-bookings-view.md) | Accounts and bookings in the window | FA-31 | #22, #24, #25 | in progress (owner's look) |

**Stage 3 is done when every statement of the owner (PDF and CSV) is booked into its account, every account's balance equals the bank's, the moves between own accounts count in no income or spending, and the owner finds each invoice and payslip at the booking that paid it.**

**Status:** Begun on 2026-11-26 on the owner's word ("direkt weiter mit Teil 3") while stage 2 waits for the owner's look at 0.2.1 (the answers "Nein" in "Prüfen" were reworked after the owner tried 0.2.0).

Order of work: #20 first, then #21, #22 and #25, then #23 and #24, then #26.

## Stage 4 Coming in by itself

| # | Title | Requirements | Depends on | Status |
|---|---|---|---|---|
| [#27](027-watched-folders.md) | Watched folders and the inbox folder | FA-32 | #05 | done |
| [#28](028-downloads-folder.md) | The downloads folder | FA-33 | #27, #14 | done |
| [#29](029-gmail-read-only.md) | Gmail, read-only | FA-34 | #05 | done |
| [#30](030-microsoft-365.md) | Microsoft 365 with OAuth in the system browser | FA-34 | #29 | deferred (the owner reads Gmail only, 2026-11-26) |
| [#31](031-invoices-from-mail.md) | Invoices from mail | FA-35 | #29, #14 | done |
| [#32](032-paypal-and-amazon.md) | PayPal and Amazon | FA-36 | #31, #24 | done |

**Stage 4 is done when a document the owner saves, downloads or receives by mail is in the archive without a click, and no mailbox was changed by the app.**

**Status:** Begun on 2026-11-26 on the owner's word ("direkt weiter") while stage 3 waits for the owner's look at its views.

Order of work: #27 first, then #28 and #29, then #30 and #31, then #32.

## Stage 5 Understanding

| # | Title | Requirements | Depends on | Status |
|---|---|---|---|---|
| [#33](033-income-and-spending.md) | Income and spending from the bookings | FA-37 | #23, #25 | done |
| [#34](034-recurring-payments.md) | Recurring payments | FA-38 | #33 | done, the owner checks a sample |
| [#35](035-salary-from-payslips.md) | Salary from the payslips | FA-39 | #13 | done, measured once payslips are in |
| [#36](036-balances-over-time.md) | Balances over time | FA-40 | #22 | done |
| [#37](037-evaluation-variants.md) | Variants of the evaluations, chosen by the owner | FA-41 | #33, #34, #35, #36 | open |
| [#38](038-evaluations-view.md) | The view "Auswertungen" | FA-41 | #37, #44, #45 | open |
| [#39](039-savings-hints.md) | Savings hints | FA-42 | #34, #33 | done, the owner checks a sample |
| [#40](040-start-view.md) | The start view | FA-43 | #38, #39, #44, #45 | open |
| [#41](041-taxes.md) | Taxes for the return | FA-44 | #33, #24, #44, #45 | open |
| [#42](042-chat.md) | The chat over the archive | FA-45 | #33, #34, #35, #41, #44, #45 | open |
| [#43](043-categories-by-model.md) | Categories by the model | FA-46 | #25, #14 | done |

**Stage 5 is done when the owner sees on "Start" what to do, reads what a year brought in "Auswertungen" with every figure leading to its bookings, collects the return in "Steuern" and gets right numbers from the chat, all on the real archive.**

**Status:** Cut on 2026-11-26 with the owner's answers (all four parts, savings hints added by the owner, the tax situation as in 1.0, the look from variants at real size) and begun on the owner's word ("weiter mit Stufe 5") while stage 4 waits for the owner's look.

Order of work: #33 first, then #43 (added by the owner's answer of 2026-11-26), then #34, #35 and #36, then #37 and #38, then #39 and #40, then #41, then #42.

## The views, brought together

| # | Title | Requirements | Depends on | Status |
|---|---|---|---|---|
| [#44](044-frame-and-navigation.md) | The frame of the views and how one moves | FA-47 | #07, #26 | open |
| [#45](045-pieces-for-the-views.md) | The pieces the views need | FA-48 | #07 | open |
| [#46](046-documents-view.md) | The view "Dokumente" as designed | FA-49 | #44, #45 | open |
| [#47](047-review-view.md) | The view "Prüfen" as designed | FA-49 | #44, #45 | open |
| [#48](048-accounts-and-bookings-view.md) | The views "Konten" and "Umsätze" as designed | FA-49 | #44, #45 | open |
| [#49](049-settings-view.md) | The view "Einstellungen" | FA-50 | #44, #45 | open |
| [#50](050-views-of-stage-5.md) | The views of stage 5 in the frame | FA-51 | #44, #45, #51 | open |
| [#51](051-check-of-the-views.md) | A check that every view keeps the rules | FA-52 | #46, #47, #48, #49 | open |
| [#52](052-recurring-payments-page.md) | The page "Wiederkehrend" under Konten | FA-49 | #34, #48 | open |
| [#53](053-fitting-a-view.md) | Fitting a view: widths, the rail, folded columns, the size | FA-53 | #44, #45, #46, #48, #51 | open |
| [#54](054-reading-and-focus.md) | Reading a document large, and the focus of "Prüfen" | FA-54 | #46, #47 | open |

**The views are brought together when the owner works through a week of real documents in the installed app, from Start to Prüfen, Dokumente, Konten and Einstellungen, and finds every action where the map puts it, in light and dark, and `npm run check:views` passes.**

**Status:** Cut on 2026-11-27 from the draft the owner went through as pictures ("Gefällt mir sehr gut", `knowledge/decision-views.md`). Not begun: the owner finishes the functions first and brings the frontend together after. It is no stage of `spec/concept.md` but the work that joins the stages in the window, so it does not wait for stage 5 to be accepted.

Order of work: #45 and #44 first, then #46, #47, #48 and #49 in any order, then #51 and #54, then #53, then #52 once #34 is done. #53 and #54 were added on 2026-11-27 on the owner's question about usability ("Elemente von der Größe her anpassen, ein/ausklappen, Fokusmodus"). #38, #40, #41 and #42 build their views after #44 and #45, #50 holds them to the map.

## Later stages

Stage 6 of `spec/concept.md` is cut into issues when its turn comes.
