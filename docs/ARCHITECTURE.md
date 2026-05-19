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

### AI Model Integration
- **Type**: Random Forest Classifier for behavioural variance analysis.
- **Format**: ONNX (Open Neural Network Exchange).
- **Inference**: Evaluated locally using the highly performant `ort` and `ndarray` crates in Rust.
- **Privacy Assurance**: The model is executed directly on the host server. No telemetry data is sent to external cloud AI providers for processing or training, fulfilling the project's strict privacy requirements.

### Federated Network (Phase 3)
- **Mechanism**: A decentralised HTTP Gossip Protocol.
- **Functionality**: Independent OpenSentinel nodes asynchronously share anonymised threat signatures (e.g., abstract mathematical representations of novel procedural bot movement). This establishes a collaborative defence mechanism without establishing a centralised point of failure or data monopoly.

## Data Flow

1.  **Collection**: The ultra-lightweight JavaScript client observes the user's interaction physics, capturing metrics such as mouse linearity, cursor speed variance, and keystroke flight times.
2.  **Transmission**: The client sanitises the telemetry and transmits it securely to the local `/verify` endpoint. The system includes built-in failover capabilities to ensure high availability.
3.  **Inference**: The Actix-web server extracts the relevant features and passes them to the local ONNX model. The model calculates the probability that the interaction originated from an automated script.
4.  **Verdict**: The server responds with a risk score (between 0.0 and 1.0) and a boolean indicating whether the session passed the human verification threshold.

## Infrastructure
- **Containerization**: Docker
- **Deployment**: Any container runtime (e.g., Kubernetes, Cloud Run).
- **Database**: Redis (optional, for rate limiting/replay protection).
