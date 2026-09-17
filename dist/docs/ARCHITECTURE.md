# System Architecture (1.0 Production Release)

OpenSentinel is meticulously engineered from the ground up to deliver uncompromising performance, absolute privacy, and seamless ease of use for developers. This document outlines the core structural design that enables our behavioural analysis engine.

## The Technology Stack

### The Backend Engine
- **Language**: Rust
- **Framework**: Actix-web
- **Justification**: Rust was specifically chosen for its guaranteed memory safety, fearless concurrency model, and exceptionally low latency footprint. This makes it ideal for handling high volumes of traffic without memory overhead, far surpassing Node.js or Python runtimes.

### The Frontend Sensor (Client)
- **Language**: Vanilla JavaScript (ES6+ UMD / Universal Module Definition)
- **Size Profile**: < 20kb (gzipped)
- **Mechanism**: The lightweight `sensor.js` silently captures critical behavioural telemetry (including cursor linearity, velocity variance, and keystroke flight dynamics).
- **Security**: Sensor payloads are dynamically encrypted using AES-256-GCM client-side encryption and signed with SHA-256 Proof-of-Work nonces before transmission.
- **Reliability**: Supports fallback endpoint arrays and offline queuing with exponential backoff retries.

### The AI & Machine Learning Pipeline
- **Type**: Random Forest Classifier
- **Format**: ONNX (Open Neural Network Exchange) via `model.onnx`
- **Inference Engine**: Driven by the highly optimised `ort` crate operating directly within the Rust backend.
- **Implementation Strategy**: By performing all ML inference locally on the sovereign server, sensitive behavioural data never leaves your infrastructure, guaranteeing strict compliance with POPIA, GDPR, and CCPA frameworks.

### First-Party SDKs & Client Libraries
- **React SDK (`@azanian-eagle/react`)**: TypeScript wrapper providing the `useOpenSentinel` hook with automated dependency handling and SSR safety checks.
- **Python SDK (`opensentinel`)**: Lightweight client wrapper relying strictly on Python standard libraries (`urllib`).
- **NPM Vanilla JS (`@azanian-eagle/client`)**: UMD client asset usable across bundlers or direct CDN script inclusions.

## Observability, Analytics & Management

- **Authenticated Dashboard (`/api/dashboard`)**: Visual telemetry monitoring request volumes, bot detection ratios, and system health secured via HTTP Basic Authentication using `ADMIN_TOKEN`.
- **Prometheus Exporter (`/metrics`)**: Standardized Prometheus metrics exposing total requests, verification outcomes (`verify_success_total`, `verify_failed_total`), rate limit events, and federation ingests.
- **Container Health Probes**: Runtime checking via `/healthz` and `/readyz` endpoints.

## The Verification Data Flow

1. **Passive Collection**: The JavaScript sensor silently accumulates non-identifying behavioural telemetry during organic user interaction.
2. **Client-Side Encryption & PoW**: The sensor calculates a SHA-256 Proof-of-Work nonce, encrypts payload telemetry using AES-256-GCM, and dispatches the payload to the `/verify` endpoint.
3. **Decryption & Replay Protection**: The Rust backend decrypts the payload, verifies the PoW hash difficulty, and checks timestamp freshness and nonce uniqueness to prevent replay attacks.
4. **Analytical Inference**: Extracted feature vectors (deviation, speed variance, typing intervals) are evaluated against the local ONNX machine-learning model (`ort`).
5. **Verdict & Response**: The server returns a score (0.0 to 1.0) and boolean pass/fail status, logging metrics locally.

## Decentralised Peer-to-Peer Federation (Phase 3)

- **Ed25519 Cryptographic Signatures**: Nodes sign threat intelligence payloads using Ed25519 private keys (`NODE_PRIVATE_KEY`) to ensure signature validation across peers.
- **Dynamic Peer Discovery**: Nodes periodically fetch peer registry lists from `FEDERATION_DISCOVERY_URL` to update trusted peer maps safely in memory.
- **Gossip Protocol**: Discovered threat signatures are asynchronously gossiped across trusted nodes to maintain collective immunity without sharing user identities.

## Infrastructure & Cloud Native Deployment

- **Containerisation**: Standardized Docker runtime built on `debian:bookworm-slim`.
- **Kubernetes & Helm**: Production-ready deployment using raw manifests (`k8s/`) or the official Helm Chart (`k8s/helm/opensentinel`) supporting Horizontal Pod Autoscaling (HPA), Prometheus ServiceMonitors, and HashiCorp/AWS ExternalSecret definitions.
