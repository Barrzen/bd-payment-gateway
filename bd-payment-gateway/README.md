# bd-payment-gateway

Feature-gated facade crate.

## Features

- `shurjopay`
- `portwallet`
- `aamarpay`
- `sslcommerz`
- `all-providers`

Default features are empty.

## Example

```toml
[dependencies]
bd-payment-gateway = { version = "0.1", default-features = false, features = ["sslcommerz"] }
```
