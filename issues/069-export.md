# #69 Taking a list and the documents of a tax year out of the archive

**Goal:** The owner hands the tax year to the return, and any list to a spreadsheet, as files on the computer.
**Requirements**: FA-69 · **Depends on**: #41, #46, #64

As of 0.7.0 nothing leaves the archive but a single file shown in its folder. "Steuern" collects a year (FA-44) and the owner then has to copy sums and find each document by hand. 1.0 exported the list and the documents of a tax year. "Als Tabelle exportieren" was drawn and left to a requirement of its own (#46, `knowledge/decision-views.md`, "Known limits").

## Done when

- [ ] "Steuern" has "Exportieren …" for the chosen year: a folder the owner picks receives one table with every entry (section, form and field in ELSTER, day, party or sender, gross, the amount counted, the notes) and the sums per section, and a copy of every document of the year under its name, sorted into a folder per section.
- [ ] The table is a CSV that opens in Excel and Numbers with umlauts and German amounts right: the encoding, the separator and the form of amounts and dates are decided here, recorded in `knowledge/decision-taxes.md` and proved by a test that reads the file back.
- [ ] The notice "Keine Steuerberatung, bitte alles selbst prüfen" stands in the first line of the table.
- [ ] Dokumente and "Umsätze" have "Als Tabelle exportieren …" in the menu "…" of the top bar: the rows the filters leave, with the columns the list and the detail show.
- [ ] Originals are copied, never moved, and the archive is unchanged after an export: a test compares the hash of the database content before and after.
- [ ] An export writes only where the owner pointed, refuses a folder that is not empty unless the owner agrees, and says in one line what it wrote with "Im Ordner zeigen".
- [ ] Nothing is sent anywhere: no new row in "Outside connections".
- [ ] Tests in the core with an invented tax year: the sums of the table equal those of the view to the cent, every entry has its file.

## Out of scope

- Moving the whole archive to another computer: stage 6.
- A PDF report or printing.
- Filling ELSTER.
