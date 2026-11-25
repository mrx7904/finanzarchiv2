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
