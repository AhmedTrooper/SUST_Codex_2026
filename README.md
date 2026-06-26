# sust_codex_2026 — QueueStorm Investigator

Production-grade internal AI/API copilot service designed for digital finance support operations. This service classifies, routes, and investigates support tickets against user transaction histories under strict safety constraints.

## Tech Stack
* **Backend**: Rust (Axum, Tokio, SQLx, Tower-HTTP)
* **Frontend**: React (Vite, TanStack Start, Bun)

---

## Getting Started

### Prerequisites
* [Rust](https://www.rust-lang.org/) (2024 edition)
* [Bun](https://bun.sh/)

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
