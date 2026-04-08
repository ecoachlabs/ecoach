ALTER TABLE accounts
ADD COLUMN pin_length INTEGER NOT NULL DEFAULT 4 CHECK (pin_length = 4);
