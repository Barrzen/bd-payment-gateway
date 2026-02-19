#[cfg(feature = "portwallet")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bd_payment_gateway::core::{Environment, PaymentProvider};
    use bd_payment_gateway::portwallet::{
        Config, CustomerInfo, InitiatePaymentRequest, PortwalletClient, RefundRequest,
        VerifyPaymentRequest,
    };
    use secrecy::SecretString;

    let client = PortwalletClient::new(Config {
        app_key: std::env::var("PORTWALLET_APP_KEY")?,
        app_secret: SecretString::new(std::env::var("PORTWALLET_APP_SECRET")?.into()),
        environment: Environment::Sandbox,
        http_settings: bd_payment_gateway::core::HttpSettings::default(),
    })?;

    let initiated = client
        .initiate_payment(&InitiatePaymentRequest {
            order: "order-123".to_owned(),
            amount: "100.00".to_owned(),
            currency: "BDT".to_owned(),
            redirect_url: "https://merchant.test/success".parse()?,
            ipn_url: "https://merchant.test/ipn".parse()?,
            reference: Some("customer-42".to_owned()),
            customer: CustomerInfo {
                name: "Demo User".to_owned(),
                email: "demo@example.com".to_owned(),
                phone: "01700000000".to_owned(),
                address: Some("Dhaka".to_owned()),
                city: Some("Dhaka".to_owned()),
                zip_code: Some("1207".to_owned()),
                country: Some("BD".to_owned()),
            },
            correlation_id: None,
        })
        .await?;

    println!("redirect = {}", initiated.redirect_url);

    let verified = client
        .verify_payment(&VerifyPaymentRequest {
            invoice_id: initiated.provider_reference.clone(),
            correlation_id: None,
        })
        .await?;

    println!("status = {:?}", verified.status);

    let _ = client
        .refund(&RefundRequest {
            invoice_id: initiated.provider_reference,
            amount: "10.00".to_owned(),
            reason: Some("test-refund".to_owned()),
            correlation_id: None,
        })
        .await;

    Ok(())
}

#[cfg(not(feature = "portwallet"))]
fn main() {
    eprintln!("Enable feature 'portwallet' to run this example.");
}
