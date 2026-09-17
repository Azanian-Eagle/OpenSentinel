# @azanian-eagle/opensentinel-client

Official Vanilla JavaScript / UMD client-side sensor library for **OpenSentinel**, engineered by **Azanian Eagle** to deliver non-invasive, privacy-preserving CAPTCHA-free human verification and bot detection.

## About OpenSentinel & Azanian Eagle

Developed by [Azanian Eagle](https://github.com/Azanian-Eagle), OpenSentinel is built on the core principle of **digital sovereignty** and **user privacy**. Traditional CAPTCHA systems force users to solve image puzzles, harvesting user data and using human labor to train commercial machine learning models.

OpenSentinel replaces invasive visual puzzles with frictionless telemetry analysis. By observing abstract human interaction metrics—such as mouse trajectory curvature, acceleration variance, and keystroke flight dynamics—OpenSentinel detects automated software bots without collecting personally identifiable information (PII).

### Key Highlights
- **Zero Visual Friction:** Users interact naturally with your web application without solving puzzles.
- **Privacy-First Architecture:** No tracking cookies, no fingerprinting, and zero PII collected. Fully compliant with South Africa's **Protection of Personal Information Act (POPIA)**, GDPR, and CCPA.
- **Client AES-256-GCM Encryption:** High-resolution sensor telemetry is encrypted on the client side using AES-GCM before transmission over the network.
- **Cryptographic Proof-of-Work (PoW):** Optional SHA-256 client-side PoW deters credential stuffing and high-volume DDoS automated attacks.
- **High Availability & Fail-Over:** Built-in endpoint fallback array and offline queuing mechanism with exponential backoff retries.

---

## Installation

### Via NPM

```bash
npm install @azanian-eagle/opensentinel-client
```

### Via Direct Script Tag (UMD / CDN)

```html
<script src="/path/to/client/src/sensor.js"></script>
```

---

## Complete Implementation Guide

### 1. Basic Initialization & Verification

Import or include `OpenSentinel`, initialize the sensor, and trigger verification upon user action (e.g., form submission or login):

```javascript
import OpenSentinel from '@azanian-eagle/opensentinel-client';

// Initialize OpenSentinel with server endpoint and configurations
OpenSentinel.init({
  endpoints: ['https://api.yourdomain.com/verify'],
  enablePoW: true,
  onSuccess: function(token) {
    console.log('Human verification successful:', token);
  },
  onFailure: function(error) {
    console.warn('Human verification failed:', error);
  }
});

// Trigger verification during form submission
async function handleSubmit(event) {
  event.preventDefault();

  const result = await OpenSentinel.verify();
  if (result.passed) {
    console.log('User verified with score:', result.score);
    // Proceed with form submission
  } else {
    alert('Security check failed: ' + result.message);
  }
}
```

### 2. Configuration Options Reference

The `OpenSentinel.init(options)` method accepts the following configuration parameters:

| Option | Type | Default | Description |
|---|---|---|---|
| `endpoint` | `string` | `'/verify'` | Single backend server endpoint URL. |
| `endpoints` | `Array<string>` | `['/verify']` | Array of fall-back backend endpoints for high availability. |
| `enablePoW` | `boolean` | `false` | Enables client-side SHA-256 Proof-of-Work challenge to deter automated bot fleets. |
| `maxEvents` | `number` | `50` | Maximum number of telemetry interaction events stored in memory before transmission. |
| `onSuccess` | `function` | `null` | Callback function executed when humanity verification passes. Receives verification token or status message. |
| `onFailure` | `function` | `null` | Callback function executed when verification fails or a bot is detected. |

### 3. Handling Network Instability & Custom Events

`OpenSentinel` automatically handles intermittent network failures by retrying failed payloads across all configured endpoints using an exponential backoff schedule.

During network retries, the sensor dispatches custom window events that you can listen for to update user interface feedback:

```javascript
window.addEventListener('opensentinel-network-unstable', function(e) {
  console.warn('Network connection unstable. Retrying attempt ' + e.detail.retryCount + ' in ' + e.detail.backoffTime + 'ms');
  // Custom UI alert or notification
});
```

---

## Technical Specifications & Cryptography

### Sensor Telemetry
- **Mouse Dynamics:** Measures cursor coordinate streams (`clientX`, `clientY`, `timestamp`) to evaluate trajectory curvature, velocity jitter, and acceleration profile.
- **Keystroke Dynamics:** Captures key flight timing profiles without logging actual text input values, preserving user privacy while distinguishing human cadence from automated scripts.

### Payload Security
1. **Client AES-256-GCM Encryption:** Sensor telemetry is encrypted into ciphertext with a 12-byte initialization vector (`iv`) before leaving the client browser.
2. **Proof-of-Work Nonce:** When `enablePoW: true` is configured, a SHA-256 nonce is calculated matching required difficulty before transmission.

---

## Compliance & Digital Sovereignty

OpenSentinel is engineered by **Azanian Eagle** to safeguard user dignity and digital sovereignty:
- **POPIA (South Africa):** Complies with lawful processing principles by omitting PII and processing abstract behavioral variance locally.
- **GDPR & CCPA:** Fully compliant due to zero cross-site tracking, zero persistent profiling, and no storage of personal telemetry on third-party servers.

---

## Licence

Distributed under the **MIT Licence**. Built with pride by **Azanian Eagle**.
