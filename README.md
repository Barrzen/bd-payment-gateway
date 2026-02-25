# bd-payment-gateway

High-performance Bangladesh payment gateway SDK, powered by Rust.

This repository provides a multi-language SDK family:

- Python: [`bd-payment-gateway`](https://pypi.org/project/bd-payment-gateway/)
- JavaScript/TypeScript: [`bd-payment-gateway-js`](bd-payment-gateway-js/README.md)
- Rust: [`bd-payment-gateway`](bd-payment-gateway/README.md) facade + provider crates

## Python Home

The Python package `bd-payment-gateway` is security-focused and designed for
production payment workloads with typed models and actionable errors.

## Supported Providers (Python)

- SSLCOMMERZ: ✅ Supported and stable
- PortWallet: ❌ Not supported yet (WIP)
- shurjoPay: ❌ Not supported yet (WIP)
- aamarPay: ❌ Not supported yet (WIP)

Only SSLCOMMERZ is supported right now in Python.

## Install (Python)

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

### Quickstart (FastAPI)

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
        return {"error": err.message, "code": err.code, "hint": err.hint}

    return {"redirect_url": str(initiate.redirect_url)}

@app.get("/payment/success")
def payment_success(request: Request) -> dict:
    session_key = request.query_params.get("sessionkey", "")
    if not session_key:
        return {"error": "Missing sessionkey in callback"}

    try:
        verified = client.verify_payment(VerifyPaymentRequest(session_key=session_key))
    except PaymentGatewayError as err:
        return {"error": err.message, "code": err.code, "hint": err.hint}

    return {"status": verified.status}
```

### Alternative without environment variables

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

## Docs

- Configuration: [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md)
- End-to-end quickstart: [`docs/QUICKSTART_SSLCOMMERZ.md`](docs/QUICKSTART_SSLCOMMERZ.md)
- Troubleshooting: [`docs/TROUBLESHOOTING.md`](docs/TROUBLESHOOTING.md)
- Python API spec: [`docs/PYTHON_API_SPEC.md`](docs/PYTHON_API_SPEC.md)

## Contributing and local build

- Contributor setup: [`docs/DEV_SETUP.md`](docs/DEV_SETUP.md)
- Contribution guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)

## Feedback and Issues

Have a suggestion, integration request, or bug report? Open an issue:

- Issues: <https://github.com/Barrzen/bd-payment-gateway/issues>
- New issue form: <https://github.com/Barrzen/bd-payment-gateway/issues/new/choose>
