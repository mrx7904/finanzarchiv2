-- An order the owner said needs no invoice (#46): cancelled, a gift, a shop without invoices. It no
-- longer counts as missing one. NULL: an invoice is expected. An invoice that comes after all is
-- linked as before and counts over it.
ALTER TABLE orders ADD COLUMN no_invoice_at TEXT;
