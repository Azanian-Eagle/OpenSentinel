# OpenSentinel (Beta 1)

![Security & CI Audit](https://github.com/Azanian-Eagle/OpenSentinel/actions/workflows/ci-audit.yml/badge.svg)

**An [Azanian Eagle](https://Azanian-Eagle.github.io/) Project**

OpenSentinel is a free, radically open-source, and entirely non-invasive CAPTCHA replacement specifically designed to fundamentally revolutionise the outdated, monopolised cybersecurity market. Built with a fierce commitment to digital sovereignty, it strongly prioritises absolute user privacy, universal accessibility, and uncompromising security, without ever degrading the core user experience.

## Quickstart

Get your secure backend and testing environment up and running in a matter of seconds.

```bash
# Clone the repository to your local machine
git clone https://github.com/Azanian-Eagle/OpenSentinel.git
cd OpenSentinel

# Navigate to the server directory and run the Rust backend
cd server
cargo run
```

The server will immediately start binding to `http://localhost:8080`. You can visit this URL to securely interact with the demo client and test the behavioural sensor.

## The Philosophy & Why It Matters

Every single time a user is forced to identify traffic lights or distorted text, they are unknowingly performing unpaid labour to train massive corporate artificial intelligence models. Traditional CAPTCHA systems rely on deeply invasive tracking cookies and browser fingerprinting that compromise digital sovereignty.

OpenSentinel entirely shifts this paradigm. **Our absolute killer feature is that we categorically do not train any third-party AI.** We are building a self-hostable, transparent infrastructure that respects user dignity. By analysing the natural, inherent imperfections of human interaction rather than demanding explicit puzzle-solving, we protect the web while remaining strictly compliant with South Africa's POPIA and Europe's GDPR.

## Core Features

- **Non-Invasive Verification**: Utilises a sophisticated client-side behavioural analysis engine (measuring mouse linearity, speed variance, and keystroke dynamics) instead of relying on frustrating, interruptive image puzzles.
- **AI-Powered Bot Detection**: Deploys highly optimised, locally run ONNX machine-learning models (Phase 2 integration) to reliably distinguish humans from procedural bots using abstract behavioural variance—ensuring no sensitive data ever leaves your server.
- **Zero-Click Operation**: Absolutely no active user interaction is explicitly required. The complex mathematical verification happens silently in the background whilst the user naturally browses your application.
- **Privacy-First Architecture**: Completely GDPR and POPIA (South Africa) compliant by design. We utilise zero invasive tracking mechanics and absolutely no persistent device fingerprinting.
- **Ultra-Lightweight Sensor**: The highly optimised JavaScript client component weighs in at strictly <20kb, remaining accessible to all developers via an incredibly simple, framework-agnostic API.
- **High-Performance Infrastructure**: A blistering Rust-based backend engineered specifically for high concurrent throughput, minimal latency, and absolute memory safety.
- **Federated Threat Intelligence**: (Phase 3) A fully decentralised peer-to-peer network where participating nodes securely share anonymised mathematical threat signatures to ensure collective immunity.

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
