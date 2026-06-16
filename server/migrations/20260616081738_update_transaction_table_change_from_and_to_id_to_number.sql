-- Add migration script here
ALTER TABLE transactions DROP CONSTRAINT transactions_from_account_id_fkey;
ALTER TABLE transactions DROP CONSTRAINT transactions_to_account_id_fkey;

ALTER TABLE transactions RENAME COLUMN from_account_id TO from_account_number;
ALTER TABLE transactions ALTER COLUMN from_account_number TYPE TEXT USING from_account_number::text;

ALTER TABLE transactions RENAME COLUMN to_account_id TO to_account_number;
ALTER TABLE transactions ALTER COLUMN to_account_number TYPE TEXT USING to_account_number::text;

ALTER TABLE transactions 
  ADD CONSTRAINT transactions_from_account_number_fkey FOREIGN KEY (from_account_number) REFERENCES accounts(account_number) ON DELETE CASCADE,
  ADD CONSTRAINT transactions_to_account_number_fkey FOREIGN KEY (to_account_number) REFERENCES accounts(account_number) ON DELETE CASCADE;