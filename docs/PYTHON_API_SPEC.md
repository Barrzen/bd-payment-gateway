# Public Python API Spec

Target: typed-first Python API with mandatory Pydantic validation for SSLCOMMERZ.

## Package Layout

```text
bd_payment_gateway/
  __init__.py
  errors.py
  sslcommerz/
    __init__.py
    client.py
    models.py
    py.typed
```

Extension module remains `bd_payment_gateway_py` and is wrapped by the pure-Python facade.

## Public Imports

```python
from bd_payment_gateway.errors import PaymentGatewayError
from bd_payment_gateway.sslcommerz import SslcommerzClient
from bd_payment_gateway.sslcommerz.models import (
    Settings,
    Urls,
    Customer,
    Product,
    InitiatePaymentRequest,
    VerifyPaymentRequest,
    InitiatePaymentResponse,
    VerifyPaymentResponse,
    FinalStatus,
)
```

## Construction Pattern

```python
settings = Settings()  # loads env and .env
client = SslcommerzClient.from_settings(settings)
```

## Method Signatures

```python
def initiate_payment(self, request: InitiatePaymentRequest) -> InitiatePaymentResponse: ...
def verify_payment(self, request: VerifyPaymentRequest) -> VerifyPaymentResponse: ...
def wait_for_final_status(
    self,
    session_key: str,
    timeout_s: int = 300,
    interval_s: int = 5,
) -> FinalStatus: ...
```

Rules:

- Public methods accept model instances only.
- No public dict/JSON input path.
- Internal bridge maps model to provider payload dict and calls extension API.

## Settings and Validation

`Settings` uses `pydantic-settings` and supports env + `.env` loading.

Expected env names:

- `BDPG_SSLCOMMERZ_STORE_ID`
- `BDPG_SSLCOMMERZ_STORE_PASSWD`
- `BDPG_SSLCOMMERZ_ENVIRONMENT` (`sandbox` | `production` | `custom`)
- `BDPG_SSLCOMMERZ_CUSTOM_BASE_URL` (required when environment is `custom`)

## Amount and Currency Contract

- `total_amount: Decimal`
- `gt=0`
- `decimal_places=2`
- digit cap enforced (`max_digits=12`)
- provider payload outputs decimal as string with fixed `0.01` quantization
- default currency restricted to `"BDT"`

## Request Models

- `Urls` (success/fail/cancel/ipn)
- `Customer` (required merchant-facing fields)
- `Product` (name/category/profile)
- `InitiatePaymentRequest`
- `VerifyPaymentRequest` (exactly one of `session_key`, `val_id`, `tran_id`)

## Response Models

- `InitiatePaymentResponse`
  - `redirect_url`
  - `provider_reference`
  - `request_id`
  - `raw`
- `VerifyPaymentResponse`
  - `status`
  - `provider_reference`
  - `amount`
  - `currency`
  - `request_id`
  - `raw`
- `FinalStatus` enum: `paid`, `failed`, `cancelled`, `refunded`

## Error Contract

`PaymentGatewayError` fields:

- `code: str`
- `message: str`
- `hint: str`
- `provider_payload: dict[str, Any] | None`

Facade wraps extension errors so users never parse strings manually.

## Extension Stubs

`bd_payment_gateway_py.pyi` must declare:

- provider clients
- response classes
- `PaymentGatewayError` attributes
- typed method signatures

## Not In Scope (This iteration)

- typed facade for non-SSLCOMMERZ providers
- async client API
- webhook verification helpers
