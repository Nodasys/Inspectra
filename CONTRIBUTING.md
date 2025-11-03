# Contributing to Inspectra

Thank you for contributing to Inspectra. This document explains how to contribute, code style, testing expectations and the release workflow.

## Who may contribute

- Nodasys internal teams
- Approved external contributors after NDA and contributor agreement

## Contributor agreement

All contributions are expected to be assigned to Nodasys. Contributors may be asked to sign a Contributor License Agreement (CLA) or transfer rights via a simple agreement.

## Branching model

- `main` or `stable` : production-ready releases
- `develop` : integration branch for next release
- feature branches: `feat/<short-desc>`
- fix branches: `fix/<short-desc>`

## Commit message format
Use conventional commit style with a prefix in square brackets. Example:

```
[feat] Add pointer scanner with multi-threading

Detailed description...
Issue: #123
```

## Code style and static analysis

### Core (C++ / Rust)
- Follow idiomatic language guidelines (use clang-format / rustfmt).
- Use modern constructs and avoid deprecated APIs.
- Keep functions small and single-responsibility.

### Bindings (Python / Lua)
- Python: PEP8, type hints, black, ruff.
- Lua: consistent style; document public functions.

### UI (Qt / Web)
- Keep UI code decoupled from business logic.
- Use linting (eslint, prettier) for web UI.

## Tests

- Unit tests must be included for all public interfaces.
- Integration tests required for core engine features that modify memory or attach to processes.
- Use CI to run tests on push and PRs.

## CI / CD

- GitHub Actions recommended for cross-platform CI.
- Build matrix:
  - Windows (Visual Studio)
  - Linux (gcc/clang)
  - macOS (limited test subset)
- Run static analysis, unit tests, packaging, and basic integration tests on PRs.

## How to submit

1. Fork the repository.
2. Create a feature branch.
3. Add tests and documentation.
4. Create a pull request describing the change and linking relevant issues.
5. A maintainer will review; address review comments promptly.

## Security-sensitive contributions

If your contribution touches security-sensitive components (sandbox, kernel drivers, remote agents), follow additional review steps:
- Open a draft PR and notify maintainers directly via email.
- Provide threat model notes and test vectors.
- Include a signed-off-by header in commits.

