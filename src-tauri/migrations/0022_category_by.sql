-- Categories by the model (FA-46, #43): who gave a party its category, and when the model was
-- asked, so a party is asked once and a category the owner chose is never changed.
ALTER TABLE parties ADD COLUMN category_by TEXT CHECK (category_by IN ('model', 'owner'));
ALTER TABLE parties ADD COLUMN category_asked_at TEXT;
UPDATE parties SET category_by = 'owner' WHERE category_id IS NOT NULL;
