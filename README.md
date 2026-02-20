# bd-payment-gateway

Production-grade, modular Rust SDK workspace for Bangladesh payment gateways.

Toolchain baseline:

- Rust `1.93.0`
- Rust edition `2024`

## Supported Providers

- shurjoPay REST API (`bd-payment-gateway-shurjopay`)
- PortWallet API v2 (`bd-payment-gateway-portwallet`)
- aamarPay REST API (`bd-payment-gateway-aamarpay`)
- SSLCOMMERZ integration API (`bd-payment-gateway-sslcommerz`)

## Workspace Crates

- `bd-payment-gateway-core`: shared types, trait, error model, resilient HTTP client
- `bd-payment-gateway-shurjopay`: shurjoPay implementation
- `bd-payment-gateway-portwallet`: PortWallet implementation
- `bd-payment-gateway-aamarpay`: aamarPay implementation
- `bd-payment-gateway-sslcommerz`: SSLCOMMERZ implementation
- `bd-payment-gateway`: feature-gated facade crate (default features: none)
- `bd-payment-gateway-js`: N-API bindings for Node/Bun/Deno
- `bd-payment-gateway-py`: PyO3/maturin bindings for Python 3.14+

## Install (Rust)

Use only the provider you need.

```toml
[dependencies]
bd-payment-gateway = { version = "0.1", default-features = false, features = ["portwallet"] }
```

Or depend directly on one provider crate:

```toml
[dependencies]
bd-payment-gateway-portwallet = "0.1"
```

## Quickstart (Rust)

```rust
use bd_payment_gateway::core::{Environment, PaymentProvider};
use bd_payment_gateway::portwallet::{Config, PortwalletClient};
use secrecy::SecretString;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = PortwalletClient::new(Config {
    app_key: "app_key".to_owned(),
    app_secret: SecretString::new("app_secret".to_owned().into()),
    environment: Environment::Sandbox,
    http_settings: bd_payment_gateway::core::HttpSettings::default(),
})?;
# Ok(()) }
```

Provider examples:

- `bd-payment-gateway/examples/shurjopay_initiate_verify.rs`
- `bd-payment-gateway/examples/portwallet_initiate_verify.rs`
- `bd-payment-gateway/examples/aamarpay_initiate_verify.rs`
- `bd-payment-gateway/examples/sslcommerz_initiate_verify.rs`

## Sandbox vs Production

All providers support:

- `Environment::Sandbox`
- `Environment::Production`
- `Environment::CustomBaseUrl(url)`

Base URLs are mapped per provider docs in each crate.

## Security & Reliability Defaults

- `rustls` TLS via `reqwest`
- request timeout defaults
- retry with exponential backoff on transient failures (`429` and `5xx`)
- correlation ID + idempotency key helpers
- secret redaction in logs and error-safe metadata
- friendly errors with stable structured codes

## Bindings

### JavaScript (Node/Bun/Deno)

- Crate: `bd-payment-gateway-js`
- Build with provider feature flags (Option 2 modular strategy)
- Includes TypeScript request/config/response types in `bd-payment-gateway-js/index.d.ts`

### Python (3.14+)

- Crate: `bd-payment-gateway-py`
- Built with PyO3 + maturin (abi3)
- Local Python workflow uses `uv`
- Typing stubs included; see `bd-payment-gateway-py/bd_payment_gateway_py.pyi`

## Development

- `cargo fmt --all`
- `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo run -p bd-payment-gateway --example portwallet_initiate_verify --features portwallet`
- `source $HOME/.local/bin/env`
- `cd bd-payment-gateway-py && uv sync --group dev && uv run maturin build --release --features all-providers`
- `./scripts/quality-check.sh` (root common quality/binding/provider test suite)

See:

- `AGENTS.md` for agent/contributor instructions
- `ARCHITECTURE.md` for design and extensibility
- `CONTRIBUTING.md` for workflow and PR checklist

## Official API Docs Used

- <https://shurjopay.com.bd/developers/shurjopay-restapi>
- <https://developer.portwallet.com/documentation-v2.php>
- <https://aamarpay.readme.io/reference/overview>
- <https://aamarpay.readme.io/reference/initiate-payment-json>
- <https://sslcommerz.com/integration-document/>
