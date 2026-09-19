# OpenSentinel Server (`opensentinel-server`)

[![Crates.io](https://img.shields.io/crates/v/opensentinel-server.svg)](https://crates.io/crates/opensentinel-server)

High-performance Rust backend server and ONNX machine learning inference engine for **OpenSentinel**, engineered by **Azanian Eagle** to power non-invasive, privacy-first CAPTCHA replacement services.

## About OpenSentinel & Azanian Eagle

Developed by [Azanian Eagle](https://github.com/Azanian-Eagle), OpenSentinel is built on the core principle of **digital sovereignty** and **user privacy**. Traditional CAPTCHA systems force users to solve visual challenges, collecting invasive tracking profiles and training third-party AI models.

OpenSentinel fundamentally shifts this model: **we categorically do not collect user profiles or train external AI models.** Operating directly on your sovereign infrastructure, the OpenSentinel backend server evaluates abstract mathematical variance in client telemetry—such as cursor curvature, acceleration profiles, and keystroke flight dynamics—to accurately identify automated bots while respecting user dignity.

### Key Features
- **Actix-Web Microsecond Latency:** Powered by the asynchronous, high-throughput Actix-Web framework in Rust.
- **In-Memory Local ONNX Machine Learning:** Local execution of Random Forest machine learning models (via `ort` and `ndarray`), evaluating bot probability completely within your network boundary.
- **AES-256-GCM Cryptographic Decryption:** Automatically decrypts AES-GCM sensor payloads transmitted by OpenSentinel client SDKs.
- **Client SHA-256 Proof-of-Work (PoW) Validation:** Validates client-side nonces to protect against high-volume automated DDoS attempts.
- **Federated Threat Intelligence Network:** P2P signature exchange using Ed25519 cryptographic signatures and dynamic peer discovery (`FEDERATION_DISCOVERY_URL`).
- **Live Observability & Analytics:** Real-time metrics at `/metrics` (Prometheus-compatible), system health endpoints (`/healthz`, `/readyz`), and authenticated dashboard (`/api/dashboard`).

---

## Installation & Building from Source

### Prerequisites
- **Rust Toolchain:** Stable Rust 1.75+ (`rustup`)
- **C++ Build Environment:** `gcc` / `g++` or `clang` for ONNX Runtime C++ linkage.

### Building
Clone the repository and build the binary in release mode:

```bash
git clone https://github.com/Azanian-Eagle/OpenSentinel.git
cd OpenSentinel
cargo build --release --manifest-path server/Cargo.toml
```

The compiled binary will be located at `target/release/opensentinel-server`.

### Running the Server

```bash
# Set required encryption secret key (64 hex characters)
export PAYLOAD_SECRET_KEY="0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"

# Start the backend server
./target/release/opensentinel-server
```

---

## Configuration Reference

The server is configured via environment variables:

| Environment Variable | Description | Default Value | Required |
|---|---|---|---|
| `PORT` | HTTP network listening port | `8080` | No |
| `PAYLOAD_SECRET_KEY` | 64-character hex string (32 bytes) for AES-256-GCM payload decryption | None | **Yes** |
| `ADMIN_TOKEN` | Bearer/Basic Auth token securing `/api/dashboard` | None | No |
| `OPEN_SENTINEL_MODEL_PATH` | File path to local `model.onnx` binary | `model.onnx` | No |
| `OPEN_SENTINEL_CLIENT_DIR` | Directory path containing client static web assets | `../client` | No |
| `OPEN_SENTINEL_DATA_DIR` | Directory path for persistent threat intelligence logs | `data` | No |
| `RATE_LIMIT_MAX_REQUESTS` | Maximum requests allowed per IP in rate limit window | `60` | No |
| `RATE_LIMIT_WINDOW_SECONDS` | Rate limit window duration in seconds | `60` | No |
| `ALLOWED_ORIGINS` | Comma-separated list of permitted CORS origins | `http://localhost:8080,http://127.0.0.1:8080` | No |
| `FEDERATION_ENABLED` | Set to `true` to enable peer-to-peer threat signature sharing | `false` | No |
| `TRUSTED_PEERS` | Comma-separated list of peer server base URLs | `""` | No |
| `TRUSTED_PEERS_PUBKEYS` | Comma-separated list of hex Ed25519 public keys for peer verification | `""` | No |
| `NODE_PRIVATE_KEY` | Hex Ed25519 private key for signing threat intelligence messages | None | No |
| `NODE_URL` | Self-identifying public URL sent to federated peers | None | No |
| `FEDERATION_DISCOVERY_URL` | URL endpoint for dynamic peer discovery updates | None | No |

---

## API Endpoints Reference

### 1. `POST /verify`
Main verification endpoint. Accepts AES-GCM encrypted payload from client sensor.

- **Request Body:**
  ```json
  {
    "data": "<base64_ciphertext>",
    "iv": "<base64_initialization_vector>"
  }
  ```
- **Response Body (200 OK):**
  ```json
  {
    "passed": true,
    "score": 0.98,
    "message": "Human verification passed"
  }
  ```

### 2. `GET /metrics`
Returns Prometheus-formatted application metrics (including `verify_success_total` and `verify_failed_total`).

### 3. `GET /healthz` & `GET /readyz`
Kubernetes liveness and readiness probe endpoints.

### 4. `GET /api/dashboard`
Authenticated real-time analytics dashboard (requires HTTP Basic Auth matching `ADMIN_TOKEN`).

---

## Compliance & Privacy

Engineered by **Azanian Eagle** to comply with global data protection laws:
- **POPIA (South Africa):** Complies with lawful processing by avoiding PII collection and running ML models locally.
- **GDPR & CCPA:** Zero cross-site tracking, zero persistent profiling, and complete data sovereignty.

---

## Licence

Distributed under the **MIT Licence**. Engineered with pride by **Azanian Eagle**.
