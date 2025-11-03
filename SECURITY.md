# Security Policy for Inspectra

This document describes how security vulnerabilities are handled and how to report them.

## Supported Versions

We use semantic versioning. Supported versions will be listed in this file with their maintenance policy.

## Reporting a vulnerability

Do not open a public issue. To report a vulnerability, send an email to:

kevin.gregoire@nodasys.com

Include:
- Affected version(s)
- Full description and steps to reproduce
- PoC and test binaries if applicable
- Suggested mitigations

We will acknowledge receipt within 72 hours and provide a remediation timeline.

## Responsible disclosure

- Do not leak details publicly before a fix is available.
- Coordinate disclosure timeline with the Inspectra team.

## Security considerations for contributors

- New dependencies must be reviewed for known vulnerabilities.
- All third-party code must be scanned and have licenses compatible with the project.
- CI must run dependency scanning (e.g., GitHub Dependabot, Snyk) and static analysis.

## Emergency contacts

- Primary: kevin.gregoire@nodasys.com

