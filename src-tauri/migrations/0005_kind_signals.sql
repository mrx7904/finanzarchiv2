-- The signals the kind of a document rests on (FA-14), as a JSON array of the matched words.
ALTER TABLE documents ADD COLUMN kind_signals TEXT;
