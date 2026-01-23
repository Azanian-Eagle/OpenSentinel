# OpenSentinel

**An Azanian Eagle Project**

OpenSentinel is a free, open-source, non-invasive AI CAPTCHA replacement designed to revolutionise the current outdated market. It prioritizes privacy, accessibility, and security without compromising user experience.

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
- **Zero-Click**: No user interaction required. The verification happens in the background.
- **Privacy-First**: GDPR and POPIA (South Africa) compliant. No invasive tracking or persistent fingerprinting.
- **Lightweight**: Client component is <20kb and accessible to all developers via a simple API.
- **High Performance**: Rust-based backend for speed and memory safety.

## Why OpenSentinel?

Unlike reCAPTCHA or hCaptcha, OpenSentinel does not train third-party AI models on your users' time. It is a self-hostable, transparent solution that respects user dignity and privacy.

## Installation

### Prerequisites

- Rust (latest stable)

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

This repository includes a GitHub Action to automatically re-apply protections if the repository visibility changes (e.g., made public).

1. Create a Personal Access Token (PAT) with `repo` scope.
2. Add it to **Settings > Secrets and variables > Actions** as a Repository Secret named `ADMIN_TOKEN`.

This enforces:
- Required status checks (test, security-audit, lint)
- Code owner reviews
- Signed commits
- Linear history

## License

MIT
