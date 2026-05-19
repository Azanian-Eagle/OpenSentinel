# System Architecture

OpenSentinel is meticulously engineered from the ground up to deliver uncompromising performance, absolute privacy, and seamless ease of use for developers. This document outlines the core structural design that enables our behavioural analysis engine.

## The Technology Stack

### The Backend Engine
- **Language**: Rust
- **Framework**: Actix-web
- **Justification**: Rust was specifically chosen for its guaranteed memory safety, fearless concurrency model, and exceptionally low latency footprint. This makes it absolutely ideal for handling the massive volume of high-traffic verification requests required by enterprise-level deployments, far surpassing the capabilities of traditional Node.js or Python implementations.

### The Frontend Sensor (Client)
- **Language**: Vanilla JavaScript (ES6+)
- **Size Profile**: < 20kb (gzipped)
- **Mechanism**: The lightweight `sensor.js` silently captures critical behavioural telemetry (including mouse linearity, cursor speed variance, and precise keystroke flight dynamics). It then rigorously obfuscates and compresses this payload before transmitting it to the server.
- **Justification**: We prioritise universal accessibility and blazing-fast Time to Interactive (TTI). By consciously avoiding bloated WebAssembly (WASM) modules for the client, we ensure the sensor remains entirely frictionless.

### The AI & Machine Learning Pipeline
- **Type**: Advanced Anomaly Detection (utilising scikit-learn generated Isolation Forests or Autoencoders)
- **Format**: ONNX (Open Neural Network Exchange)
- **Inference Engine**: Driven by the highly optimised `ort` crate operating directly within the Rust backend.
- **Implementation Strategy**: By performing all complex inference strictly on the local server via ONNX definitions (`model.onnx`), we completely eliminate the need to send any sensitive behavioural data to opaque third-party cloud providers, thereby guaranteeing strict adherence to POPIA and GDPR frameworks.

## The Verification Data Flow

1. **Passive Collection**: The JavaScript client continuously, yet silently, accumulates non-identifying behavioural telemetry as the user organically interacts with the webpage interface.
2. **Secure Transmission**: The raw data is intelligently sanitised, securely encoded (e.g., via Base64 obfuscation combined with client-side Proof-of-Work to deter trivial script abuse), and dispatched asynchronously to the designated `/verify` endpoint.
3. **Analytical Inference**: The Rust server aggressively unpacks the payload, mathematically extracting vital features (such as trajectory deviations and typing intervals). It then rigorously evaluates these metrics against the local ONNX machine-learning model to reliably differentiate between organic human imperfection and programmatic bot rigidity.
4. **Final Verdict**: The server securely returns an encapsulated "risk score" (ranging from 0.0 to 1.0) alongside a definitive boolean pass/fail verification token to the client.

## Network & Infrastructure

- **Containerisation**: Fully supported via Docker for rapid, reproducible deployments.
- **Deployment Flexibility**: Completely agnostic to runtime environments; easily deployable across Kubernetes clusters, Google Cloud Run, or bare-metal servers.
- **Decentralised Federation**: Built-in support for a highly resilient Peer-to-Peer (P2P) threat intelligence network, allowing disparate OpenSentinel instances to securely exchange anonymised attack signatures without ever compromising local user identities.
