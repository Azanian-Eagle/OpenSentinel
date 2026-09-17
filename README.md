# OpenSentinel (1.0.0 Stable Production Release)

![Security & CI Audit](https://github.com/Azanian-Eagle/OpenSentinel/actions/workflows/ci-audit.yml/badge.svg)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](http://makeapullrequest.com)
[![Licence: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**An [Azanian Eagle](https://Azanian-Eagle.github.io/) Project**

OpenSentinel is a free, radically open-source, and entirely non-invasive CAPTCHA replacement specifically engineered to revolutionise the cybersecurity market. Built with an absolute commitment to digital sovereignty, it prioritises user privacy, universal accessibility, and uncompromising security without ever degrading the user experience or collecting personally identifiable information.

## Quickstart

Get your secure backend and testing environment operational in seconds.

```bash
# Clone the repository to your local machine
git clone https://github.com/Azanian-Eagle/OpenSentinel.git
cd OpenSentinel

# Start the self-hosted service with Docker Compose
cp .env.example .env
docker compose up --build
```

The server binds to `http://localhost:8080`. Visit that URL to interact with the client interface and confirm service health.

### Production Release (1.0.0)

For production deployments, install prebuilt binaries or pull the official release image from GitHub Container Registry:

#### Install Prebuilt Binaries via Shell Script
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Azanian-Eagle/OpenSentinel/releases/download/1.0.0/opensentinel-server-installer.sh | sh
```

#### Install Prebuilt Binaries via Homebrew
```bash
brew install opensentinel-server
```

#### Pull Tagged Docker Image
```bash
docker pull ghcr.io/azanian-eagle/opensentinel:v1.0.0
```

#### Official Prebuilt Binary Assets
- **ARM64 Linux:** `opensentinel-server-aarch64-unknown-linux-gnu.tar.xz`
- **x64 Linux:** `opensentinel-server-x86_64-unknown-linux-gnu.tar.xz`
- **Apple Silicon macOS:** `opensentinel-server-aarch64-apple-darwin.tar.xz`
- **Intel macOS:** `opensentinel-server-x86_64-apple-darwin.tar.xz`
- **x64 Windows:** `opensentinel-server-x86_64-pc-windows-msvc.zip`

The release package includes the pre-compiled backend binary, the ONNX machine learning model, client assets, and default configurations.
Threat intelligence logs are automatically rotated once they reach the configured `THREAT_INTEL_MAX_BYTES` limit.

## Philosophy & Digital Sovereignty

Traditional CAPTCHA systems force users to solve visual puzzles, effectively conducting unpaid labour to train third-party artificial intelligence models. Legacy solutions rely on invasive tracking cookies, browser fingerprinting, and device profiling that breach fundamental digital privacy.

OpenSentinel fundamentally alters this paradigm: **we categorically do not train any third-party AI models or harvest user profiles.** Our self-hostable, radically transparent infrastructure respects user dignity. By analysing the natural, mathematical variance of human interaction (such as mouse movement curvature and keystroke flight dynamics) rather than demanding active puzzle solving, we protect web applications while guaranteeing compliance with global privacy regulations (including POPIA, GDPR, and CCPA).

## Core Features & Technical Architecture

- **Non-Invasive Verification Array:** Dynamic client-side telemetry measuring cursor trajectory linearity, velocity variance, and keystroke timing, completely eliminating intrusive image challenges.
- **Local AI ONNX Inference:** Local execution of Random Forest machine learning models (powered by `ort` and `ndarray`), calculating bot probabilities entirely on your sovereign server without sending telemetry to external clouds.
- **Client AES-256-GCM Encryption:** High-resolution sensor payloads are dynamically encrypted on the client side using AES-GCM and SHA-256 Proof-of-Work nonces before transmission.
- **Cryptographic Proof-of-Work (PoW):** Optional client-side PoW computation deters automated script attacks and high-volume DDoS attempts by raising the computational cost for bots.
- **Federated Threat Intelligence Network:** Peer-to-peer threat signature sharing across instances using Ed25519 cryptographic signatures and dynamic peer discovery (`FEDERATION_DISCOVERY_URL`).
- **Live Observability & Analytics:** An authenticated real-time dashboard at `/api/dashboard` paired with Prometheus-compatible metrics at `/metrics`, `/healthz`, and `/readyz`.
- **First-Party SDK Ecosystem:** Ready-to-use SDKs for React (`@azanian-eagle/opensentinel-react`), Python (`opensentinel`), and UMD Vanilla JS (`@azanian-eagle/opensentinel-client`).
- **Cloud-Native Deployment:** First-class support for Kubernetes via raw manifests (`k8s/`) and a production-grade Helm Chart (`k8s/helm/opensentinel`).

## First-Party SDKs & Client Libraries

OpenSentinel offers official packages across ecosystems:

### React SDK (`@azanian-eagle/opensentinel-react`)

```bash
npm install @azanian-eagle/opensentinel-react
```

```tsx
import React from 'react';
import { useOpenSentinel } from '@azanian-eagle/opensentinel-react';

export const VerificationComponent = () => {
  const { isVerified, isEvaluating, score, verify } = useOpenSentinel({
    endpoints: ['https://api.yourdomain.com/verify'],
    enablePoW: true,
  });

  return (
    <div>
      <button onClick={verify} disabled={isEvaluating}>
        {isEvaluating ? 'Evaluating...' : 'Verify Humanity'}
      </button>

      {isVerified && <p>Verified Human (Score: {score})</p>}
    </div>
  );
};
```

### Python SDK (`opensentinel`)

```bash
pip install opensentinel
```

```python
from opensentinel import OpenSentinelClient

client = OpenSentinelClient(endpoint="https://api.yourdomain.com/verify")
# Verify client telemetry backend-side or query status
```

### Vanilla JS / UMD (`@azanian-eagle/opensentinel-client`)

```html
<script src="/src/sensor.js"></script>
<script>
  OpenSentinel.init({
    endpoints: ['https://api.yourdomain.com/verify'],
    enablePoW: true,
    onSuccess: function(token) { console.log('Human verified'); },
    onFailure: function() { console.warn('Bot detected'); }
  });
</script>
```

## Regulatory Compliance & Data Protection

OpenSentinel guarantees complete compliance with global privacy regulations, including South Africa's Protection of Personal Information Act (POPIA), Europe's General Data Protection Regulation (GDPR), and the California Consumer Privacy Act (CCPA).

- **Zero Personally Identifiable Information (PII):** No IP logging, browser fingerprinting, or user account tracking.
- **Strict Data Minimisation:** Evaluates abstract mathematical variance (speed, trajectory, keystroke timing) rather than user identities.
- **Local Processing:** Machine learning models execute locally on your infrastructure. Telemetry never leaves your controlled network boundary.
- **No Cross-Site Tracking:** Eliminates cookies and persistent device beacons across sites.

## Configuration & Environment Variables

| Variable | Description | Default |
|---|---|---|
| `PORT` | HTTP server listening port | `8080` |
| `PAYLOAD_SECRET_KEY` | 64-character hex string (32-byte key) for AES-256-GCM encryption | Required |
| `ADMIN_TOKEN` | Bearer/Basic Auth token for `/api/dashboard` access | Optional |
| `FEDERATION_ENABLED` | Enable P2P threat intelligence sharing | `false` |
| `TRUSTED_PEERS` | Comma-separated peer URLs | `""` |
| `TRUSTED_PEERS_PUBKEYS` | Comma-separated hex Ed25519 public keys | `""` |
| `NODE_PRIVATE_KEY` | Hex Ed25519 private key for signing threat intel payloads | Optional |
| `NODE_URL` | Public URL of this node sent to peers | Optional |
| `FEDERATION_DISCOVERY_URL` | URL to fetch dynamic peer lists | Optional |
| `RATE_LIMIT_MAX_REQUESTS` | Max requests per rate limit window per IP | `60` |
| `RATE_LIMIT_WINDOW_SECONDS` | Rate limit window in seconds | `60` |
| `OPEN_SENTINEL_MODEL_PATH` | Path to `model.onnx` file | `model.onnx` |
| `OPEN_SENTINEL_CLIENT_DIR` | Path to client static asset directory | `../client` |
| `OPEN_SENTINEL_DATA_DIR` | Data storage directory for threat logs | `data` |
| `ALLOWED_ORIGINS` | Comma-separated allowed CORS origins | `http://localhost:8080,http://127.0.0.1:8080` |

## Deployment & Infrastructure Manual

### Option 1: Docker Compose

```bash
cp .env.example .env
# Edit PAYLOAD_SECRET_KEY in .env
docker compose up -d
```

### Option 2: Kubernetes & Helm Chart

Deploy with raw manifests:

```bash
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
```

Or deploy via Helm (from local chart repository or OCI registry):

```bash
# Deploy from local chart repository
helm install opensentinel k8s/helm/opensentinel \
  --set env.payloadSecretKey="0102030405060708090a0b4c0d0e0f101112131415161718191a1b1c1d1e1f20"

# Or deploy directly from GitHub Container Registry (OCI)
helm install opensentinel oci://ghcr.io/azanian-eagle/helm/opensentinel \
  --set env.payloadSecretKey="0102030405060708090a0b4c0d0e0f101112131415161718191a1b1c1d1e1f20"
```

The Helm chart includes production features such as Horizontal Pod Autoscaler (HPA), Prometheus ServiceMonitor, and HashiCorp/AWS ExternalSecrets support.

## Testing & Verification

Run the full Rust test suite:

```bash
cd server
cargo test --workspace
```

Run Python integration tests:

```bash
pip install playwright cryptography
playwright install
python3 tests/integration_tests.py
python3 tests/fail_safes_test.py
```

## Governance & Community

OpenSentinel is an open-source project licensed under the MIT Licence. Contributions, audits, and security reports are actively welcomed.

- **Contributing:** See `CONTRIBUTING.md` for guidelines.
- **Code of Conduct:** See `CODE_OF_CONDUCT.md`.
- **Security Policy:** Report security issues according to `SECURITY.md`.

## Licence

MIT
