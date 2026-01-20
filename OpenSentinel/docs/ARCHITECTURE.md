# Architecture

OpenSentinel is designed for high performance, privacy, and ease of use.

## Tech Stack

### Backend
- **Language**: Rust
- **Framework**: Actix-web (or Axum)
- **Reason**: Memory safety, concurrency, and extremely low latency suitable for high-traffic verification.

### Frontend (Client)
- **Language**: TypeScript / Vanilla JavaScript
- **Size**: < 20kb (gzipped)
- **Mechanism**: Captures telemetry (mouse, keyboard, touch, accelerometer) and sends a compressed payload to the server.
- **Reason**: Accessibility and performance. No heavy WASM required for basic telemetry, though WASM is supported for advanced crypto/hashing if needed.

### AI Model
- **Type**: Anomaly Detection (Isolation Forest or Autoencoder)
- **Format**: ONNX
- **Inference**: `ort` crate in Rust.
- **Reason**: ONNX allows interoperability. Lightweight models allow for low-latency inference on CPU.

## Data Flow

1.  **Collection**: Client collects behavioral data during user interaction (or passive monitoring).
2.  **Transmission**: Data is sanitized and sent to `/verify` endpoint.
3.  **Inference**: Server extracts features and feeds them to the ONNX model.
4.  **Verdict**: Server returns a "risk score" (0.0 - 1.0).

## Infrastructure
- **Containerization**: Docker
- **Deployment**: GCP Cloud Run (Serverless)
- **Database**: Redis (optional, for rate limiting/replay protection).
