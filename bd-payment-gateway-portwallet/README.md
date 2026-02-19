# bd-payment-gateway-portwallet

PortWallet API v2 provider crate for `bd-payment-gateway`.

## Example

```rust
use bd_payment_gateway_core::{Environment, PaymentProvider};
use bd_payment_gateway_portwallet::{Config, PortwalletClient};
use secrecy::SecretString;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = PortwalletClient::new(Config {
    app_key: "app_key".to_owned(),
    app_secret: SecretString::new("app_secret".to_owned().into()),
    environment: Environment::Sandbox,
    http_settings: bd_payment_gateway_core::HttpSettings::default(),
})?;
# let _ = client;
# Ok(()) }
```

See `../bd-payment-gateway/examples/portwallet_initiate_verify.rs`.
