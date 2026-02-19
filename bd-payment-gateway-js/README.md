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

## API Shape

- `create_shurjopay_client(configJson)`
- `create_portwallet_client(configJson)`
- `create_aamarpay_client(configJson)`
- `create_sslcommerz_client(configJson)`

Each client exposes async methods:

- `initiate_payment(requestJson)`
- `verify_payment(requestJson)`
- `refund(requestJson)` (for providers with refund support)

Inputs are JSON strings that map directly to Rust request structs.

## Error Contract

Rust errors are converted to JS `Error` with JSON payload string:

- `message`
- `code`
- `hint`

## TypeScript

See `index.d.ts` for typed surface.
