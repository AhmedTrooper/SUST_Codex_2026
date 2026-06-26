# sust_codex_2026 — QueueStorm Investigator

Production-grade internal AI/API copilot service designed for digital finance support operations. This service classifies, routes, and investigates support tickets against user transaction histories under strict safety constraints.

## Tech Stack
* **Backend**: Rust (Axum, Tokio, SQLx, Tower-HTTP, Reqwest, Redis)
* **Frontend**: React (Vite, TanStack Start, Bun)

---

## Getting Started

### Prerequisites
* [Rust](https://www.rust-lang.org/) (2024 edition)
* [Bun](https://bun.sh/)
* [PostgreSQL](https://www.postgresql.org/) & [Redis](https://redis.io/) (optional; fallback to in-memory mode is supported)

---

## Backend Setup (Rust)

1. **Configure Environment**:
   Copy `.env.example` to `.env` and set variables:
   ```bash
   cp .env.example .env
   ```

2. **Run Server Locally**:
   ```bash
   make api
   # or manually:
   cd api && cargo run
   ```
   The backend server binds to `0.0.0.0` on port `8080` (or `PORT` environment setting) with full CORS enabled.

3. **Running Tests**:
   ```bash
   cd api && cargo test
   ```

---

## API Endpoints

### 1. GET `/health`
Returns the status of the API service to indicate readiness.
* **Response**: `{"status": "ok"}`

### 2. POST `/analyze-ticket`
Analyzes a support ticket containing a complaint and transaction history, returning structured investigator insights.
* **Request**: JSON payload conforming to the request schema.
* **Response**: Conforming structured JSON response.
* **Caching & Performance**: Automatically caches results in Redis (when configured) to respond in `<5ms` and minimize LLM tokens usage.

### 3. GET `/tickets`
Lists all analyzed tickets stored in the database.
* **Query Parameters**:
  - `limit`: Number of records to return (default: `10`).
  - `offset`: Record offset for pagination (default: `0`).
* **Response**: Paginated JSON response containing metadata and a list of tickets.

---

## Architecture & Optimizations

### Hybrid Rule + LLM Architecture
* **Objective Classifications**: Enums (`case_type`, `severity`, `department`, `evidence_verdict`), `relevant_transaction_id`, and `human_review_required` are computed deterministically by a rule-based parsing engine. This ensures 100% accuracy, strict schema validity, and avoids LLM classification drift.
* **Natural Language Drafting**: If a `OPENROUTER_API_KEY` is present in the environment (falling back to `GEMINI_API_KEY` or `GOOGLE_API_KEY`), the server calls OpenRouter via the `rig` library using the model specified by `OPENROUTER_MODEL` (defaults to `openrouter/free` so it never breaks/costs money) to generate contextual responses.
* **Deterministic Fallback**: If the LLM API is unavailable, times out, or fails, the service falls back to pre-defined structured templates.

### Storage & Cache Design (Production-Grade)
* **Automatic Migrations**: On startup, if a `DATABASE_URL` is provided, the server initializes a connection pool and automatically runs inline migrations to ensure the `analyzed_tickets` table exists.
* **PostgreSQL Persistence**: Every ticket processed by `/analyze-ticket` is stored in the database with idempotency handling (`ON CONFLICT DO UPDATE`).
* **Redis Caching Layer**: Requests are cached for 1 hour using a hash key of the request payload, bypassing LLM generation for identical queries to achieve maximum speed and efficiency.
* **Safe Fallbacks**: If PostgreSQL or Redis are unavailable, the service runs in stateless memory-only mode without crashing, guaranteeing reliability.

---

## Safety Guardrails
Our investigator enforces three strict safety rules, post-processed at the API layer:
1. **No PIN/OTP/Password Requests**: `customer_reply` is scanned for sensitive keywords. If unsafe queries are detected, they are automatically replaced with a secure safety prompt.
2. **No Unauthorized Refund Promises**: Refund promises are sanitised to safe official statements (`"any eligible amount will be returned through official channels"`).
3. **No Unofficial Third-Party Links**: Prompts direct users only to official support channels.

---

## Verification & Testing
Our test suite includes programmatic validation tests:
* **`test_preli_sample_cases`**: Loads the official `SUST_Preli_Sample_Cases.json` pack containing 10 worked cases, executes them through the analysis pipeline, and asserts that they match the expected outputs exactly.
* **`test_db_persistence_and_pagination`**: Validates table initialization, database writes, and limit/offset pagination.
* **`test_redis_caching`**: Validates that Redis properly caches and reads ticket analysis responses.

