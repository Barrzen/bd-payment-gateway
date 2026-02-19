#[cfg(feature = "aamarpay")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bd_payment_gateway::aamarpay::{
        AamarpayClient, Config, InitiatePaymentRequest, VerifyPaymentRequest,
    };
    use bd_payment_gateway::core::{Environment, PaymentProvider};
    use secrecy::SecretString;

    let client = AamarpayClient::new(Config {
        store_id: std::env::var("AAMARPAY_STORE_ID")?,
        signature_key: SecretString::new(std::env::var("AAMARPAY_SIGNATURE_KEY")?.into()),
        environment: Environment::Sandbox,
        http_settings: bd_payment_gateway::core::HttpSettings::default(),
    })?;

    let initiated = client
        .initiate_payment(&InitiatePaymentRequest {
            tran_id: "txn-123".to_owned(),
            amount: "100.00".to_owned(),
            currency: "BDT".to_owned(),
            success_url: "https://merchant.test/success".parse()?,
            fail_url: "https://merchant.test/fail".parse()?,
            cancel_url: "https://merchant.test/cancel".parse()?,
            desc: Some("Checkout for order 123".to_owned()),
            cus_name: "Demo User".to_owned(),
            cus_email: "demo@example.com".to_owned(),
            cus_add1: "Dhaka".to_owned(),
            cus_add2: None,
            cus_city: "Dhaka".to_owned(),
            cus_state: None,
            cus_postcode: None,
            cus_country: "Bangladesh".to_owned(),
            cus_phone: "01700000000".to_owned(),
            opt_a: None,
            opt_b: None,
            opt_c: None,
            opt_d: None,
            signature_key: None,
        })
        .await?;

    println!("redirect = {}", initiated.redirect_url);

    let verified = client
        .verify_payment(&VerifyPaymentRequest {
            request_id: initiated.provider_reference,
        })
        .await?;

    println!("status = {:?}", verified.status);
    Ok(())
}

#[cfg(not(feature = "aamarpay"))]
fn main() {
    eprintln!("Enable feature 'aamarpay' to run this example.");
}
