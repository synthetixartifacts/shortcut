# Security Policy

ShortCut handles user-provided API keys and interacts with third-party AI providers. We take security reports seriously.

## Supported Versions

Only the latest tagged release on `main` receives security fixes during the initial release cycle.

| Version | Supported |
|---------|-----------|
| latest release | yes |
| earlier releases | no |

## Reporting a Vulnerability

**Do not open a public issue for security bugs.**

Report vulnerabilities privately via GitHub's Private Vulnerability Reporting:

<https://github.com/synthetixartifacts/shortcut/security/advisories/new>

Please include:

- A clear description of the issue and its impact
- Steps to reproduce, including OS, app version, and configured provider(s)
- Any relevant logs (redact API keys before sharing)
- Your preferred attribution, if you want public credit after a fix ships

## Response

- We aim to acknowledge reports within 7 days.
- We will keep you updated on the status of the investigation and the fix timeline.
- Once a fix is released, we will publish a security advisory crediting the reporter (unless you request anonymity).

## Scope

In scope:

- The ShortCut desktop app (Tauri binary, Rust backend, Svelte frontend)
- Build and release artifacts published on the Releases page
- Provider integrations that could leak user API keys or intercept provider traffic

Out of scope:

- Issues in upstream providers (OpenAI, Anthropic, Gemini, Grok, Ollama, Soniox, ONNX Runtime, Parakeet) — report those upstream
- Vulnerabilities that require physical access to an unlocked, user-authenticated device
- Social-engineering of end users

## Safe Harbor

We will not pursue legal action against researchers who act in good faith, follow this policy, and avoid privacy violations, service disruption, or data destruction. Please stop and contact us if you are unsure whether your testing crosses those lines.
