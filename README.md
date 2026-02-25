# bd-payment-gateway

Python and Rust toolkit for Bangladesh payment gateways.

This README is the user guide for the Python package on PyPI: `bd-payment-gateway`.
If you are integrating payments in a Python app, start here.

## Overview

`bd-payment-gateway` helps you:

- create a payment session
- redirect the customer to a hosted payment page
- receive callback/IPN events
- verify final payment status safely

It is designed for backend services where you need clear errors and typed request models.

## Supported Providers (Python)

- SSLCOMMERZ: ✅ Supported and stable
- PortWallet: ❌ Not supported yet (WIP)
- shurjoPay: ❌ Not supported yet (WIP)
- aamarPay: ❌ Not supported yet (WIP)

Only SSLCOMMERZ is supported right now in Python.

## Install

```bash
pip install bd-payment-gateway
```

## Configuration

Use these environment variables (shell or `.env`):

| Env var | Required | Meaning | Example |
| --- | --- | --- | --- |
| `BDPG_SSLCOMMERZ_STORE_ID` | Yes | SSLCOMMERZ store ID | `testbox` |
| `BDPG_SSLCOMMERZ_STORE_PASSWD` | Yes | SSLCOMMERZ store password | `qwerty` |
| `BDPG_SSLCOMMERZ_ENVIRONMENT` | Yes | `sandbox`, `production`, or `custom` | `sandbox` |
| `BDPG_SSLCOMMERZ_CUSTOM_BASE_URL` | Only for `custom` | Override provider base URL | `https://sandbox.sslcommerz.com` |

Example:

```dotenv
BDPG_SSLCOMMERZ_STORE_ID=your_store_id
BDPG_SSLCOMMERZ_STORE_PASSWD=your_store_password
BDPG_SSLCOMMERZ_ENVIRONMENT=sandbox
```

## Quickstart (SSLCOMMERZ)

```python
from decimal import Decimal

from bd_payment_gateway.errors import PaymentGatewayError
from bd_payment_gateway.sslcommerz import SslcommerzClient
from bd_payment_gateway.sslcommerz.models import (
    Customer,
    InitiatePaymentRequest,
    Product,
    Settings,
    Urls,
    VerifyPaymentRequest,
)

settings = Settings()  # Reads BDPG_SSLCOMMERZ_* from env/.env
client = SslcommerzClient.from_settings(settings)

try:
    initiate = client.initiate_payment(
        InitiatePaymentRequest(
            total_amount=Decimal("500.00"),
            tran_id="TXN-10001",
            urls=Urls(
                success_url="https://merchant.example/payment/success",
                fail_url="https://merchant.example/payment/fail",
                cancel_url="https://merchant.example/payment/cancel",
                ipn_url="https://merchant.example/payment/ipn",
            ),
            customer=Customer(
                name="Demo User",
                email="demo@example.com",
                address_line_1="Dhaka",
                city="Dhaka",
                country="Bangladesh",
                phone="01700000000",
            ),
            product=Product(
                name="Python Course",
                category="Education",
                profile="general",
            ),
        )
    )
except PaymentGatewayError as err:
    print(err.code, err.message, err.hint)
    raise

print("Send customer here:", initiate.redirect_url)

# Later, in your callback handler, verify using session_key/sessionkey
verified = client.verify_payment(
    VerifyPaymentRequest(session_key=initiate.provider_reference)
)
print("Final status:", verified.status)
```

## How the payment flow works

1. `initiate_payment`: Your backend creates a session.
2. `redirect_url`: You redirect the customer to SSLCOMMERZ hosted checkout.
3. Callback and IPN:
   - Callback: browser return to your `success_url`, `fail_url`, or `cancel_url`.
   - IPN: server-to-server notification from SSLCOMMERZ to your `ipn_url`.
4. `verify_payment`: Your backend confirms final status before marking order as paid.

## Error handling example

```python
from bd_payment_gateway.errors import PaymentGatewayError

try:
    verified = client.verify_payment(
        VerifyPaymentRequest(session_key="received-from-callback")
    )
except PaymentGatewayError as err:
    # Structured error fields are safe to read directly.
    print("code=", err.code)
    print("message=", err.message)
    print("hint=", err.hint)
    print("provider_payload=", err.provider_payload)
```

## Docs

- Configuration: [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md)
- End-to-end quickstart: [`docs/QUICKSTART_SSLCOMMERZ.md`](docs/QUICKSTART_SSLCOMMERZ.md)
- Troubleshooting: [`docs/TROUBLESHOOTING.md`](docs/TROUBLESHOOTING.md)
- Python API spec: [`docs/PYTHON_API_SPEC.md`](docs/PYTHON_API_SPEC.md)

## Contributing and local build

Build-from-source and contributor tooling are intentionally separated from usage docs.

- Contributor setup: [`docs/DEV_SETUP.md`](docs/DEV_SETUP.md)
- Contribution guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)

## Links

- Repository: <https://github.com/Barrzen/bd-payment-gateway>
- Project home page: <https://github.com/Barrzen/bd-payment-gateway#readme>
- Documentation: <https://github.com/Barrzen/bd-payment-gateway/tree/main/docs>
- Issues: <https://github.com/Barrzen/bd-payment-gateway/issues>
