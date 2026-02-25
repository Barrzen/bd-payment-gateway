# Troubleshooting (Python)

This page covers common issues when integrating SSLCOMMERZ.

Only SSLCOMMERZ is supported right now in Python.

## `ValidationError` on `Settings()`

### Symptom

App fails when creating `Settings()`.

### Cause

Required env vars are missing or invalid.

### Fix

- Set `BDPG_SSLCOMMERZ_STORE_ID`.
- Set `BDPG_SSLCOMMERZ_STORE_PASSWD`.
- Set `BDPG_SSLCOMMERZ_ENVIRONMENT` to `sandbox`, `production`, or `custom`.
- If `custom`, also set `BDPG_SSLCOMMERZ_CUSTOM_BASE_URL`.

## `PaymentGatewayError` with auth/merchant errors

### Symptom

`initiate_payment` or `verify_payment` raises `PaymentGatewayError`.

### Cause

Usually wrong credentials, invalid callback URLs, or malformed request fields.

### Fix

Inspect structured fields in exception:

```python
from bd_payment_gateway.errors import PaymentGatewayError

try:
    # call client method
    pass
except PaymentGatewayError as err:
    print(err.code)
    print(err.message)
    print(err.hint)
    print(err.provider_payload)
```

Then fix credentials/URLs/request data based on `hint`.

## Payment remains `pending`

### Symptom

`verify_payment` returns `pending` repeatedly.

### Cause

Customer has not completed payment, or callback/IPN has not reached your app yet.

### Fix

- Wait and retry verification.
- Ensure callback URLs are reachable over HTTPS.
- Confirm app can receive IPN requests publicly.

## Callback endpoint does not receive `sessionkey`

### Symptom

Success endpoint gets called but cannot verify because session key is missing.

### Cause

Query parameter read logic is wrong or callback URL mismatch.

### Fix

Read `sessionkey` from query params and keep callback URL exactly aligned with values sent in `Urls(...)`.

## Import error for native extension

### Symptom

Extension module import fails.

### Cause

Broken environment or incomplete install.

### Fix

- Reinstall package in a clean virtual environment:
  - `pip install --upgrade --force-reinstall bd-payment-gateway`
- If building locally, follow contributor setup:
  - [`docs/DEV_SETUP.md`](DEV_SETUP.md)
