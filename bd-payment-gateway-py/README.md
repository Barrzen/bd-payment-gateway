# bd-payment-gateway-py

PyO3 bindings for `bd-payment-gateway`.

## Python Support

- Python 3.14 supported (built with `abi3`, minimum ABI 3.9)

## Build

```bash
cd bd-payment-gateway-py
maturin build --release --features all-providers
```

Build only one provider:

```bash
maturin build --release --features portwallet
```

## API

Provider-specific classes:

- `ShurjopayClient`
- `PortwalletClient`
- `AamarpayClient`
- `SslcommerzClient`

Each constructor accepts config JSON string.

Methods:

- `initiate_payment(request_json: str)`
- `verify_payment(request_json: str)`
- `refund(request_json: str)` where supported

## Error Contract

Errors raise `PaymentGatewayError` with JSON payload string including:

- `message`
- `code`
- `hint`

## Typing

Typing stubs are provided in `bd_payment_gateway_py.pyi`.
