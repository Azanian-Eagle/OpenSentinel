# OpenSentinel

**An Azanian Eagle Project**

OpenSentinel is a free, open-source, non-invasive AI CAPTCHA replacement designed to revolutionise the current outdated market. It prioritizes privacy, accessibility, and security without compromising user experience.

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
- Node.js (optional, for client builds if we add a bundler)

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

## License

MIT
