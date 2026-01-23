# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
