-- The downloads folder (FA-33, #28): a watched folder that takes only what looks like a document of
-- money, and the verdict on each file it looked at, with the reason.
ALTER TABLE watched_folders ADD COLUMN is_downloads INTEGER NOT NULL DEFAULT 0 CHECK (is_downloads IN (0, 1));
ALTER TABLE seen_files ADD COLUMN verdict TEXT;
