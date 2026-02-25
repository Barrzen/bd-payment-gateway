# SSLCOMMERZ Quickstart (Python)

This guide shows a practical end-to-end payment flow for beginners.

Only SSLCOMMERZ is supported right now in Python.

## 1) Install

```bash
pip install bd-payment-gateway
```

## 2) Configure environment variables

```dotenv
BDPG_SSLCOMMERZ_STORE_ID=your_store_id
BDPG_SSLCOMMERZ_STORE_PASSWD=your_store_password
BDPG_SSLCOMMERZ_ENVIRONMENT=sandbox
```

## 3) Create payment, redirect user, and verify

```python
from decimal import Decimal

from flask import Flask, request

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
settings = Settings()
client = SslcommerzClient.from_settings(settings)


@app.post("/checkout")
def checkout() -> tuple[dict, int]:
    try:
        initiated = client.initiate_payment(
            InitiatePaymentRequest(
                total_amount=Decimal("100.00"),
                tran_id="ORDER-1001",
                urls=Urls(
                    success_url="https://merchant.example/payment/success",
                    fail_url="https://merchant.example/payment/fail",
                    cancel_url="https://merchant.example/payment/cancel",
                    ipn_url="https://merchant.example/payment/ipn",
                ),
                customer=Customer(
                    name="Customer Name",
                    email="customer@example.com",
                    address_line_1="Dhaka",
                    city="Dhaka",
                    country="Bangladesh",
                    phone="01700000000",
                ),
                product=Product(
                    name="Course Subscription",
                    category="Education",
                    profile="general",
                ),
            )
        )
    except PaymentGatewayError as err:
        return {"error": err.message, "code": err.code, "hint": err.hint}, 400

    # Return this URL to frontend, then redirect the customer there.
    return {"redirect_url": str(initiated.redirect_url)}, 200


@app.get("/payment/success")
def payment_success() -> tuple[dict, int]:
    session_key = request.args.get("sessionkey", "")
    if not session_key:
        return {"error": "Missing sessionkey"}, 400

    try:
        verified = client.verify_payment(
            VerifyPaymentRequest(session_key=session_key)
        )
    except PaymentGatewayError as err:
        return {"error": err.message, "code": err.code, "hint": err.hint}, 400

    # Mark your order paid only after successful verify status.
    return {"status": verified.status}, 200


@app.get("/payment/fail")
def payment_fail() -> tuple[dict, int]:
    return {"status": "failed"}, 200


@app.get("/payment/cancel")
def payment_cancel() -> tuple[dict, int]:
    return {"status": "cancelled"}, 200


@app.post("/payment/ipn")
def payment_ipn() -> tuple[dict, int]:
    # IPN means Instant Payment Notification from SSLCOMMERZ server.
    return {"received": True}, 200
```

## 4) Payment flow explained

1. Initiate: backend calls `initiate_payment` and receives `redirect_url`.
2. Redirect: frontend sends customer to `redirect_url`.
3. Callback: customer returns to your `success_url` / `fail_url` / `cancel_url`.
4. IPN: SSLCOMMERZ may also send server-to-server update to `ipn_url`.
5. Verify: backend calls `verify_payment` and confirms final status.

## 5) Common mistakes

- Using `float` for money.
  - Use `Decimal("100.00")`.
- Marking order paid before verification.
  - Always verify with `verify_payment` first.
- Missing callback URLs.
  - Set all URLs in `Urls(...)`.
