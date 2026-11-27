# finanzarchiv2

Routing table only. Rules, stack and domain knowledge live in the files below. Read the routed file before acting and add no content here.

## Task routing

| Situation | Approach |
|---|---|
| **Any task, before starting** | [`knowledge/conventions.md`](./knowledge/conventions.md) |
| An issue from `issues/` is referenced | `issue-analysis` skill, then `implementation-workflow` skill |
| Free implementation request without an issue | `implementation-workflow` skill |
| Broken behaviour or a failing test with unknown cause | `debugging` skill, then the fix via `implementation-workflow` |
| A requirement needs a test plan before code | `test-strategy-planner` skill, then `acceptance-test-builder` |
| UI work | `frontend-design` skill with the `ui-libraries` skill for the six UI sources and their links |
| New requirements need to be cut into issues | `issue-writing` skill |
| Something is useful beyond this project | `knowledge-promotion` skill |
| Documentation or research only | No full workflow needed |

The skills come from the vault plugin `agentos` (the vault's `skills/`), enabled in [`.claude/settings.json`](./.claude/settings.json). They load as `agentos:<name>`. The path of the vault on a machine stands in `.claude/settings.local.json`, see [`knowledge/conventions.md`](./knowledge/conventions.md).

## Knowledge routing

| When you | Read |
|---|---|
| implement or modify anything | [`knowledge/conventions.md`](./knowledge/conventions.md) |
| need the stack, a version or the split between core and UI | [`knowledge/decision-stack.md`](./knowledge/decision-stack.md) |
| touch the schema, money, documents, bookings, links or the provenance of a value | [`knowledge/decision-data-model.md`](./knowledge/decision-data-model.md) |
| touch the database file, a migration, the start check, snapshots or `Cents` | [`knowledge/decision-archive-file.md`](./knowledge/decision-archive-file.md) |
| touch the kind of a document, a reader, or what is read how | [`knowledge/decision-reading.md`](./knowledge/decision-reading.md) |
| choose, measure or change the local model, or touch `model.name` | [`knowledge/decision-model.md`](./knowledge/decision-model.md) |
| touch a check, filing by itself, `decisions`, "Warum?", "Rückgängig" or "Prüfen" | [`knowledge/decision-checks.md`](./knowledge/decision-checks.md) |
| touch the name of a document, its templates or "Umbenennen" | [`knowledge/decision-names.md`](./knowledge/decision-names.md) |
| touch a statement, its layouts, `accounts`, `statements`, `bookings` or the balance of an account | [`knowledge/decision-statements.md`](./knowledge/decision-statements.md) |
| touch a party, its spellings, a category, the categories by the model, the payee of a PayPal booking or a correction | [`knowledge/decision-parties.md`](./knowledge/decision-parties.md) |
| touch a watched folder, the inbox folder or the downloads | [`knowledge/decision-folders.md`](./knowledge/decision-folders.md) |
| touch a mailbox, IMAP, the app password, "Postfächer" or invoices from mail | [`knowledge/decision-mail.md`](./knowledge/decision-mail.md) |
| touch PayPal payments, a receipt paired with its invoice, Amazon orders or "Bestellungen" | [`knowledge/decision-payments.md`](./knowledge/decision-payments.md) |
| touch income and spending, the evaluations or their figures | [`knowledge/decision-evaluations.md`](./knowledge/decision-evaluations.md) |
| touch recurring payments, their rhythm, a changed amount or "ist keine" | [`knowledge/decision-recurring.md`](./knowledge/decision-recurring.md) |
| touch the salary, the months of the payslips or a missing month | [`knowledge/decision-salary.md`](./knowledge/decision-salary.md) |
| touch balances over time, the sum of all accounts or the balance at the end of a month | [`knowledge/decision-balances.md`](./knowledge/decision-balances.md) |
| touch the savings hints, their kinds or dismissing one | [`knowledge/decision-hints.md`](./knowledge/decision-hints.md) |
| touch taxes, a tax assignment, the values of a tax year or the tax profile | [`knowledge/decision-taxes.md`](./knowledge/decision-taxes.md) |
| generate the corpus, measure a reader or add test documents | [`knowledge/decision-corpus.md`](./knowledge/decision-corpus.md) |
| touch the intake queue, its stages, workers or the pause | [`knowledge/decision-intake.md`](./knowledge/decision-intake.md) |
| open, check or read a PDF, or touch PDFium and how it ships | [`knowledge/decision-pdf.md`](./knowledge/decision-pdf.md) |
| build, package or release, or think about CI | [`knowledge/decision-local-builds.md`](./knowledge/decision-local-builds.md) |
| add a file, a fixture, a test document or anything else to the repository | [`knowledge/decision-public-repository.md`](./knowledge/decision-public-repository.md) |
| touch the look, motion or a UI library | [`knowledge/decision-ui-libraries.md`](./knowledge/decision-ui-libraries.md) |
| build or change a view, the sidebar, a dialog, a menu, an empty, loading or error state, or decide where an action or a setting stands | [`knowledge/decision-views.md`](./knowledge/decision-views.md) |
| touch the top bar, the window frame, the window buttons or the window's permissions | [`knowledge/decision-app-shell.md`](./knowledge/decision-app-shell.md) |
| need what version 1.0 did and what it taught | [`docs/analyse-finanzarchiv-1.0.md`](./docs/analyse-finanzarchiv-1.0.md) and the repository `mrx7904/finanzarchiv` (private) |
| make or challenge a design decision | `knowledge/decision-*.md` |
| need what happened | the git history, the commit messages carry the evidence |

## Specification routing

| When you | Read |
|---|---|
| need the product intent | [`spec/concept.md`](./spec/concept.md) |
| need what is required | [`spec/requirements.md`](./spec/requirements.md) |
| need a work unit, its Definition of Done or its status | [`issues/README.md`](./issues/README.md) |

## Vault routing

| When you | Read |
|---|---|
| need the requirements and conventions of AgentOS | `~/Entwicklung/AgentOS/essentials.md` |
| need the vault map | `~/Entwicklung/AgentOS/INDEX.md` |
