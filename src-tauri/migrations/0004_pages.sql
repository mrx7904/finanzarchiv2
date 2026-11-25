-- The text of each page with the box of each line (FA-10), in PDF points from the lower left corner.
-- text_state of a document: 'text' when a page carries text, 'scan' for a PDF without any, 'image' for
-- an image file. NULL until the stage `text` of the intake has run.
ALTER TABLE documents ADD COLUMN text_state TEXT CHECK (text_state IN ('text', 'scan', 'image'));

CREATE TABLE pages (
    document_id INTEGER NOT NULL REFERENCES documents (id),
    number INTEGER NOT NULL CHECK (number >= 1),
    width REAL NOT NULL,
    height REAL NOT NULL,
    PRIMARY KEY (document_id, number)
) STRICT;

CREATE TABLE page_lines (
    document_id INTEGER NOT NULL,
    page INTEGER NOT NULL,
    line INTEGER NOT NULL CHECK (line >= 1),
    text TEXT NOT NULL,
    left REAL NOT NULL,
    bottom REAL NOT NULL,
    right REAL NOT NULL,
    top REAL NOT NULL,
    PRIMARY KEY (document_id, page, line),
    FOREIGN KEY (document_id, page) REFERENCES pages (document_id, number)
) STRICT;
