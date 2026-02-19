# Provider Template

Use this template when adding `bd-payment-gateway-<provider>`.

## Required Files

- `Cargo.toml`
- `src/lib.rs`
- `README.md`
- `tests/` (unit tests at minimum)

## Required Types

- `Config`
  - credentials as `SecretString`
  - `environment: Environment`
  - `http_settings: HttpSettings`
- `<Provider>Client`
  - constructed via `new(config)`
  - internally uses core `HttpClient`
- Typed request/response structs for initiate/verify/refund

## Required Behavior

1. Validate user input before network calls.
2. Use `Environment::resolve` for base URL selection.
3. Implement `PaymentProvider` trait methods.
4. Preserve `raw` provider response in all outputs.
5. Map provider failures to `BdPaymentError::ProviderError` with hint and code.
6. Return `Unsupported` for unavailable refund APIs.

## Required Tests

- validation failures
- auth/signature/token logic (if provider requires)
- success response parsing
- provider error mapping

## Required Integrations

- Add facade feature + re-export
- Add JS wrapper class/constructor
- Add Python wrapper class
- Add root + provider README examples
