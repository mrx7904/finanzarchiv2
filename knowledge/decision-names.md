# Decision: names by template

**Decided:** 2026-11-26 (#18, FA-23) · applies to `src-tauri/src/naming.rs`, `documents.name`, `documents.name_by_owner`, the settings `name.template.<kind>`

## Decision

- A document is named when it is filed (`checks::decide` calls `naming::name_document`). The name is built by code from the values that count (`checks::chosen`), never by a model. The file in the archive keeps its hash name, only `documents.name` changes.
- One template per kind, the defaults in `naming::default_template`, date first as the owner chose in 1.0:
  - invoice and receipt `{datum}_{absender}_{thema}`
  - payslip `{monat}_{absender}_Gehaltsabrechnung`
  - statement `{datum}_{absender}_Kontoauszug`
  - other and unknown keep the name of their file.
  The owner changes a template in "Namensvorlagen …" (setting `name.template.<kind>`), an empty one restores the default, and the filed documents of that kind are named again.
- Placeholders:
  - `{datum}`: the date as `YYYY-MM-DD`.
  - `{monat}`: `YYYY-MM`.
  - `{absender}`: the sender or employer without its legal form, for example GmbH, AG, KG, e.V., UG (haftungsbeschränkt) or GmbH & Co. KG.
  - `{thema}`: the kind with its number, for example "Rechnung R-1".
  - `{nummer}`
  - `{betrag}`: the gross or the Netto as "1.234,56 €".

  An unknown placeholder or a template without any is refused with its reason.
- A placeholder without a value is left out with its separator. A name with no value of the document in it (only the kind's word) is no name: the document keeps the name of its file. So the statements keep their file names until stage 3 reads their dates and banks.
- Characters a file system of Windows, macOS or Linux refuses (`/ \ : * ? " < > |` and control characters) become spaces, spaces fold, dots and spaces at the end go, at most 120 characters. A property test holds that no name contains one.
- A name the owner types ("Umbenennen" in the preview) sets `name_by_owner` and is kept, no template writes over it. An empty name gives the document back to its template.
- The list shows the name, the preview the name and the original file name beside it.
