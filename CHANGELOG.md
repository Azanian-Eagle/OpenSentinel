# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-01-23

### Added
- **Cryptographic HPKE Challenge Mechanism**: Added `GET /challenge` endpoint providing short-lived ephemeral keypairs and challenge nonces for encrypted client telemetry.
- **Ed25519 Signed Verification Tokens & JWKS**: Every successful verification yields an Ed25519-signed JWT token. Introduced `GET /.well-known/jwks.json` to allow offline verification by relying parties.
- **IETF Privacy Pass Protocol Integration**: Evaluated requests include standard `WWW-Authenticate: PrivateToken` and `Private-Token` headers for Privacy Pass redemption.
- **Drop-in Siteverify Compatibility Endpoint**: Added `POST /compat/siteverify` supporting URL-encoded, JSON, and query parameter inputs for 1:1 replacement of Cloudflare Turnstile, reCAPTCHA, and hCaptcha backend validation calls.
- **Provenance & Supply Chain Attestations**: Integrated keyless Cosign container image signing on GHCR, Sigstore package provenance, and SLSA attestations across release pipelines.

### Changed
- Updated all SDKs (`@azanian-eagle/opensentinel-client`, `@azanian-eagle/opensentinel-react`, `azanian-eagle-opensentinel`), server binary, Helm charts, and documentation site to SemVer 1.1.0.

## [1.0.4279] - 2026-01-23

### Changed
- Updated release tags, app versions, package manifests, and documentation references to version 1.0.4279.

## [1.0.4] - 2026-01-23

### Changed
- Updated release tags, app versions, package manifests, and documentation references to version 1.0.4.

## [1.0.1] - 2026-01-23

### Changed
- Updated release tags and package versions to 1.0.1 with app version 1.0.

## [1.0.0] - 2026-01-23

### Added
- **Production Release**: Officially transitioning from Beta to 1.0 Stable. Added Kubernetes manifests, load testing, and offline queue capabilities for unreliable internet connections.

## [0.1.0] - 2026-01-23

### Added
- **Core Logic:** Implemented behavioral analysis engine in Rust backend.
    - Mouse movement linearity detection (to flag bot-like straight lines).
    - Mouse speed variance analysis (to flag constant speed).
    - Keystroke dynamics analysis (inter-key timing and variance).
- **Testing:** Added integration test suite (`tests/integration_tests.py`) using Playwright.
    - Simulates bot behavior (linear movement, instant typing).
    - Simulates human behavior (curved movement, variable typing).
- **Documentation:** Added Quickstart guide to README.

### Changed
- Updated server to use enhanced scoring algorithm instead of placeholder heuristics.
