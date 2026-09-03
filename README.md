# 💴 Okane (お金) — High-Performance Financial Ledger & Payment Processing Engine

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8.9-blue.svg?style=flat-square)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16+-blue.svg?style=flat-square&logo=postgresql)](https://www.postgresql.org/)
[![SQLx](https://img.shields.io/badge/SQLx-0.9.0-lightgrey.svg?style=flat-square)](https://github.com/launchbadge/sqlx)
[![Tokio](https://img.shields.io/badge/Tokio-1.52.3-purple.svg?style=flat-square)](https://tokio.rs/)
[![Security](https://img.shields.io/badge/Security-Argon2--id_%7C_HMAC--SHA256-green.svg?style=flat-square)](#security-architecture)

**Okane** is a enterprise-grade, asynchronous financial backend service and immutable ledger built with **Rust**, **Axum**, and **PostgreSQL**. Designed for high throughput, strict security, and zero floating-point rounding errors, Okane powers secure user account management, internal peer-to-peer fund transfers, and real-time e-wallet / card deposits via integration with the **PayMongo API v2**.

---

## 📌 Executive Summary

Modern financial systems demand absolute precision, strict cryptographic validation, and memory safety without sacrificing throughput. Okane is engineered from the ground up to solve critical financial engine challenges:

- **Zero-Loss Financial Math**: Utilizes fixed-precision decimal arithmetic (`rust_decimal` mapped to PostgreSQL `NUMERIC(30, 2)`), completely eliminating IEEE 754 floating-point inaccuracies.
- **Double-Entry Dynamic Balance Auditing**: Account balances are dynamically computed from an immutable ledger of completed transactions, ensuring tamper-evident balance state verification.
- **PayMongo Payment Gateway Integration**: Native integration with PayMongo Checkout Sessions (GCash, Maya, Credit/Debit Cards) featuring asynchronous, webhook-driven transaction settlement.
- **Cryptographic Webhook Verification**: Protects payment settlement endpoints against forgery using HMAC-SHA256 signature verification with constant-time string comparison.
- **Stateless & Secure Session Management**: Implements Argon2 password hashing and HTTP-Only, SameSite, type-safe JWT cookies for secure cross-origin frontend communication.

---

## 🏗️ System Architecture

Okane follows a clean, modular, layered architecture adhering to the **Separation of Concerns (SoC)** principle.

```
                    ┌────────────────────────────────────────┐
                    │          HTTP Client / Web App         │
                    └───────────────────┬────────────────────┘
                                        │ (REST / HTTPS / Cookies)
                                        ▼
 ┌────────────────────────────────────────────────────────────────────────────────────┐
 │  Axum Router Layer (src/app.rs & src/routes/)                                      │
 │  ├── /api/auth          (Registration, Authentication)                             │
 │  ├── /api/user          (Profile Retrieval)                                        │
 │  ├── /api/account       (Balances, Deposits, Withdrawals, Transfers, Webhooks)    │
 │  └── /api/transaction   (Ledger History)                                           │
 └──────────────────────────────────────┬─────────────────────────────────────────────┘
                                        │
                                        ▼
 ┌────────────────────────────────────────────────────────────────────────────────────┐
 │  Handlers Layer (src/handlers/)                                                    │
 │  ├── Extractors: State, CookieJar, Headers, Bytes, Json<T>                        │
 │  └── Response Builders: Status Codes, JSON DTOs, HTTP Error Mapping                │
 └──────────────────────────────────────┬─────────────────────────────────────────────┘
                                        │
                                        ▼
 ┌────────────────────────────────────────────────────────────────────────────────────┐
 │  Services Layer (src/services/)                                                    │
 │  ├── auth_services       (Argon2 Hashing, JWT Claim Generation)                    │
 │  ├── accounts_services   (Ledger Balance Calculation, Deposit/Transfer Workflows)  │
 │  ├── paymongo_services   (Checkout API, HMAC-SHA256 Webhook Verification)          │
 │  └── transactions_services (Immutable Record Creation, Audit Logs)                  │
 └───────────────────┬────────────────────────────────────────┬───────────────────────┘
                     │                                        │
                     ▼                                        ▼
 ┌──────────────────────────────────────┐  ┌──────────────────────────────────────────┐
 │  PostgreSQL Database (SQLx Pool)     │  │  PayMongo API v2 Gateway                 │
 │  ├── users table                     │  │  ├── Checkout Sessions API               │
 │  ├── accounts table                  │  │  └── Asynchronous Payment Webhooks       │
 │  └── transactions table              │  └──────────────────────────────────────────┘
 └──────────────────────────────────────┘
```

### Architectural Modules

| Layer | Path | Responsibility |
| :--- | :--- | :--- |
| **Server Initialization** | `src/main.rs` | Environment loading, PostgreSQL connection pool (`PgPool`) setup, PayMongo credentials binding, Tokio TCP binding. |
| **Router Configuration** | `src/app.rs` | HTTP router assembly, CORS middleware definition, route nesting (`/api/*`), HTTP request tracing. |
| **Routes** | `src/routes/` | Declarative mapping of HTTP methods and endpoints to handler functions. |
| **Handlers** | `src/handlers/` | Request extraction, HTTP validation, cookie parsing, header extraction, and HTTP status response mapping. |
| **Domain Services** | `src/services/` | Core business logic, financial validation, external payment gateway HTTP client interactions, cryptographic operations. |
| **Data Models** | `src/models/` | Type-safe Rust structs representing database rows, derived with `sqlx::FromRow` and `serde`. |
| **DTO Requests/Responses**| `src/requests/`, `src/responses/` | Strictly typed payload definitions for request deserialization and response serialization. |
| **Error Handling** | `src/errors.rs` | Centralized `AppError` enum implementing Axum's `IntoResponse` for uniform JSON error schemas. |

---

## 🔄 Data Flow & Payment Lifecycle

### PayMongo Deposit & Webhook Settlement Flow

The deposit workflow bridges synchronous user requests with asynchronous payment gateway notifications to guarantee ledger integrity.

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant App as Okane Backend
    participant DB as PostgreSQL DB
    participant PM as PayMongo API

    User->>App: POST /api/account/deposit { amount, account_number } (with Auth Cookie)
    App->>App: Validate JWT & Account Status
    App->>DB: INSERT INTO transactions (type='deposit', status='pending', uuid=gen_random_uuid())
    DB-->>App: Return pending Transaction (with transaction_uuid)
    App->>PM: POST /v2/checkout_sessions (Amount in centavos, metadata: { account_number, transaction_uuid })
    PM-->>App: Return checkout_url
    App-->>User: 200 OK { checkout_url, transaction }
    
    User->>PM: User opens checkout_url & pays via GCash / Maya / Card
    PM->>App: POST /api/account/webhook/paymongo (Headers: paymongo-signature, Body: raw JSON)
    
    App->>App: Verify HMAC-SHA256 signature against paymongo_webhook_secret
    alt Signature Invalid
        App-->>PM: 401 Unauthorized
    else Signature Valid
        App->>App: Extract metadata (account_number, transaction_uuid, amount_in_centavos)
        App->>DB: INSERT INTO transactions (type='deposit', status='completed', transaction_uuid=uuid)
        DB-->>App: Record created
        App->>DB: UPDATE accounts balance (computed from completed transactions)
        App-->>PM: 200 OK { received: true }
    end
```

---

## 🗄️ Database Architecture & Schema

The system uses **PostgreSQL** managed via compile-time verified **SQLx** queries and raw SQL migration files (`/server/migrations`).

```mermaid
erdiagram
    USERS ||--o{ ACCOUNTS : "owns"
    ACCOUNTS ||--o{ TRANSACTIONS : "source of (from_account)"
    ACCOUNTS ||--o{ TRANSACTIONS : "destination of (to_account)"

    USERS {
        BIGINT id PK
        VARCHAR email UK
        VARCHAR first_name
        VARCHAR last_name
        VARCHAR country
        VARCHAR city
        VARCHAR street
        VARCHAR house_no
        VARCHAR zip_code
        VARCHAR contact_no
        DATE birth_date
        VARCHAR sex
        VARCHAR nationality
        VARCHAR password "Argon2 Hash"
        VARCHAR user_type "customer | admin"
        TIMESTAMP created_at
        TIMESTAMP updated_at
    }

    ACCOUNTS {
        INTEGER id PK
        INTEGER user_id FK
        VARCHAR account_number UK "10-digit random string"
        VARCHAR account_type "savings | checking"
        NUMERIC balance "30, 2"
        VARCHAR currency "PHP"
        VARCHAR status "active | frozen | closed"
        TIMESTAMP created_at
        TIMESTAMP updated_at
    }

    TRANSACTIONS {
        INTEGER id PK
        UUID transaction_uuid "Shared UUID linking pending and completed entries"
        VARCHAR from_account_number FK
        VARCHAR to_account_number FK
        NUMERIC amount_transferred "30, 2"
        VARCHAR transaction_type "deposit | withdrawal | transfer"
        VARCHAR status "pending | completed | failed"
        TIMESTAMP created_at
    }
```

### Key Schema Optimizations

1. **Foreign Key Cascade Constraints**: Deleting a user automatically purges linked account and transaction logs via `ON DELETE CASCADE`.
2. **Precision Numeric Column**: `balance` and `amount_transferred` use `NUMERIC(30, 2)` to retain exact monetary precision without floating-point drift.
3. **Transaction Pair Correlation**: The `transaction_uuid` column links initial `pending` deposit records with their corresponding `completed` webhook settlement entries.

---

## 🔒 Security Architecture

Okane incorporates multiple defense-in-depth security layers crucial for financial applications:

### 1. Argon2id Password Hashing
Password credentials are hashed using `argon2` (v0.5.3) with a cryptographically secure 16-byte B64 salt generated per user via `rand::random()`:
```rust
let salt = SaltString::encode_b64(&bytes).expect("failed to create salt");
let hashed = Argon2::default().hash_password(password.as_bytes(), &salt)?.to_string();
```

### 2. Cryptographic Webhook Authentication (HMAC-SHA256)
PayMongo webhook notifications are validated to prevent spoofed deposits. The backend extracts `t` (timestamp) and `te` (test/live signature) from `paymongo-signature` headers, reconstructs the signature payload `<timestamp>.<raw_body>`, and computes the expected HMAC-SHA256 signature using `hmac` and `sha2`:
```rust
let mut signed_payload = timestamp.into_bytes();
signed_payload.push(b'.');
signed_payload.extend_from_slice(raw_body);

let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
mac.update(&signed_payload);
let computed = hex::encode(mac.finalize().into_bytes());

// Constant-time string comparison against header signature
computed == te_signature
```

### 3. JWT Authentication & HTTP-Only Cookies
Authentication sessions use JSON Web Tokens (`jsonwebtoken`) transmitted via HTTP-only, `SameSite=None` cookies (`axum-extra`), neutralizing client-side Cross-Site Scripting (XSS) token theft:

| JWT Claim | Description |
| :--- | :--- |
| `sub` | User ID (`i64`) |
| `email` | Registered User Email |
| `account_number` | User's Primary Financial Account Number |
| `user_type` | User Role (`customer`, `admin`) |
| `exp` | Token Expiration Unix Timestamp (7-day duration) |

---

## 📡 API Reference Specification

Base URL: `http://localhost:3000/api`

### 1. Authentication Endpoints (`/api/auth`)

#### `POST /api/auth/register`
Registers a new user, hashes the password, automatically provisions a 10-digit savings account in `PHP`, and issues a JWT session token.

- **Request Body**:
```json
{
  "email": "user@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "country": "Philippines",
  "city": "Manila",
  "street": "Ayala Ave",
  "house_no": "123",
  "zip_code": "1200",
  "contact_no": "+639171234567",
  "birth_date": "1995-08-15",
  "sex": "Male",
  "nationality": "Filipino",
  "password": "SecurePassword123!",
  "confirm_password": "SecurePassword123!"
}
```
- **Response (`200 OK`)**:
```json
{
  "message": "User registered successfully",
  "user": {
    "id": 1,
    "email": "user@example.com",
    "first_name": "John",
    "last_name": "Doe",
    "user_type": "customer",
    "created_at": "2026-09-03T12:00:00"
  },
  "account": {
    "id": 1,
    "account_number": "8472910482",
    "account_type": "savings",
    "currency": "PHP",
    "balance": "0.00",
    "status": "active"
  },
  "access_token": "eyJhbGciOiJIUzI1Ni..."
}
```

#### `POST /api/auth/login`
Authenticates user credentials against stored Argon2 hashes and returns an access token / sets HTTP cookie.

---

### 2. User & Account Management (`/api/user` & `/api/account`)

#### `GET /api/user/me`
Retrieves authenticated user profile details. Requires valid `access_token` cookie.

#### `GET /api/account/my-account`
Calculates dynamic balance from completed ledger transactions, updates cached account balance, and returns account details with owner metadata.

#### `POST /api/account/deposit`
Initiates an e-wallet or credit card deposit via PayMongo Checkout Session V2.

- **Request Body**:
```json
{
  "account_number": "8472910482",
  "amount": 500.00
}
```
- **Response (`200 OK`)**:
```json
{
  "message": "Deposit initiated. Complete payment at the provided URL.",
  "checkout_url": "https://checkout.paymongo.com/cs_test_abc123...",
  "transaction": {
    "id": 42,
    "transaction_uuid": "c39a8e94-3b1a-4d76-92a1-0f81d1912903",
    "amount_transferred": "500.00",
    "to_account_number": "8472910482",
    "transaction_type": "deposit",
    "status": "pending",
    "created_at": "2026-09-03T12:05:00"
  }
}
```

#### `POST /api/account/transfer`
Executes an internal peer-to-peer balance transfer between two active accounts.

- **Request Body**:
```json
{
  "target_account_number": "9102847361",
  "amount": 150.00
}
```

#### `POST /api/account/withdraw`
Records an account withdrawal transaction.

#### `POST /api/account/webhook/paymongo`
Public webhook endpoint for receiving PayMongo payment status callbacks. Performs HMAC-SHA256 signature validation and converts pending transactions into completed ledger entries.

---

### 3. Transaction History (`/api/transaction`)

#### `GET /api/transaction/my-transactions`
Retrieves the user's complete transaction audit trail (deposits, withdrawals, transfers) ordered by creation date descending.

---

## ⚡ Error Handling Architecture

The backend centralizes error handling into an `AppError` enum implementing Axum's `IntoResponse` trait. Database errors from `SQLx` automatically convert into internal server errors:

```rust
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    InternalServerError(String),
    BadRequest(String),
}
```

### Standardized Error Format (`Json`)
```json
{
  "error": "Detailed error message here"
}
```

| HTTP Status | Trigger Condition |
| :--- | :--- |
| `400 Bad Request` | Invalid JSON, invalid transaction amount ($\le 0$), age validation failed ($<18$), mismatched passwords |
| `401 Unauthorized` | Invalid/missing JWT cookie, invalid login credentials, invalid PayMongo webhook HMAC signature |
| `404 Not Found` | Non-existent user, missing financial account, inactive account status |
| `500 Internal Server Error` | Database connection failures, PayMongo API call timeout, serialization failures |

---

## 💻 Tech Stack & System Requirements

### Core Dependencies (`Cargo.toml`)

- **Language / Runtime**: Rust (2024 edition) with `tokio` (v1.52.3) multi-threaded async executor
- **Web Framework**: `axum` (v0.8.9) with `tower-http` CORS and HTTP tracing
- **Database Connection**: `sqlx` (v0.9.0) with PostgreSQL native TLS, compile-time query checking, `rust_decimal`, and `uuid` support
- **Cryptography & Security**: `argon2` (v0.5.3), `jsonwebtoken` (v10.4.0), `hmac` (v0.13.0), `sha2` (v0.11.0), `hex` (v0.4.3), `rand` (v0.10.1)
- **Financial Precision Math**: `rust_decimal` (v1.42.1)
- **External HTTP Client**: `reqwest` (v0.13.4) with JSON support

---

## 🚀 Getting Started & Local Development

### Prerequisites

- **Rust Toolchain**: `rustc` 1.85+ and `cargo` installed ([rustup.rs](https://rustup.rs/))
- **Database**: PostgreSQL 14+ database instance
- **SQLx CLI**: Installed via `cargo install sqlx-cli --no-default-features --features postgres`

### Setup Instructions

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/SioPauwiii/okane.git
   cd okane/server
   ```

2. **Configure Environment Variables**:
   Create a `.env` file in the `/server` root directory:
   ```env
   DATABASE_URL=postgres://postgres:password@localhost:5432/okane_db
   JWT_SECRET=your_super_secret_jwt_key_here
   PAYMONGO_SECRET_KEY=sk_test_your_paymongo_secret_key
   PAYMONGO_WEBHOOK_SECRET=whsk_your_paymongo_webhook_secret
   FRONTEND_ORIGIN=http://localhost:8081
   ADDR=127.0.0.1:3000
   COOKIE_SECURE=false
   ```

3. **Run Database Migrations**:
   ```bash
   sqlx database create
   sqlx migrate run
   ```

4. **Start Development Server**:
   ```bash
   cargo run
   ```
   The server will start listening on `http://127.0.0.1:3000`. Test server health:
   ```bash
   curl http://127.0.0.1:3000/health
   # Returns: OK
   ```

---

## 👨‍💻 Key Engineering & Design Decisions

1. **Why Rust for Financial Services?**
   Rust delivers C-like performance with compile-time memory safety, eliminating buffer overflows, data races, and null pointer exceptions—critical guarantees when handling monetary assets.

2. **Dynamic Ledger Computation vs. Static Column Mutation**
   Rather than performing destructive column mutations directly on user balance cells, Okane dynamically computes balances from immutable transaction logs. This prevents race conditions and balance desynchronization.

3. **Fixed-Point Decimal Over Floating-Point**
   Using `rust_decimal` prevents classical binary floating-point representation bugs (e.g. `0.1 + 0.2 != 0.3`). SQLx natively binds `Decimal` to PostgreSQL's `NUMERIC(30, 2)` type for absolute financial precision.

4. **Webhook Replay Protection & Idempotency**
   By embedding a unique `transaction_uuid` inside PayMongo request metadata, the webhook handler guarantees that pending deposit records are explicitly paired with completed settlement records, preventing duplicate balance credits.

---

## 📜 License

This project is open-source and available under the [MIT License](LICENSE).
