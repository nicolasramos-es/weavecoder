# Security Policy

We take the security of Weavecoder seriously. If you believe you have found a
security vulnerability, please report it responsibly.

## Reporting a Vulnerability

### GitHub Security Advisories

Please use **GitHub Security Advisories** to submit a private advisory:

**https://github.com/nicolasramos-es/weavecoder/security/advisories**

This is the preferred and only reporting channel. It allows for coordinated,
private disclosure and keeps the report out of the public issue tracker until
a fix is ready.

If the Security Advisories feature is not enabled for the repository, please
open a regular issue with the word `SECURITY` in the title, without including
exploit details in the public description.

Please include the following information in your report:

- **Type of vulnerability** (e.g., RCE, XSS, injection, auth bypass, DoS)
- **Version(s) affected** (e.g., `0.67.0`, or "all versions")
- **Platform(s)** affected (Linux, macOS, Windows)
- **Steps to reproduce** a minimal, clear reproduction case
- **Potential impact** — your assessment of the severity and blast radius
- **Suggested fix** (optional) — if you have one

## Scope

The following components are in scope:

- The `wvc` CLI binary and all its subcommands
- Agent swarm orchestration logic
- Code Knowledge Graph (tree-sitter integration, SQLite storage)
- Local embedding pipeline
- Authentication flows (Ollama, LM Studio, oMLX/llama.cpp, OpenAI-compatible,
  and cloud providers)
- Installer scripts (`scripts/install.sh`, `scripts/install.ps1`)

The following are **out of scope** (these are external dependencies):

- The underlying LLM models themselves
- Third-party services (Ollama server, LM Studio, etc.)
- The user's local environment configuration

## Response Timeline

- **Acknowledgement**: within 48 hours of report submission
- **Assessment**: within 5 business days
- **Fix target**: agreed upon with the reporter based on severity

## Severity Guidelines

| Severity | Description |
|---|---|
| **Critical** | Remote code execution, arbitrary file read/write, auth bypass |
| **High** | Information disclosure, privilege escalation, denial of service |
| **Medium** | Unexpected behaviour with security implications |
| **Low** | Minor information leakage, edge-case issues |

## Coordinated Disclosure

We follow a coordinated disclosure model:

1. Reporter submits the vulnerability (via GitHub Security Advisories)
2. We acknowledge and assess within 48 hours
3. We work on a fix and keep the report private
4. Once a fix is ready, we coordinate a public disclosure date
5. The vulnerability is disclosed publicly with credit to the reporter

## Security Best Practices for Users

- Always verify installer checksums against the published values
- Keep Weavecoder updated to the latest version
- Use pinned versions in production environments
- Review the `scripts/install.sh` / `scripts/install.ps1` scripts before
  running them
