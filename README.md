# Rust Axum REST API

A production-ready, modular RESTful API built in Rust using **Axum**, **SQLx** (PostgreSQL), **Argon2** password hashing, **JWT authentication** with refresh token rotation and version revocation, and interactive OpenAPI documentation powered by **Utoipa** and **Scalar**.

---

## Features

- **Asynchronous Architecture**: Built on top of [Axum](https://github.com/tokio-rs/axum) and [Tokio](https://tokio.rs).
- **PostgreSQL Persistence**: Database operations managed via [SQLx](https://github.com/launchbadge/sqlx) with compile-time embedded migrations located in `src/database/migrations/`.
- **Authentication & Security**:
  - Secure password hashing using **Argon2**.
  - Dual-token workflow: short-lived **Access Tokens** (15 min) and persistent **Refresh Tokens** (7 days).
  - Refresh token rotation and single-session / multi-session invalidation via `token_version`.
  - Protected endpoints with custom `BearerToken` extractor.
- **User CRUD**: Complete endpoints to register, list, fetch, update, and delete users.
- **Interactive API Documentation**:
  - OpenAPI 3.0 specification generated with [Utoipa](https://github.com/juhaku/utoipa).
  - Modern, responsive UI powered by [Scalar](https://scalar.com) at `/docs`.
  - Raw JSON OpenAPI schema available at `/api-docs/openapi.json`.
- **E2E & Unit Testing**: Comprehensive automated test suite with automated migration execution and database isolation.
- **Docker Ready**: Production and development Docker Compose configurations with managed PostgreSQL services and health checks.

---

## Tech Stack

| Technology | Purpose |
|------------|---------|
| [Rust](https://www.rust-lang.org/) (Edition 2024) | Core language |
| [Axum](https://github.com/tokio-rs/axum) | Web application framework |
| [Tokio](https://tokio.rs/) | Asynchronous runtime |
| [SQLx](https://github.com/launchbadge/sqlx) | Async SQL database toolkit (PostgreSQL) |
| [PostgreSQL](https://www.postgresql.org/) | Relational database |
| [Argon2](https://github.com/RustCrypto/password-hashes) | Password hashing algorithm |
| [jsonwebtoken](https://github.com/Keats/jsonwebtoken) | JWT creation and verification |
| [Utoipa](https://github.com/juhaku/utoipa) | OpenAPI schema generation |
| [Scalar](https://scalar.com) | Interactive API documentation UI |

---

## Getting Started

### Prerequisites

- [Rust & Cargo](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Docker](https://www.docker.com/) and [Docker Compose](https://docs.docker.com/compose/) (or a running PostgreSQL instance)

### 1. Environment Configuration

Copy the example environment file and configure your variables:

```bash
cp .env.example .env
```

Ensure `DATABASE_URL` is configured to point to your PostgreSQL database:

```env
DATABASE_URL="postgres://postgres:postgres@localhost:5432/app_db"
```

### 2. Run Locally with Cargo

Start the PostgreSQL database service using Docker Compose:

```bash
docker compose up -d db
```

Run the server locally:

```bash
cargo run
```

The server will automatically apply pending migrations from `src/database/migrations/` and start listening on:

```
http://127.0.0.1:3000
```

### 3. Run with Docker Compose

To run the entire stack (API + PostgreSQL) inside Docker:

```bash
# Production mode
docker compose up --build

# Development mode (with live reload)
docker compose -f docker-compose.dev.yml up --build
```

---

## Database Migrations

Migrations are stored in [`src/database/migrations/`](file:///c:/Users/mixbr/OneDrive/Área%20de%20Trabalho/VSCode/rust/src/database/migrations/) and executed automatically on startup via `sqlx::migrate!`:

- `20261012000001_create_users_table.sql`: Users table with `BIGSERIAL` primary key and unique email index.
- `20261012000002_create_refresh_tokens_table.sql`: Refresh tokens table with cascade deletion foreign key and indexes.

---

## API Documentation

Interactive API documentation is served directly by the application:

- **Scalar UI**: [http://127.0.0.1:3000/docs](http://127.0.0.1:3000/docs)
- **OpenAPI JSON**: [http://127.0.0.1:3000/api-docs/openapi.json](http://127.0.0.1:3000/api-docs/openapi.json)

---

## API Endpoints

### Authentication (`/Auth`)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|:-------------:|
| `POST` | `/login` | Authenticate user with email and password | No |
| `POST` | `/refresh` | Issue a new access/refresh token pair | No |
| `POST` | `/logout` | Invalidate refresh token and revoke session | No |
| `GET` | `/profile` | Fetch authenticated user's profile details | Bearer Token |

### User Management (`/Users`)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|:-------------:|
| `POST` | `/users` | Create a new user | No |
| `GET` | `/users` | List all registered users | No |
| `GET` | `/users/{id}` | Get user details by ID | No |
| `PUT` | `/users/{id}` | Update an existing user by ID | No |
| `DELETE` | `/users/{id}` | Delete a user by ID | No |

---

## Testing & Quality Checks

Run the entire test suite (including E2E and documentation tests):

```bash
cargo test
```

Check code formatting and run the Clippy linter:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

---

## Project Structure

```text
├── .docker/               # Dockerfile configurations
├── .github/workflows/     # GitHub Actions CI workflow
├── src/
│   ├── database/          # PostgreSQL pool connection & schema initialization
│   │   └── migrations/    # Versioned SQL migration files
│   ├── docs/              # OpenAPI specs, modular schemas & security addon
│   ├── errors/            # Centralized error handling and responses
│   ├── handlers/          # HTTP request handlers (auth & users)
│   ├── middlewares/       # Bearer token extractor & auth guards
│   ├── models/            # Data models and DTOs with schema derives
│   ├── repositories/      # SQLx database query abstraction layer
│   ├── routes/            # Route declarations and routing tree
│   ├── services/          # Business logic, password hashing & JWT management
│   ├── lib.rs             # Axum app factory & Scalar docs mounting
│   └── main.rs            # Application entry point
├── tests/                 # E2E integration and OpenAPI documentation tests
├── Cargo.toml             # Project manifest and dependencies
└── README.md              # Project documentation
```
