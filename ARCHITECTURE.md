# Architecture

## Design Goals

- Provider isolation and modular installs
- Consistent API shape across gateways
- Safe production defaults (timeouts/retries/redaction)
- Extensible with minimal core churn
- FFI parity for JS and Python

## Core vs Provider Split

`bd-payment-gateway-core` has no provider-specific logic.

It owns:

- Domain types
- `Environment` routing (`Sandbox`, `Production`, `CustomBaseUrl`)
- `PaymentProvider` trait
- Unified error model (`BdPaymentError`, `ErrorCode`)
- Shared HTTP runtime

Provider crates own:

- Endpoint/auth contracts
- Payload schemas
- Status/error mapping
- Validation rules

This keeps future gateways additive and low-risk.

## Facade Strategy

`bd-payment-gateway` is a feature-gated facade.

- `default = []`
- per-provider features: `shurjopay`, `portwallet`, `aamarpay`, `sslcommerz`

Users only compile providers they enable.

## HTTP Runtime

Shared `HttpClient` behavior:

- reqwest + rustls TLS
- global timeout
- retry with exponential backoff on transient failures (`429`, `5xx`, connect/timeout)
- redacted request/response logging hooks
- correlation/idempotency helper headers

## Error Model

All crates return `BdPaymentError`:

- `ConfigError`
- `ValidationError`
- `HttpError`
- `ProviderError`
- `Unsupported`
- `ParseError`

Every variant includes user-facing hints and a stable `ErrorCode`.

## Binding Packaging Choice

Chosen option: **Option 2** (single package per language with provider-specific feature builds).

Rationale:

- Reduces packaging duplication while preserving modular compile-time features
- Keeps one canonical N-API module and one canonical Python extension
- Still supports install-only-what-you-need by building with selected features

Notes:

- JS: build `bd-payment-gateway-js` with chosen provider features.
- Python: build wheel via maturin with chosen provider features.

## Extensibility Points

- New providers add a crate implementing `PaymentProvider`
- Facade crate only needs additive feature + re-export
- Binding crates only need additive class wrappers
- No core changes required unless a truly cross-provider primitive is missing

## API Ambiguities and Safe Choices

Some provider docs contain incomplete or inconsistent examples.

Safe implementation choices made:

- Status mapping is tolerant of multiple documented field variants (`status`, `pay_status`, `bank_status`, etc.)
- Request/response parsing keeps raw payloads for downstream diagnostics
- Missing/unclear refund contracts return explicit `Unsupported` with guidance
- Base URLs are overrideable via `CustomBaseUrl` to handle provider edge deployments
