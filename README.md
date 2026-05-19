# OpenSentinel

![Security & CI Audit](https://github.com/Azanian-Eagle/OpenSentinel/actions/workflows/ci-audit.yml/badge.svg)

**An [Azanian Eagle](https://Azanian-Eagle.github.io/) Project**

OpenSentinel is a free, open-source, non-invasive AI CAPTCHA replacement designed to revolutionise the current outdated market. It prioritises privacy, accessibility, and security without compromising user experience.

## Quickstart

Get up and running in seconds.

```bash
# Clone the repository
git clone https://github.com/Azanian-Eagle/OpenSentinel.git
cd OpenSentinel

# Run the server (Requires Rust)
cd server
cargo run
```

The server will start at `http://localhost:8080`. You can visit this URL to see the demo client.

## Features

- **Non-Invasive**: Uses client-side behavioural analysis (mouse movements, keystroke dynamics, device telemetry) instead of annoying image puzzles.
- **AI-Powered Bot Detection**: Utilises a fast, locally run ONNX machine-learning model (Phase 2) to distinguish humans from procedural bots using behavioural variance, without sending data to the cloud.
- **Zero-Click**: No user interaction required. The verification happens in the background.
- **Privacy-First**: GDPR and POPIA (South Africa) compliant. No invasive tracking or persistent fingerprinting.
- **Lightweight**: Client component is <20kb and accessible to all developers via a simple API.
- **High Performance**: Rust-based backend for speed and memory safety.
- **Decentralised Intelligence Sharing**: Federated network topology (Phase 3) allows independent nodes to exchange anonymised threat signatures. This provides collective network immunity without centralisation.

## Why OpenSentinel?

Traditional bot mitigation services force users to solve interruptive visual puzzles, turning them into unpaid data labellers for corporate AI training. Furthermore, these systems often rely on invasive cross-site tracking and browser fingerprinting that compromise individual privacy.

OpenSentinel addresses these issues by measuring the physics of user interaction locally. By shifting the paradigm from challenge-response tests to passive behavioural observation, we provide a self-hostable, transparent solution that strictly adheres to the principles of the General Data Protection Regulation (GDPR) and the Protection of Personal Information Act (POPIA). Because our threat detection model runs directly on your server, no sensitive user data is ever exposed to third-party providers.

## Installation

### Prerequisites

- Rust (latest stable)
- ONNX Runtime dependencies (automatically managed by `ort` crate on most systems, but requires build tools)

### Backend

1. Navigate to `server/`
2. Run `cargo run`

### Frontend

1. Include the `client/src/sensor.js` in your HTML.
2. Initialize the sensor.

```html
<script src="sensor.js"></script>
<script>
  OpenSentinel.init({ endpoint: '/verify' });
</script>
```

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

## License

MIT
