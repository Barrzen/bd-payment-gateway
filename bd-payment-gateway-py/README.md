# bd-payment-gateway-py

PyO3 bindings for `bd-payment-gateway`.

## Python Support

- Python 3.14 supported (built with `abi3`, minimum ABI 3.9)

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

Methods:

- `initiate_payment(request)`
- `verify_payment(request)`
- `refund(request)` where supported

## Typing

- Typed request/config contracts are provided in `bd_payment_gateway_py.pyi`.
- `pydantic` is available in dev dependencies if you want stronger runtime validation in app code.

## Error Contract

Errors raise `PaymentGatewayError` with JSON payload string including:

- `message`
- `code`
- `hint`
