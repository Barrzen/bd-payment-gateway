# Python API Inventory (Current State)

Snapshot of the current Python surface before typed-first refactor.

Captured from commit `0b1c0c6` to preserve pre-refactor baseline.

## Package and Import Paths

- Distribution name: `bd-payment-gateway` (from `bd-payment-gateway-py/pyproject.toml`)
- Compiled extension module: `bd_payment_gateway_py`
- Current public imports are extension-level only (no pure-Python package namespace yet).

## Public Classes Exposed by Extension

From `bd-payment-gateway-py/src/lib.rs`:

- `PaymentGatewayError`
- `InitiatePaymentResponse`
- `VerifyPaymentResponse`
- `RefundResponse`
- `ShurjopayClient` (feature-gated)
- `PortwalletClient` (feature-gated)
- `AamarpayClient` (feature-gated)
- `SslcommerzClient` (feature-gated)

## Constructor and Method Behavior

All provider client constructors and methods accept either:

- JSON string input, or
- JSON-serializable Python object/mapping

Current signatures (effective behavior):

- `Client(config)`
- `client.initiate_payment(request)`
- `client.verify_payment(request)`
- `client.refund(request)` for providers that support refund

Validation currently happens in Rust after deserialization, not in Python models.

## Response Attributes Exposed to Python

`InitiatePaymentResponse`:

- `redirect_url: str`
- `provider_reference: str`
- `raw: str` (JSON string)
- `request_id: str | None`

`VerifyPaymentResponse`:

- `status: str`
- `provider_reference: str`
- `amount: str | None`
- `currency: str | None`
- `raw: str` (JSON string)
- `request_id: str | None`

`RefundResponse`:

- `status: str`
- `provider_reference: str`
- `raw: str` (JSON string)
- `request_id: str | None`

## Exception Behavior

`PaymentGatewayError` currently carries a JSON string message. That payload includes:

- `message`
- `code`
- `hint`

Current ergonomics require `json.loads(str(exc))` to read structured fields.

## Provider Modules and Source Locations

- Python extension code: `bd-payment-gateway-py/src/lib.rs`
- Existing extension stub at capture time: `bd-payment-gateway-py/bd_payment_gateway_py.pyi`
- Provider Rust implementations:
  - `bd-payment-gateway-sslcommerz/src/lib.rs`
  - `bd-payment-gateway-portwallet/src/lib.rs`
  - `bd-payment-gateway-shurjopay/src/lib.rs`
  - `bd-payment-gateway-aamarpay/src/lib.rs`

## Observed Gaps vs Target Typed-First DX

- No `bd_payment_gateway.sslcommerz` package namespace
- No Pydantic-validated models as required API inputs
- No model-only facade methods
- No first-class structured Python exception attributes
- No `py.typed` marker in installed package
