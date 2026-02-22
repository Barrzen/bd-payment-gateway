# bd-payment-gateway-py

PyO3 bindings for `bd-payment-gateway`.

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

## API

Provider-specific classes:

- `ShurjopayClient`
- `PortwalletClient`
- `AamarpayClient`
- `SslcommerzClient`

Constructors and methods accept either:

- JSON string
- Typed Python mapping/dict (recommended)

Config payloads may include optional `http_settings`:

- `timeout_ms`
- `max_retries`
- `initial_backoff_ms`
- `max_backoff_ms`
- `user_agent`

Methods:

- `initiate_payment(request)`
- `verify_payment(request)`
- `refund(request)` where supported

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

- Typed request/config contracts are provided in `bd_payment_gateway_py.pyi`.
- `pydantic` is available in dev dependencies if you want stronger runtime validation in app code.

## Error Contract

Errors raise `PaymentGatewayError` with JSON payload string including:

- `message`
- `code`
- `hint`
