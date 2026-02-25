# bd-payment-gateway (Python)

`bd-payment-gateway` helps you collect online payments in Bangladesh using a Python API.
It gives you a typed SSLCOMMERZ client for create-session, redirect, callback handling, and payment verification.
You can start from environment variables and call strongly typed request models.

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

Set these environment variables in your shell or `.env` file.

| Environment variable | Meaning | Example |
| --- | --- | --- |
| `BDPG_SSLCOMMERZ_STORE_ID` | Your SSLCOMMERZ store ID | `testbox` |
| `BDPG_SSLCOMMERZ_STORE_PASSWD` | Your SSLCOMMERZ store password | `qwerty` |
| `BDPG_SSLCOMMERZ_ENVIRONMENT` | `sandbox`, `production`, or `custom` | `sandbox` |
| `BDPG_SSLCOMMERZ_CUSTOM_BASE_URL` | Required only when environment is `custom` | `https://sandbox.sslcommerz.com` |

Example:

```dotenv
BDPG_SSLCOMMERZ_STORE_ID=your_store_id
BDPG_SSLCOMMERZ_STORE_PASSWD=your_store_password
BDPG_SSLCOMMERZ_ENVIRONMENT=sandbox
# BDPG_SSLCOMMERZ_CUSTOM_BASE_URL=https://sandbox.sslcommerz.com
```

## Quickstart (SSLCOMMERZ)

This example shows the full payment flow:
1. Create payment session
2. Send user to redirect URL
3. Handle callback endpoints (`success`, `fail`, `cancel`, `ipn`)
4. Verify payment status

```python
from decimal import Decimal

from flask import Flask, jsonify, request

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

app = Flask(__name__)
settings = Settings()  # Reads BDPG_SSLCOMMERZ_* from env/.env
client = SslcommerzClient.from_settings(settings)


@app.post("/pay")
def pay() -> tuple[dict, int]:
    try:
        initiated = client.initiate_payment(
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
        return {"error": err.message, "code": err.code, "hint": err.hint}, 400

    # Frontend should redirect the customer to this URL.
    return {"redirect_url": str(initiated.redirect_url)}, 200


@app.get("/payment/success")
def payment_success() -> tuple[dict, int]:
    # SSLCOMMERZ sends sessionkey in callback query params.
    session_key = request.args.get("sessionkey", "")
    if not session_key:
        return {"error": "Missing sessionkey in callback"}, 400

    try:
        verified = client.verify_payment(
            VerifyPaymentRequest(session_key=session_key)
        )
    except PaymentGatewayError as err:
        return {"error": err.message, "code": err.code, "hint": err.hint}, 400

    return {
        "status": verified.status,
        "provider_reference": verified.provider_reference,
        "amount": str(verified.amount) if verified.amount else None,
    }, 200


@app.get("/payment/fail")
def payment_fail() -> tuple[dict, int]:
    return {"status": "failed"}, 200


@app.get("/payment/cancel")
def payment_cancel() -> tuple[dict, int]:
    return {"status": "cancelled"}, 200


@app.post("/payment/ipn")
def payment_ipn() -> tuple[dict, int]:
    # IPN = Instant Payment Notification from SSLCOMMERZ server.
    return jsonify({"received": True}), 200
```

## Common Errors and Troubleshooting

- `ValidationError` when loading `Settings`:
  - Cause: missing `BDPG_SSLCOMMERZ_*` values.
  - Fix: set required environment variables exactly as shown above.
- `PaymentGatewayError` with provider error code:
  - Cause: wrong credentials, invalid callback URLs, or invalid transaction data.
  - Fix: check `err.code`, `err.message`, and `err.hint` in your `except` block.
- Payment stays pending:
  - Cause: customer has not completed payment yet.
  - Fix: verify again after callback/IPN or poll with your own retry logic.

## Links

- GitHub repository: <https://github.com/Barrzen/bd-payment-gateway>
- Project home page: <https://github.com/Barrzen/bd-payment-gateway#readme>
- Documentation index: <https://github.com/Barrzen/bd-payment-gateway/tree/main/docs>
- SSLCOMMERZ quickstart doc: <https://github.com/Barrzen/bd-payment-gateway/blob/main/docs/QUICKSTART_SSLCOMMERZ.md>
- Python API spec: <https://github.com/Barrzen/bd-payment-gateway/blob/main/docs/PYTHON_API_SPEC.md>
- Issue tracker: <https://github.com/Barrzen/bd-payment-gateway/issues>
