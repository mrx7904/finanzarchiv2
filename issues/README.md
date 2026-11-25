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

## Later stages

Stages 2 to 6 of `spec/concept.md` are cut into issues when their turn comes, with what stage 1 taught.
