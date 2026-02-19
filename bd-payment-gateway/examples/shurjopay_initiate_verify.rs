#[cfg(feature = "shurjopay")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bd_payment_gateway::core::{Environment, PaymentProvider};
    use bd_payment_gateway::shurjopay::{
        Config, InitiatePaymentRequest, ShurjopayClient, VerifyPaymentRequest,
    };
    use secrecy::SecretString;

    let client = ShurjopayClient::new(Config {
        username: std::env::var("SHURJOPAY_USERNAME")?,
        password: SecretString::new(std::env::var("SHURJOPAY_PASSWORD")?.into()),
        prefix: std::env::var("SHURJOPAY_PREFIX")?,
        environment: Environment::Sandbox,
        http_settings: bd_payment_gateway::core::HttpSettings::default(),
    })?;

    let initiated = client
        .initiate_payment(&InitiatePaymentRequest {
            amount: "100.00".to_owned(),
            order_id: "order-123".to_owned(),
            currency: "BDT".to_owned(),
            return_url: "https://merchant.test/success".parse()?,
            cancel_url: "https://merchant.test/cancel".parse()?,
            client_ip: "127.0.0.1".to_owned(),
            customer_name: "Demo User".to_owned(),
            customer_phone: "01700000000".to_owned(),
            customer_email: "demo@example.com".to_owned(),
            customer_address: "Dhaka".to_owned(),
            customer_city: "Dhaka".to_owned(),
            customer_state: "Dhaka".to_owned(),
            customer_postcode: "1207".to_owned(),
            customer_country: "Bangladesh".to_owned(),
            value1: None,
            value2: None,
            value3: None,
            value4: None,
            discount_amount: None,
            discount_percent: None,
            correlation_id: None,
        })
        .await?;

    println!("redirect = {}", initiated.redirect_url);

    let verified = client
        .verify_payment(&VerifyPaymentRequest {
            sp_order_id: initiated.provider_reference,
            correlation_id: None,
        })
        .await?;

    println!("status = {:?}", verified.status);
    Ok(())
}

#[cfg(not(feature = "shurjopay"))]
fn main() {
    eprintln!("Enable feature 'shurjopay' to run this example.");
}
