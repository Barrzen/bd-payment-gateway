# bd-payment-gateway-py

Python package + PyO3 extension for `bd-payment-gateway`.

## Python Support

- Python 3.9+ supported (built with `abi3`, minimum ABI 3.9; tested on 3.14 in CI)

## Local Tooling

Use `uv` locally for Python dependency and command execution.

```bash
source $HOME/.local/bin/env
cd bd-payment-gateway-py
uv sync --group dev
```

## Build

```bash
source $HOME/.local/bin/env
cd bd-payment-gateway-py
uv run maturin build --release --features all-providers
```

Build only one provider:

```bash
source $HOME/.local/bin/env
uv run maturin build --release --features portwallet
```

## Typed API

Primary public API for SSLCOMMERZ:

- `bd_payment_gateway.sslcommerz.SslcommerzClient`
- `bd_payment_gateway.sslcommerz.models`
  - `Settings`
  - `Urls`
  - `Customer`
  - `Product`
  - `InitiatePaymentRequest`
  - `VerifyPaymentRequest`

The compiled extension is internal (`bd_payment_gateway._bd_payment_gateway_py`).
Application code should use the facade and models, not raw dict payloads.

Extension-level provider classes still support JSON/mapping configs and can accept optional
`http_settings` keys:

- `timeout_ms`
- `max_retries`
- `initial_backoff_ms`
- `max_backoff_ms`
- `user_agent`

## Smoke Test Example

Run SSLCOMMERZ sandbox full-flow test (real initiate + verify polling):

```bash
cd bd-payment-gateway-py
uv run python examples/smoke_test.py
```

Recommended environment variables (override temporary hardcoded sandbox credentials):

```bash
export SSLCOMMERZ_STORE_ID="your_store_id"
export SSLCOMMERZ_STORE_PASSWD="your_store_passwd"
export SSLCOMMERZ_RETURN_BASE_URL="https://your-registered-domain.com"
```

Or create a local `examples/.env` file from `examples/.env.example`.

## Typing

- The package is typed (`py.typed` included).
- Extension stubs are provided at `python/bd_payment_gateway/_bd_payment_gateway_py.pyi`.
- Pydantic v2 + pydantic-settings are required runtime dependencies.

## Error Contract

Use `bd_payment_gateway.errors.PaymentGatewayError`.

Structured fields:

- `code`
- `message`
- `hint`
- `provider_payload`
