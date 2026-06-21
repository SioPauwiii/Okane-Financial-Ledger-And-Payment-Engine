-- Add migration script here
ALTER TABLE transactions 
ADD COLUMN transaction_uuid UUID DEFAULT gen_random_uuid();