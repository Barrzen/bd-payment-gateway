# bd-payment-gateway (Python)

`bd-payment-gateway` is a high-performance, Rust-powered, security-focused
Bangladesh payment gateway library for Python.
It is built for production checkout flows with typed request models, actionable
error codes, and safe defaults.

This SDK family is available in multiple languages:

- Python: `bd-payment-gateway` (this package)
- JavaScript/TypeScript: [`bd-payment-gateway-js`](https://github.com/Barrzen/bd-payment-gateway/tree/main/bd-payment-gateway-js)
- Rust: [`bd-payment-gateway`](https://github.com/Barrzen/bd-payment-gateway/tree/main/bd-payment-gateway)

## Supported Providers (Python)

- SSLCOMMERZ: ✅ Supported and stable
- PortWallet: ❌ Not supported yet (WIP)
- shurjoPay: ❌ Not supported yet (WIP)
- aamarPay: ❌ Not supported yet (WIP)

Only SSLCOMMERZ is supported right now in Python.

## Install

Use the copy button on each command block when your docs viewer supports it.

```bash
pip install bd-payment-gateway
```

```bash
uv add bd-payment-gateway
```

```bash
uv pip install bd-payment-gateway
```

## Provider Docs (Python)

<details open>
<summary><strong>SSLCOMMERZ (default)</strong></summary>

### Configuration

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

### Quickstart (FastAPI)

This example shows the full payment flow:
1. Create payment session
2. Send user to redirect URL
3. Handle callback endpoints (`success`, `fail`, `cancel`, `ipn`)
4. Verify payment status

```python
from decimal import Decimal

from fastapi import FastAPI, Request

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

app = FastAPI()
settings = Settings()  # Reads BDPG_SSLCOMMERZ_* from env/.env
client = SslcommerzClient.from_settings(settings)


@app.post("/pay")
def pay() -> dict:
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
        return {"error": err.message, "code": err.code, "hint": err.hint}

    # Frontend should redirect the customer to this URL.
    return {"redirect_url": str(initiated.redirect_url)}


@app.get("/payment/success")
def payment_success(request: Request) -> dict:
    # SSLCOMMERZ sends sessionkey in callback query params.
    session_key = request.query_params.get("sessionkey", "")
    if not session_key:
        return {"error": "Missing sessionkey in callback"}

    try:
        verified = client.verify_payment(
            VerifyPaymentRequest(session_key=session_key)
        )
    except PaymentGatewayError as err:
        return {"error": err.message, "code": err.code, "hint": err.hint}

    return {
        "status": verified.status,
        "provider_reference": verified.provider_reference,
        "amount": str(verified.amount) if verified.amount else None,
    }


@app.get("/payment/fail")
def payment_fail() -> dict:
    return {"status": "failed"}


@app.get("/payment/cancel")
def payment_cancel() -> dict:
    return {"status": "cancelled"}


@app.post("/payment/ipn")
def payment_ipn() -> dict:
    # IPN = Instant Payment Notification from SSLCOMMERZ server.
    return {"received": True}
```

### Alternative without environment variables

You can configure the client directly without setting `BDPG_SSLCOMMERZ_*` vars:

```python
from bd_payment_gateway.sslcommerz import SslcommerzClient
from bd_payment_gateway.sslcommerz.models import Settings

settings = Settings(
    store_id="your_store_id",
    store_passwd="your_store_password",
    environment="sandbox",
)
client = SslcommerzClient.from_settings(settings)
```

</details>

<details>
<summary><strong>PortWallet (Python: coming soon)</strong></summary>

Python bindings are not published yet.

- Rust crate docs: <https://github.com/Barrzen/bd-payment-gateway/tree/main/bd-payment-gateway-portwallet>
- JavaScript package docs: <https://github.com/Barrzen/bd-payment-gateway/tree/main/bd-payment-gateway-js>

</details>

<details>
<summary><strong>shurjoPay (Python: coming soon)</strong></summary>

Python bindings are not published yet.

- Rust crate docs: <https://github.com/Barrzen/bd-payment-gateway/tree/main/bd-payment-gateway-shurjopay>
- JavaScript package docs: <https://github.com/Barrzen/bd-payment-gateway/tree/main/bd-payment-gateway-js>

</details>

<details>
<summary><strong>aamarPay (Python: coming soon)</strong></summary>

Python bindings are not published yet.

- Rust crate docs: <https://github.com/Barrzen/bd-payment-gateway/tree/main/bd-payment-gateway-aamarpay>
- JavaScript package docs: <https://github.com/Barrzen/bd-payment-gateway/tree/main/bd-payment-gateway-js>

</details>

## Common Errors and Troubleshooting (SSLCOMMERZ)

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

Have a suggestion, integration request, or bug report? Please open an issue:
<https://github.com/Barrzen/bd-payment-gateway/issues/new/choose>
