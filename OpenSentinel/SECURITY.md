# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a vulnerability in OpenSentinel, please follow these steps:

1.  **Do NOT open a public issue.**
2.  Email the maintainers at `security@azn-eagle.example.com` (replace with actual email).
3.  Include a detailed description of the vulnerability, steps to reproduce, and potential impact.

We will acknowledge your report within 48 hours and provide a timeline for a fix.

## Security Practices

- **SAST**: We run Static Application Security Testing on every commit.
- **Dependency Scanning**: We monitor dependencies for known vulnerabilities using `cargo audit` and GitHub Dependabot.
- **Privacy**: We adhere to GDPR and POPIA standards. No PII is collected without explicit consent (and OpenSentinel is designed to be PII-free).
