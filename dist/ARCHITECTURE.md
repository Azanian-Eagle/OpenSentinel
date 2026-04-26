# Architecture

OpenSentinel is designed for high performance, privacy, and ease of use.

## Tech Stack

### Backend
- **Language**: Rust
- **Framework**: Actix-web
- **Reason**: Memory safety, concurrency, and extremely low latency suitable for high-traffic verification.

### Frontend (Client)
- **Language**: Vanilla JavaScript (ES6+)
- **Size**: < 20kb (gzipped)
- **Mechanism**: Captures telemetry (mouse, keyboard, touch, accelerometer) and sends a compressed payload to the server.
- **Reason**: Accessibility and performance. No heavy WASM required for basic telemetry.

### AI Model (Planned)
- **Type**: Anomaly Detection (Isolation Forest or Autoencoder)
- **Format**: ONNX
- **Inference**: `ort` crate in Rust.
- **Current Implementation**: Heuristic-based behavioral analysis (Proof of Concept).

## Data Flow

1.  **Collection**: Client collects behavioral data during user interaction (or passive monitoring).
2.  **Transmission**: Data is sanitized and sent to `/verify` endpoint.
3.  **Inference**: Server extracts features and evaluates them (currently via heuristics, planned via ONNX model).
4.  **Verdict**: Server returns a "risk score" (0.0 - 1.0) and a pass/fail boolean.

## Infrastructure
- **Containerization**: Docker
- **Deployment**: Any container runtime (e.g., Kubernetes, Cloud Run).
- **Database**: Redis (optional, for rate limiting/replay protection).
