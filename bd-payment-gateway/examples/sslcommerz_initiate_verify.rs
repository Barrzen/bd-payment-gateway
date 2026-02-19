#[cfg(feature = "sslcommerz")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bd_payment_gateway::core::{Environment, PaymentProvider};
    use bd_payment_gateway::sslcommerz::{
        Config, InitiatePaymentRequest, RefundRequest, SslcommerzClient, VerifyPaymentRequest,
        VerifyReference,
    };
    use secrecy::SecretString;

    let client = SslcommerzClient::new(Config {
        store_id: std::env::var("SSLCOMMERZ_STORE_ID")?,
        store_passwd: SecretString::new(std::env::var("SSLCOMMERZ_STORE_PASSWD")?.into()),
        environment: Environment::Sandbox,
        http_settings: bd_payment_gateway::core::HttpSettings::default(),
    })?;

    let initiated = client
        .initiate_payment(&InitiatePaymentRequest {
            total_amount: "100.00".to_owned(),
            currency: "BDT".to_owned(),
            tran_id: "txn-123".to_owned(),
            success_url: "https://merchant.test/success".parse()?,
            fail_url: "https://merchant.test/fail".parse()?,
            cancel_url: "https://merchant.test/cancel".parse()?,
            ipn_url: Some("https://merchant.test/ipn".parse()?),
            shipping_method: Some("NO".to_owned()),
            product_name: "Book".to_owned(),
            product_category: "General".to_owned(),
            product_profile: "general".to_owned(),
            cus_name: "Demo User".to_owned(),
            cus_email: "demo@example.com".to_owned(),
            cus_add1: "Dhaka".to_owned(),
            cus_city: "Dhaka".to_owned(),
            cus_country: "Bangladesh".to_owned(),
            cus_phone: "01700000000".to_owned(),
            value_a: None,
            value_b: None,
            value_c: None,
            value_d: None,
        })
        .await?;

    println!("redirect = {}", initiated.redirect_url);

    let verified = client
        .verify_payment(&VerifyPaymentRequest {
            reference: VerifyReference::SessionKey(initiated.provider_reference.clone()),
        })
        .await?;
    println!("status = {:?}", verified.status);

    let _ = client
        .refund(&RefundRequest::Query {
            refund_ref_id: "RFD-123".to_owned(),
        })
        .await;

    Ok(())
}

#[cfg(not(feature = "sslcommerz"))]
fn main() {
    eprintln!("Enable feature 'sslcommerz' to run this example.");
}
