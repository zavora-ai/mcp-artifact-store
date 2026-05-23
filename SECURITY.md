# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.x     | ✅        |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report via email: james.karanja@zavora.ai

Subject: `[mcp-artifact-store] Security Vulnerability`

We will acknowledge receipt within 48 hours.

## Security Design

- Content-immutable: blobs cannot be silently modified
- SHA-256 integrity on every version
- Retention enforcement prevents unauthorized deletion
- Policy-gated reads for sensitive artifacts
- Redaction creates derived copies, never mutates originals
