# bd-payment-gateway-sslcommerz

SSLCOMMERZ provider crate for `bd-payment-gateway`.

## Example

```rust
use bd_payment_gateway_core::Environment;
use bd_payment_gateway_sslcommerz::{Config, SslcommerzClient};
use secrecy::SecretString;

let client = SslcommerzClient::new(Config {
    store_id: "store_id".to_owned(),
    store_passwd: SecretString::new("store_password".to_owned().into()),
    environment: Environment::Sandbox,
    http_settings: bd_payment_gateway_core::HttpSettings::default(),
})?;
# Ok::<(), bd_payment_gateway_core::BdPaymentError>(())
```

See `../bd-payment-gateway/examples/sslcommerz_initiate_verify.rs`.
