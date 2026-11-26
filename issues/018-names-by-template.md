# #18 Names by template

**Goal:** A filed document is named by the template of its kind, and the owner can change the templates and rename a document by hand.
**Requirements**: FA-23 · **Depends on**: #16

## Done when

- [x] One template per kind with placeholders (date first, as the owner chose in 1.0: for example `{datum}_{absender}_{thema}`), defaults in one place in the core.
- [x] The name is built by code from confirmed values, the legal form left out of the sender, characters a file system refuses removed. Tested with generated values and property tests that a name never holds `/` or `\`.
- [x] A name typed by the owner is kept and never overwritten by a template.
- [x] The list and the preview show the name, the original file name stays in the preview.

## Out of scope

- Renaming files on disk, the originals keep their hash name
