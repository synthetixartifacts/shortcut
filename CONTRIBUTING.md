# Contributing to ShortCut

By opening a pull request, you agree that your contribution is licensed under the repository's [LICENSE](LICENSE) (PolyForm Noncommercial 1.0.0). This is the project's lightweight inbound-license rule.

Please also read our [Code of Conduct](CODE_OF_CONDUCT.md).

## Scope

**In scope** (PRs likely to be accepted):
- Bug fixes
- New LLM or STT provider integrations
- New i18n locales
- Documentation improvements
- Performance and accessibility fixes

**Out of scope** (please open an issue first to discuss):
- Whole-app rewrites or large architectural changes
- New proprietary / closed-source dependencies
- Features that require a server-side proxy (the project is deliberately direct-to-provider)

## Development Setup

See [docs/SETUP.md](docs/SETUP.md) for full environment setup instructions.

Quick start:
```bash
docker compose build check
docker compose run --rm check
docker compose up frontend-dev
```

`frontend-dev` is a web UI preview only. Use Docker-built desktop artifacts from `dist/` for native feature smoke tests.

## Adding a New LLM Provider

1. Create `src-tauri/src/providers/{name}.rs` implementing the `LlmProvider` trait:
   ```rust
   #[async_trait::async_trait]
   impl LlmProvider for YourProvider {
       async fn complete(&self, req: &ChatRequest) -> Result<String, AppError> { ... }
       async fn stream(&self, req: &ChatRequest, sink: &EventSinkFn) -> Result<(), AppError> { ... }
       fn capabilities(&self) -> ProviderCapabilities { ... }
       fn provider_id(&self) -> &'static str { "your-provider" }
   }
   ```

2. Add a factory case in `providers/mod.rs::get_llm_provider()` matching the new `provider_id`.

3. Add credential fields in `config/types.rs::ProviderCredentials`.

4. Add the provider to the Settings UI (API key input + task assignment dropdown option).

5. Add i18n keys to all 3 locale files (`src/lib/i18n/locales/{en,fr,es}.json`).

See [docs/PROVIDERS.md](docs/PROVIDERS.md) for provider configuration details and [docs/EXTENDING.md](docs/EXTENDING.md) for the full extension pattern guide.

## Adding a New STT Provider

The STT path is in `src-tauri/src/transcription/`. The active engine is read from
`config.transcription.active_engine` and dispatched in `transcription/mod.rs::transcribe_audio`.

Steps:
1. Create `src-tauri/src/transcription/{name}_provider.rs`.
2. Add the engine ID to the match in `transcription/mod.rs`.
3. Add any required credentials to `ProviderCredentials` in `config/types.rs`.
4. Surface engine selection in the Dictation settings UI.

## Code Style

- **Rust**: target <200 lines per file, hard limit 300. Use `thiserror` for errors via `AppError`.
- **TypeScript/Svelte**: Svelte 5 runes only (`$state`, `$derived`, `$effect`). No `$:` reactive syntax.
- **State**: use `.svelte.ts` files in `src/lib/state/` — no stores.
- **No new dependencies** without prior discussion.

## Validation

```bash
docker compose build check && docker compose run --rm check   # Standard validation
docker compose build audit && docker compose run --rm audit   # Dead-code audit
```

## Testing

Manual end-to-end test with real API keys is required before merging provider changes. Run the artifact produced by Docker on the target OS for hotkeys, clipboard, overlay, and provider verification.
Automated tests are not yet set up; contributions welcome.

## Pull Request Checklist

- [ ] `docker compose build check && docker compose run --rm check` passes
- [ ] No file exceeds 300 lines
- [ ] New i18n keys added to `en.json`, `fr.json`, `es.json`
- [ ] CSS uses variables from `app.css` (no hardcoded colors)
- [ ] All user-visible strings use `t()` from `src/lib/i18n`
- [ ] I agree my contribution is licensed under the repository's LICENSE
