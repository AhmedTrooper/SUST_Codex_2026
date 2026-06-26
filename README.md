# QueueStorm Investigator — Digital Finance Support Copilot

An enterprise-grade, hybrid rule-engine & LLM-powered internal copilot service designed for digital finance support teams. This platform classifies, routes, investigates, and drafts safe responses for customer tickets against transaction records under strict safety and compliance regulations.

---

## 🏗️ Visual Architecture

```mermaid
graph TD
    A[Customer Ticket / Request] --> B[API Layer: POST /analyze-ticket]
    B --> C{Deterministic Parser}
    
    C -->|Extract Details| D[Language, Channel, User Type]
    C -->|Numeral Normalization| E[Bangla & English Digit Normalizer]
    C -->|Transaction Scrutiny| F[Resolve Relevant Transaction ID]
    
    D & E & F --> G[Cache Check: Redis Hash Key]
    
    G -->|Cache Hit <5ms| H[Return Cached Response]
    G -->|Cache Miss| I[Compile Guarded Preamble & Prompt]
    
    I --> J[AI Agent: OpenRouter / Free Model]
    J --> K[Raw Reply Draft]
    
    K --> L[Compliance & Safety Post-Processor]
    L -->|Regex & Sensitive Keyword Scan| M{Violation Found?}
    M -->|Yes| N[Force Overwrite with Safe Compliance Template]
    M -->|No| O[Keep Generated Draft]
    
    N & O --> P[PostgreSQL Persistent Log Store]
    P --> Q[Save in Redis Cache]
    Q --> R[Return Final Analysis JSON]
    
    R --> S[High-Contrast Black & White Frontend Dashboard]
```

---

## 🚀 Key Features

* **Collapsible Walkthrough/Tutorial**: Built directly into the frontend dashboard for step-by-step guidance.
* **POST `/analyze-ticket`**: High-performance endpoint validating parameters, resolving transaction matchings, and returning compliant JSON.
* **GET `/health`**: Instant, zero-dependency readiness endpoint returning `{"status": "ok"}`.
* **PostgreSQL Persistent Audit logs**: All analyzed tickets are tracked with pagination support.
* **Ultra-Fast Redis Caching**: Identical request payloads bypass LLM invocation to respond in `<5ms`.
* **Zero-Failure Fallbacks**: If Redis or PostgreSQL are unavailable, the system automatically logs warnings and functions in stateless memory-only mode without crashing.

---

## 🛠️ Getting Started

### 📋 Prerequisites
* **Rust**: Rust Toolchain (Edition 2024)
* **Bun**: Modern JS/TS package manager & runtime
* **Docker & Docker Compose** (Optional, for running PostgreSQL/Redis locally)

---

### 📥 1. Repository Access & Setup
> [!IMPORTANT]
> Read-only collaborator access has been granted to the hackathon organizer: **`bipulhf`**.

Clone the repository and set up environment files:
```bash
git clone https://github.com/AhmedTrooper/SUST_Codex_2026.git
cd SUST_Codex_2026
cp .env.example .env
```

---

### ⚙️ 2. Environment Configuration (`.env`)
Configure the variables inside your `.env` file:
* `PORT`: Port backend service binds to (defaults to `8080`).
* `DATABASE_URL`: PostgreSQL connection string (defaults to `postgresql://sust_codex_2026_user:sust_codex_2026_password@localhost:5432/sust_codex_2026_db`).
* `REDIS_URL`: Redis URI (defaults to `redis://localhost:6379`).
* `OPENROUTER_API_KEY`: API Key for LLM drafting. Fallbacks to `GEMINI_API_KEY` or `GOOGLE_API_KEY` are supported.
* `OPENROUTER_MODEL`: Selected model (defaults to `"openrouter/free"` for cost-free manual testing).

---

### 🐳 3. Starting Services with Docker Compose
To run PostgreSQL and Redis locally:
```bash
make docker-up
# or manually:
docker-compose up -d
```

---

### 🦀 4. Running the Backend (Rust)
From the root directory:
```bash
cd api
cargo run
```
* **Bindings**: Binds to `0.0.0.0:8080`.
* **CORS**: Fully exposed to `*` to allow API evaluation.
* **Database migrations**: Tables are automatically checked and initialized on startup.

#### Running Backend Tests:
```bash
cargo test
```
The test suite validates:
1. `test_health_check`: Correct response formats and status code.
2. `test_unprocessable_entity`: Payload validators for missing/empty fields.
3. `test_preli_sample_cases`: Loads the public sample cases (`SUST_Preli_Sample_Cases.json`) and runs them through the analysis pipeline.
4. `test_db_persistence_and_pagination`: Validates log writes and database paginated offset fetches.
5. `test_redis_caching`: Asserts correct caching and cache hits on identical payloads.

---

### ⚛️ 5. Running the Frontend (React & TanStack Start)
From the root directory:
```bash
cd web
bun install
bun run dev
```
* **Client URL**: Access the client dashboard at `http://localhost:3000`.
* **Build command**: To check production builds:
  ```bash
  bun run build
  ```

---

## 🤖 AI & Model Usage

* **Primary Model**: `"openrouter/free"` (or any configured model passed via `OPENROUTER_MODEL` environment variable).
* **Role**: The LLM is used exclusively for natural language drafting of `customer_reply` and providing contextual helper briefs (`agent_summary`, `recommended_next_action`).
* **Hardening**: System prompts contain a strict adversarial protection block. The complaint text is injected into the LLM context wrapped in XML tags, telling the model to treat it strictly as raw customer input and never execute instructions found within the complaint.

---

## 🛡️ Safety Logic & Guardrails

The service implements a multi-stage safety firewall to protect user data and ensure compliance:

1. **No Sensitive Credential Requests**:
   * *Mechanism*: A post-processing regex scanner checks the generated reply for words like `PIN`, `OTP`, `Password`, `পাসওয়ার্ড`, `পিন`, and `ওটিপি`.
   * *Fallback*: If a violation is caught, the draft is instantly overwritten with:
     > *"Please note that customer support will never ask for your PIN, OTP, or password. Keep your account details private."*
2. **No Unauthorized Refund Promises**:
   * *Mechanism*: If the LLM generates a message promising an immediate refund or specific recovery timeframe without authorization, the system overrides it to a standard statement:
     > *"Any eligible amount will be returned through official channels."*
3. **No Unofficial Third-Party Links**:
   * *Mechanism*: Ensures only verified domains or help centers are present in the response.

---

## ⚠️ Limitations & Boundary Conditions

1. **Deterministic Dependency**: Structuring case classifications, transaction matches, and routing rules is done deterministically. If a ticket contains ambiguous or multiple transaction IDs (e.g. both a failed pay and a wrong cash-out), it routes to the primary transaction context matches or escalates to human review.
2. **Offline Mode Limitations**: If the database and cache go offline, persistence and cache acceleration are bypassed. Responses are generated correctly but historical logs won't be saved.
3. **Language Context**: Although Bengali digit normalizations are fully supported (e.g., matching `১২৩৪` with `1234`), highly mixed/transliterated dialect slang may lead the LLM to draft responses in the default language (English) unless the input structure is clear.
4. **Third-Party API Limits**: OpenRouter/Free models are subject to external rate limiting and latencies. If a call fails or exceeds 30 seconds, the backend falls back to standard templates.

---

## 🔒 Security Compliance
* **No Real Secrets**: All passwords and keys are configured via standard `.env` variables.
* **No Customer Data**: The codebase has zero hardcoded customer profiles, bank pins, or live financial transactions.

---
*Developed for the SUST Codex 2026 Digital Finance Hackathon.*
