# bd-payment-gateway-js

N-API bindings for `bd-payment-gateway`.

## Runtime Support

- Node.js (N-API)
- Bun (N-API)
- Deno (Node compatibility / N-API loader)

## Modular Build Strategy

Option 2: single package with provider-specific feature builds.

Examples:

- `cargo build -p bd-payment-gateway-js --features shurjopay`
- `cargo build -p bd-payment-gateway-js --features portwallet`
- `cargo build -p bd-payment-gateway-js --features all-providers`

If a provider was not enabled during native build, its factory function throws an actionable error that names the missing Rust feature.

## API Shape

Factory functions (camelCase):

- `createShurjopayClient(config)`
- `createPortwalletClient(config)`
- `createAamarpayClient(config)`
- `createSslcommerzClient(config)`

Backward-compatible aliases (snake_case) are also exported.

Client methods:

- `initiatePayment(request)`
- `verifyPayment(request)`
- `refund(request)` where supported

Each method accepts either:

- JSON string
- typed JS object (recommended)

Config payloads may include optional `http_settings`:

- `timeout_ms`
- `max_retries`
- `initial_backoff_ms`
- `max_backoff_ms`
- `user_agent`

## Typing

`index.d.ts` includes typed config/request/response contracts for all providers.

## Error Contract

Rust errors are converted to JS `Error` with JSON payload string:

- `message`
- `code`
- `hint`
