# OpenSentinel (1.0.0 Stable Production Release)

![Security & CI Audit](https://github.com/Azanian-Eagle/OpenSentinel/actions/workflows/ci-audit.yml/badge.svg)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](http://makeapullrequest.com)
[![Licence: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**An [Azanian Eagle](https://Azanian-Eagle.github.io/) Project**

OpenSentinel is a free, radically open-source, and entirely non-invasive CAPTCHA replacement specifically designed to fundamentally revolutionise the outdated, monopolised cybersecurity market. Built with a fierce commitment to digital sovereignty, it strongly prioritises absolute user privacy, universal accessibility, and uncompromising security, without ever degrading the core user experience.

## Quickstart

Get your secure backend and testing environment up and running in a matter of seconds.

```bash
# Clone the repository to your local machine
git clone https://github.com/Azanian-Eagle/OpenSentinel.git
cd OpenSentinel

# Start the self-hosted service with Docker Compose
cp .env.example .env
docker compose up --build
```

The server will bind to `http://localhost:8080`. Visit that URL to use the client and verify the service is healthy.

### Production Release

For a reusable deployment, use the published backend image from GitHub Container Registry or the release tarball attached to tagged releases.

```bash
docker pull ghcr.io/azanian-eagle/opensentinel:latest
```

The release tarball includes the backend binary, the ONNX model, `.env.example`, and `docker-compose.yml` for self-hosted setup.
It also includes the client assets so the binary can serve the web UI outside the source tree.
For the tarball, the packaged env sample points `OPEN_SENTINEL_MODEL_PATH` to `model.onnx` and `OPEN_SENTINEL_CLIENT_DIR` to `client`.
The Docker Compose path also persists `data/` so threat-intel logs survive container restarts.
Threat intel logs rotate once they reach the configured `THREAT_INTEL_MAX_BYTES` limit.

## The Philosophy & Why It Matters

Every single time a user is forced to identify traffic lights or distorted text, they are unknowingly performing unpaid labour to train massive corporate artificial intelligence models. Traditional CAPTCHA systems rely on deeply invasive tracking cookies and browser fingerprinting that fundamentally compromise digital sovereignty.

OpenSentinel entirely shifts this paradigm. **Our absolute killer feature is that we categorically do not train any third-party AI.** We are building a self-hostable, radically transparent infrastructure that respects user dignity. By meticulously analysing the natural, inherent imperfections of human interaction rather than demanding explicit puzzle-solving, we protect the open web while remaining strictly compliant with stringent global data protection laws.

## Core Features & Technical Architecture

- **Non-Invasive Verification Array**: Utilises a sophisticated client-side behavioural analysis engine that dynamically measures mouse movement linearity, cursor speed variance, and keystroke flight dynamics, entirely replacing frustrating image puzzles.
- **AI-Powered Local Inference**: Deploys highly optimised, locally run ONNX machine-learning models (Phase 2 integration, powered by `ort` and `ndarray`) to reliably distinguish humans from procedural bots using abstract behavioural variance—ensuring absolutely no sensitive telemetry ever leaves your sovereign server.
- **Zero-Click Operation**: Absolutely no active user interaction is explicitly required. The complex mathematical verification executes silently in the background whilst the user organically browses your application.
- **Cryptographic Proof-of-Work**: To aggressively deter script kiddies and massive automated DDoS attempts, the client optionally computes a mathematical Proof-of-Work (PoW) challenge prior to payload transmission.
- **Ultra-Lightweight Sensor**: The highly optimised JavaScript client component weighs in at strictly <20kb and dynamically encrypts payloads via robust AES-GCM encryption, remaining securely accessible to all developers via an incredibly simple, framework-agnostic API.
- **High-Performance Infrastructure**: A blistering Rust and Actix-web based backend meticulously engineered specifically for high concurrent throughput, minimal latency, and absolute memory safety.
- **Federated Threat Intelligence**: (Phase 3) A fully decentralised peer-to-peer network where participating nodes securely share anonymised mathematical threat signatures asynchronously to ensure collective immunity without relying on corporate CDNs.

## Regulatory Compliance & Data Protection

OpenSentinel is systematically designed from the ground up to guarantee absolute compliance with the most stringent global data protection laws, including South Africa's Protection of Personal Information Act (POPIA), Europe's General Data Protection Regulation (GDPR), and the California Consumer Privacy Act (CCPA).

- **Zero Personally Identifiable Information (PII):** We categorically never collect IP addresses or browser fingerprints.
- **Strict Data Minimisation:** We only measure abstract mathematical variance (e.g., speed, linearity), completely avoiding personal profiling.
- **No Cross-Site Tracking:** We fundamentally reject tracking cookies and shared identifiers.

## Implementation Manual (Production Deployment)

OpenSentinel is ready for production. Follow these steps to implement the stable release.

### Prerequisites

- Rust (latest stable)
- ONNX Runtime dependencies (automatically managed by the `ort` crate on most systems, but requires standard build tools)

### Step 1: Deploy the Sovereign Backend

Create a `.env` file from the provided template and set the required secret key.

Run the service with Docker Compose for local environments or lightweight deployments:

```bash
docker compose up --build -d
```

For enterprise and high-availability production environments, we officially support Kubernetes out-of-the-box. Apply the provided manifests for zero-downtime rolling updates:

```bash
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
```

Optional federation settings belong in the same `.env` file:

```env
FEDERATION_ENABLED=true
TRUSTED_PEERS=https://node1.example.com
# Hex-encoded Ed25519 public keys of trusted peers for signature validation
TRUSTED_PEERS_PUBKEYS=80b91e92c2193b2bb08a1cbcc7e9d77f864e7dbde406d289dc6c8736e149f12d
# Your own Node's hex-encoded Ed25519 private key for signing outgoing threat intel
NODE_PRIVATE_KEY=your_private_key_hex
# The public URL of this node, sent to peers as the source of threat intel
NODE_URL=https://node2.example.com
# 64-character hex string representing the 32-byte AES-GCM payload encryption key
PAYLOAD_SECRET_KEY=0102030405060708090a0b4c0d0e0f101112131415161718191a1b1c1d1e1f20
# Optional: override the ONNX model location when running outside the repo root
OPEN_SENTINEL_MODEL_PATH=/app/server/model.onnx
```

If you want to run the Rust backend directly without Docker, export `PAYLOAD_SECRET_KEY` first and then run `cargo run --release` from the `server/` directory.

### Step 2: Integrate the Frontend Sensor

1. Include the ultra-lightweight `client/src/sensor.js` directly into your web application HTML.
2. Initialise the sensor with advanced network awareness.

```html
<script src="sensor.js"></script>
<script>
  OpenSentinel.init({
      endpoints: [
          'https://api.yourdomain.com/verify',
          'https://node1.example.com/verify', // Public Fallback Node
          'https://node2.example.com/verify'
      ],
      enablePoW: true,
      onSuccess: function(token) { console.log('Human verified successfully.'); },
      onFailure: function() { console.warn('Automated bot behaviour detected.'); }
  });
</script>
```

## Open-Source Community & Governance

Much like the foundational pillars of the open internet (such as Linux or Mozilla), OpenSentinel thrives on radical transparency and community collaboration.

- **Contributing:** We actively encourage developers, security researchers, and data scientists to thoroughly audit our codebase and submit intricate pull requests.
- **Governance:** All major architectural decisions are openly documented and heavily debated within public GitHub issues to ensure the project remains truly community-driven.

## Validation & Testing

OpenSentinel includes an integration test suite to verify its ability to distinguish between bots and humans.

To run the tests:

```bash
# Install dependencies
pip install playwright
playwright install

# Run the integration tests
python3 tests/integration_tests.py
```

## Repository Setup

To ensure strict branch protection and compliance with governance standards, this repository uses a script to configure GitHub protection rules.

### Manual Setup

1. Ensure you have a `GITHUB_TOKEN` with `repo` scope.
2. Run the configuration script:

```bash
export GITHUB_TOKEN=your_token_here
./scripts/protect_branch.sh
```

### Automated Enforcement

This repository includes a GitHub Action to automatically re-apply protections if the repository visibility changes.

1. Create a Personal Access Token (PAT) with `repo` scope.
2. Add it to **Settings > Secrets and variables > Actions** as a Repository Secret named `ADMIN_TOKEN`.

This enforces:

- Required status checks (test, security-audit, lint)
- Code owner reviews
- Signed commits
- Linear history

## Licence

MIT
