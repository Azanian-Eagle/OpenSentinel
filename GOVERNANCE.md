# Governance Model

OpenSentinel is an open-source project managed by the Azanian Eagle community.

## Roles

- **Maintainers**: Have write access to the repository and can merge pull requests.
- **Contributors**: Submit pull requests and issues.

## Decision Making

Decisions are made by consensus among maintainers. Major architectural changes require a Request for Comment (RFC) process.

## Branch Protection Rules

To ensure the integrity of the codebase, the `main` branch is protected by the following rules:

1.  **Code Review & Ownership**:
    *   All Pull Requests require approval from at least one Code Owner as defined in `.github/CODEOWNERS`.
    *   Sneaky contributions are prevented by ensuring no one can merge without independent review.
2.  **Require Signed Commits**: All commits must be GPG signed to verify the identity of the author.
3.  **Linear History**: Merge commits are not allowed. All PRs must be rebased or squashed.
4.  **Strict Status Checks**:
    *   CI/CD pipeline must pass (tests, linting, security audit).
    *   `cargo test` must pass.
    *   `cargo audit` must pass (no known vulnerabilities).
5.  **No Direct Pushes**: Direct pushes to `main` are disabled. Administrators must also follow these rules.

## Security

We prioritize security. See [SECURITY.md](SECURITY.md) for our security policy and reporting guidelines.
