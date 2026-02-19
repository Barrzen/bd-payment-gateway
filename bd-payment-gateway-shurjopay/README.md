# bd-payment-gateway-shurjopay

shurjoPay provider crate for `bd-payment-gateway`.

## Example

```rust
use bd_payment_gateway_core::{Environment, PaymentProvider};
use bd_payment_gateway_shurjopay::{Config, InitiatePaymentRequest, ShurjopayClient};
use secrecy::SecretString;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = ShurjopayClient::new(Config {
    username: "merchant_user".to_owned(),
    password: SecretString::new("merchant_pass".to_owned().into()),
    prefix: "NOK123".to_owned(),
    environment: Environment::Sandbox,
    http_settings: bd_payment_gateway_core::HttpSettings::default(),
})?;
# let _ = client;
# Ok(()) }
```

See `../bd-payment-gateway/examples/shurjopay_initiate_verify.rs`.
