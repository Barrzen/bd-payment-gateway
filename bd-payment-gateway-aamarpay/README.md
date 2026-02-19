# bd-payment-gateway-aamarpay

aamarPay provider crate for `bd-payment-gateway`.

## Example

```rust
use bd_payment_gateway_aamarpay::{AamarpayClient, Config};
use bd_payment_gateway_core::Environment;
use secrecy::SecretString;

let client = AamarpayClient::new(Config {
    store_id: "store_id".to_owned(),
    signature_key: SecretString::new("signature_key".to_owned().into()),
    environment: Environment::Sandbox,
    http_settings: bd_payment_gateway_core::HttpSettings::default(),
})?;
# Ok::<(), bd_payment_gateway_core::BdPaymentError>(())
```

See `../bd-payment-gateway/examples/aamarpay_initiate_verify.rs`.
