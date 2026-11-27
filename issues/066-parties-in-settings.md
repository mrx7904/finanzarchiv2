# #66 Parties in the window

**Goal:** The owner sees every party with its spellings and its category, joins two that are one and parts two that are not.
**Requirements**: FA-66 · **Depends on**: #25, #49

Parties with their spellings carry the categories, the recurring payments and the evaluations (FA-30), but as of 0.7.0 no view lists them and no command reaches them: the owner sets a category through one booking ("Für alle von …") and cannot see or mend which spellings were taken as one party. A wrong join puts every figure by party wrong.

To verify before the work: what `src-tauri/src/parties.rs` already offers for joining and parting, and what `knowledge/decision-parties.md` decided about corrections.

## Done when

- [ ] A page "Partner" in Einstellungen lists the parties with their category, the count of their bookings and documents, searched by name or spelling.
- [ ] A chosen party shows its spellings, with where each came from (a booking, a document), and its bookings as a jump to "Umsätze".
- [ ] "Umbenennen …" sets the name shown. A statement or document read later keeps it.
- [ ] "Zusammenführen …" joins the chosen party with another: spellings, bookings, documents and recurring payments follow, the category of the one the owner keeps counts. "Schreibweise abtrennen" makes a spelling its own party again. Both are decisions with "Rückgängig".
- [ ] The category of a party is set here, with the mark whether the owner or the model chose it (FA-46). A category the owner chose is never changed by the model.
- [ ] Evaluations, recurring payments and hints follow a join or a parting at once.
- [ ] Tests in the core with invented parties: a join, its undoing, a parting, the category that counts, the sums before and after.

## Out of scope

- Rules that file by party: stage 2 owns the checks.
