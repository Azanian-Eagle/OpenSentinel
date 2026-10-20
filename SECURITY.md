# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 1.1.x   | &#10003;  |
| 1.0.x   | &#10003;  |
| < 1.0   | &#10007;  |

## Reporting a Vulnerability

We take security seriously. If you discover a vulnerability in OpenSentinel, please follow these steps:

1. **Do NOT open a public issue.**
2. Email the maintainers at `azanian.eagle@gmail.com`.
3. Include a detailed description of the vulnerability, steps to reproduce, and potential impact.

We will acknowledge your report within 72 hours and provide a timeline for a fix.

## Security Practices

- **Single-Use Ephemeral Session Keys:** Per-challenge single-use session keys derived via server-side SHA-256 prevent secret key leakage and client-side payload forging.
- **Lock-Free Atomic Operations:** Atomic counter metrics and timestamped TTL nonce stores prevent thread lock contention and replay attacks.
- **Constant-Time Cryptographic Verification:** Ed25519 signatures, JWKS public key distribution, and constant-time token comparison protect against timing side-channels.
- **SAST**: Static Application Security Testing executed on every commit.
- **Dependency Scanning**: Dependencies monitored for known vulnerabilities using `cargo audit` and GitHub Dependabot.
- **Privacy & Sovereignty**: Adherence to POPIA, GDPR, and CCPA standards with zero PII collection.
