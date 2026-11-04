# Azanian Eagle

> **Radical Digital Sovereignty & Privacy-Preserving Cyber Infrastructure**

Welcome to the official GitHub organization profile for **Azanian Eagle**. We architect and build high-performance, open-source cybersecurity solutions that prioritise user dignity, digital sovereignty, and accessibility without collecting personally identifiable information (PII).

---

## Flagship Project: [OpenSentinel](https://github.com/Azanian-Eagle/OpenSentinel)

**OpenSentinel** is the premier open-source, self-hosted, zero-click CAPTCHA replacement and bot detection engine. By evaluating the natural mathematical variance of human interaction rather than demanding visual puzzles, OpenSentinel eliminates friction while guaranteeing compliance with global data privacy frameworks including **POPIA**, **GDPR**, and **CCPA**.

### Core Architecture

- **Sovereign Rust & Local ONNX Inference Engine:** Built on Actix-Web and local ONNX Runtime (`ort`), performing behavioral classifications in microseconds on your infrastructure without cloud dependencies.
- **Client AES-256-GCM & SHA-256 Proof-of-Work:** Cryptographically hardened sensor telemetry prevents replay and headless automated abuse.
- **Ed25519 Federated Threat Sharing:** Decentralised, cryptographically verified threat-signature exchange between trusted peer nodes.
- **Drop-in Compatibility:** Standard `/compat/siteverify` endpoint allowing seamless migration from Cloudflare Turnstile, Google reCAPTCHA, and hCaptcha.
- **Kubernetes & Cloud-Native Ready:** Production Helm chart (`k8s/helm/opensentinel`) with HPA, Prometheus metrics, and ExternalSecrets integration.

---

## Multi-Registry Ecosystem

OpenSentinel is distributed across major package ecosystems:

| Ecosystem | Package | Target |
|:---|:---|:---|
| **crates.io** | [`opensentinel-server`](https://crates.io/crates/opensentinel-server) | Rust Backend Server & ONNX Engine |
| **npm** | [`@azanian-eagle/opensentinel-client`](https://www.npmjs.com/package/@azanian-eagle/opensentinel-client) | Vanilla JS / Client Sensor |
| **npm** | [`@azanian-eagle/opensentinel-react`](https://www.npmjs.com/package/@azanian-eagle/opensentinel-react) | React Hook SDK (`useOpenSentinel`) |
| **PyPI** | [`azanian-eagle-opensentinel`](https://pypi.org/project/azanian-eagle-opensentinel/) | Python SDK & Middleware (FastAPI / Flask) |
| **GHCR** | `ghcr.io/azanian-eagle/opensentinel` | Multi-Arch OCI Container Images |
| **Artifact Hub / Helm** | `k8s/helm/opensentinel` | Production Kubernetes Helm Chart |

---

## Explore & Connect

- **Documentation & Live Demo:** [https://azanian-eagle.github.io/OpenSentinel/](https://azanian-eagle.github.io/OpenSentinel/)
- **Repository:** [Azanian-Eagle/OpenSentinel](https://github.com/Azanian-Eagle/OpenSentinel)
- **Creator Profile:** [Azanian Eagle on GitHub](https://github.com/Azanian-Eagle)
- **Contact:** `azanian.eagle@gmail.com`
