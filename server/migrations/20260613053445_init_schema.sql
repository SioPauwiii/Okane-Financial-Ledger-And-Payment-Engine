-- Add migration script here
-- Users table

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    country VARCHAR(255),
    province VARCHAR(255),
    city VARCHAR(255),
    street VARCHAR(255),
    house_no VARCHAR(50),
    zip_code VARCHAR(20),
    birth_date DATE,
    sex VARCHAR(10),
    nationality VARCHAR(255),
    password VARCHAR(255) NOT NULL,
    user_type VARCHAR(20) NOT NULL DEFAULT 'customer', -- customer, admin
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Accounts table

CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    account_number TEXT UNIQUE NOT NULL,
    account_type VARCHAR(50) NOT NULL,
    balance NUMERIC(30, 2) DEFAULT 0.00, 
    currency VARCHAR(10) NOT NULL DEFAULT 'PHP',
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, frozen, suspended, closed
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Transactions table

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    from_account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    to_account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    amount_transfered NUMERIC(30, 2) NOT NULL,
    transaction_type VARCHAR(20) NOT NULL DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT now()
)