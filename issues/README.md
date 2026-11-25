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
| [#04](004-document-store.md) | Unchanged originals under their hash, in by drop and dialog | FA-07, FA-08 | #02, #03 | open |
| [#05](005-intake-queue.md) | One intake queue that survives a restart | FA-09 | #04 | open |
| [#06](006-pdf-text-and-pages.md) | Text and pages of a PDF in the core | FA-10 | #05 | open |
| [#07](007-design-system.md) | The design system, chosen by the owner | FA-11 | #01 | done |
| [#08](008-local-releases.md) | Releases built on the owner's machines | FA-12 | #01, #06 | open |
| [#09](009-corpus.md) | A generated corpus and a measure command | FA-13 | #06 | open |

**Stage 1 is done when the owner installs 2.0 on the Windows PC and on the Mac from a release, drops 20 real PDFs, sees each in the list with its first page, quits in the middle of the import, and after the next start finds every file once.**

Order of work: #01 first. Then #02, #03 and #07 in parallel. Then #04, #05 and #06, then #08 and #09.

## Later stages

Stages 2 to 6 of `spec/concept.md` are cut into issues when their turn comes, with what stage 1 taught.
