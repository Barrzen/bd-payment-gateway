# AGENTS.md

## Repository Overview

`bd-payment-gateway` is a modular Rust workspace for Bangladesh payment providers.

Goals:

- Production-grade SDK for initiating/verifying/refunding payments
- Provider isolation (add new gateways without breaking existing API)
- Friendly, actionable errors with stable error codes
- Safe defaults for payment workloads (timeouts, retries, redaction)
- Cross-language bindings (Node/Bun/Deno via N-API, Python 3.14+ via PyO3)

## Workspace Layout

- `bd-payment-gateway-core`
  - Shared domain types (`Money`, `Currency`, `Environment`, etc.)
  - `PaymentProvider` trait
  - `BdPaymentError` + `ErrorCode`
  - HTTP runtime (`HttpClient`) with retries/backoff/redaction
- `bd-payment-gateway-shurjopay`
- `bd-payment-gateway-portwallet`
- `bd-payment-gateway-aamarpay`
- `bd-payment-gateway-sslcommerz`
  - Provider-specific config/client/typed requests/responses/error mapping
- `bd-payment-gateway`
  - Facade crate, feature-gated re-exports, default features = none
- `bd-payment-gateway-js`
  - N-API binding crate (feature-gated providers)
- `bd-payment-gateway-py`
  - PyO3 binding crate (feature-gated providers)
- `docs/provider-template.md`
  - New-provider template and conventions

## Coding Standards

### Rust Style

- Rust stable only
- Workspace baseline: Rust `1.93.0`, edition `2024`
- `cargo fmt` required
- `cargo clippy -- -D warnings` required
- Prefer small modules, typed request/response structs, explicit validation
- Avoid panics in library code
- For JS/Python bindings, expose typed request/config contracts (TypeScript + `.pyi`)

### Error Format

- Use `BdPaymentError` variants only
- Every error must be actionable:
  - human-readable message
  - hint (`how to fix`)
  - stable `ErrorCode` for programmatic handling
- Map provider errors to `ProviderError` with original provider code when possible

### Logging Rules

- Never log raw credentials, signatures, tokens, store/app secrets
- Use core redaction helpers for headers and JSON payload fields
- Include correlation/request IDs in logs where possible
- Keep log payloads minimal and safe for production

### Secrets & Redaction

- Credentials must use `secrecy::SecretString`
- Do not `Debug`/print exposed secrets
- Redact keys containing: token, secret, password, authorization, key, signature, store_id

## Runbook (Local)

- Format: `cargo fmt --all`
- Lint: `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- Test: `cargo test --workspace --all-features`
- Root quality suite: `./scripts/quality-check.sh`
- Run example:
  - `cargo run -p bd-payment-gateway --example portwallet_initiate_verify --features portwallet`

Bindings:

- JS build: `cargo build -p bd-payment-gateway-js --all-features`
- Python build: `cargo build -p bd-payment-gateway-py --all-features`
- Python env/tooling: use `uv` locally
  - `source $HOME/.local/bin/env`
  - `cd bd-payment-gateway-py && uv sync --group dev`
  - `cd bd-payment-gateway-py && uv run maturin build --release --features all-providers`

## Add New Provider in 30 Minutes (Checklist)

1. Create crate: `bd-payment-gateway-<provider>`.
2. Add it to workspace `Cargo.toml` members.
3. Implement `Config` + `Client` with:
  - `environment: Environment`
  - `http_settings: HttpSettings`
  - input `validate()` methods
4. Implement `PaymentProvider` trait:
  - `initiate_payment`
  - `verify_payment`
  - `refund` (or return `Unsupported` with clear hint)
5. Use core `HttpClient` (timeouts/retries/redaction already enforced).
6. Map provider error payloads into `BdPaymentError::ProviderError`.
7. Add unit tests:
  - request validation
  - auth/signature logic (if applicable)
  - parsing/error mapping via mock responses
8. Add facade feature in `bd-payment-gateway/Cargo.toml`.
9. Re-export in `bd-payment-gateway/src/lib.rs` behind feature gate.
10. Add binding support:
  - JS class + constructor in `bd-payment-gateway-js`
  - Python class in `bd-payment-gateway-py`
11. Add README example for provider crate.
12. Update root README supported providers table.
13. Run fmt/clippy/test before opening PR.

## Release & Versioning Policy

- SemVer across all public crates
- `0.x`: minor may include breaking changes; still document clearly
- Once `1.0.0+`:
  - no breaking public API changes in minor/patch releases
  - breaking changes only in major release
- Feature flags:
  - facade default features must remain empty
  - adding a new provider must be additive feature flag
  - existing feature names are stable API
- Breaking change rules:
  - changing request/response field names/types is breaking
  - renaming/removing error codes is breaking
  - changing default security behavior must be documented and versioned carefully
