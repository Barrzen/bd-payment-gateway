use async_trait::async_trait;
use bd_payment_gateway_core::{
    http::add_default_headers, BdPaymentError, Environment, HttpClient, HttpSettings,
    InitiatePaymentResponse, PaymentProvider, PaymentStatus, RefundResponse, Result,
    VerifyPaymentResponse,
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;

const SANDBOX_BASE: &str = "https://sandbox.shurjopayment.com";
const PRODUCTION_BASE: &str = "https://engine.shurjopayment.com";

#[derive(Debug, Clone)]
pub struct Config {
    pub username: String,
    pub password: SecretString,
    pub prefix: String,
    pub environment: Environment,
    pub http_settings: HttpSettings,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.username.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "shurjoPay username is empty.",
                "Set Config.username from your shurjoPay merchant credentials.",
            ));
        }
        if self.password.expose_secret().trim().is_empty() {
            return Err(BdPaymentError::validation(
                "shurjoPay password is empty.",
                "Set Config.password from your shurjoPay merchant credentials.",
            ));
        }
        if self.prefix.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "shurjoPay prefix is empty.",
                "Set Config.prefix to your assigned merchant prefix (for example, NOK123).",
            ));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ShurjopayClient {
    config: Config,
    http: HttpClient,
    base_url: Url,
}

impl ShurjopayClient {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        let base_url = config.environment.resolve(SANDBOX_BASE, PRODUCTION_BASE)?;
        let http = HttpClient::new(config.http_settings.clone(), None)?;
        Ok(Self {
            config,
            http,
            base_url,
        })
    }

    async fn fetch_token(&self) -> Result<String> {
        let url = self.base_url.join("/api/get_token").map_err(|e| {
            BdPaymentError::config(
                format!("Invalid shurjoPay token URL: {e}"),
                "Check environment base URL configuration.",
            )
        })?;

        let req = TokenRequest {
            username: self.config.username.clone(),
            password: self.config.password.expose_secret().to_owned(),
        };

        let response: TokenResponse = self.http.post_json(&url, HeaderMap::new(), &req).await?;

        if let Some(token) = response.token {
            if token.trim().is_empty() {
                return Err(BdPaymentError::provider(
                    "shurjoPay returned an empty auth token.",
                    "Verify your shurjoPay username/password and environment (sandbox vs production).",
                    response.sp_code.map(|v| v.to_string()),
                    None,
                ));
            }
            return Ok(token);
        }

        Err(BdPaymentError::provider(
            response
                .message
                .unwrap_or_else(|| "Unable to get shurjoPay token.".to_owned()),
            "Check merchant credentials and confirm IP is allowed by shurjoPay.",
            response.sp_code.map(|v| v.to_string()),
            None,
        ))
    }

    fn base_headers(token: &str, correlation_id: Option<&str>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let auth = format!("Bearer {token}");
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth).map_err(|e| {
                BdPaymentError::validation(
                    format!("Invalid Authorization header: {e}"),
                    "Ensure token only contains valid HTTP header characters.",
                )
            })?,
        );
        add_default_headers(headers, correlation_id, None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiatePaymentRequest {
    pub amount: String,
    pub order_id: String,
    pub currency: String,
    pub return_url: Url,
    pub cancel_url: Url,
    pub client_ip: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: String,
    pub customer_address: String,
    pub customer_city: String,
    pub customer_state: String,
    pub customer_postcode: String,
    pub customer_country: String,
    pub value1: Option<String>,
    pub value2: Option<String>,
    pub value3: Option<String>,
    pub value4: Option<String>,
    pub discount_amount: Option<String>,
    pub discount_percent: Option<String>,
    pub correlation_id: Option<String>,
}

impl InitiatePaymentRequest {
    pub fn validate(&self) -> Result<()> {
        if self.amount.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "amount is required for shurjoPay.",
                "Provide a decimal amount as string, e.g. '100.00'.",
            ));
        }
        if self.order_id.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "order_id is required for shurjoPay.",
                "Use your unique order reference for reconciliation.",
            ));
        }
        if self.customer_name.trim().is_empty() || self.customer_phone.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "customer_name and customer_phone are required for shurjoPay.",
                "Provide customer identity fields as required by shurjoPay checkout.",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentRequest {
    pub sp_order_id: String,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct TokenRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    token: Option<String>,
    sp_code: Option<i64>,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct SecretPayRequest<'a> {
    prefix: &'a str,
    currency: &'a str,
    return_url: &'a str,
    cancel_url: &'a str,
    amount: &'a str,
    order_id: &'a str,
    #[serde(rename = "discsount_amount", skip_serializing_if = "Option::is_none")]
    discount_amount: Option<&'a str>,
    #[serde(rename = "disc_percent", skip_serializing_if = "Option::is_none")]
    discount_percent: Option<&'a str>,
    client_ip: &'a str,
    customer_name: &'a str,
    customer_phone: &'a str,
    customer_email: &'a str,
    customer_address: &'a str,
    customer_city: &'a str,
    customer_state: &'a str,
    customer_postcode: &'a str,
    customer_country: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    value1: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value2: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value3: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value4: Option<&'a str>,
}

#[async_trait]
impl PaymentProvider for ShurjopayClient {
    type InitiateRequest = InitiatePaymentRequest;
    type VerifyRequest = VerifyPaymentRequest;
    type RefundRequest = Value;

    async fn initiate_payment(
        &self,
        req: &Self::InitiateRequest,
    ) -> Result<InitiatePaymentResponse> {
        req.validate()?;

        let token = self.fetch_token().await?;
        let headers = Self::base_headers(&token, req.correlation_id.as_deref())?;

        let url = self.base_url.join("/api/secret-pay").map_err(|e| {
            BdPaymentError::config(
                format!("Invalid shurjoPay secret-pay URL: {e}"),
                "Check environment base URL configuration.",
            )
        })?;

        let body = SecretPayRequest {
            prefix: &self.config.prefix,
            currency: &req.currency,
            return_url: req.return_url.as_str(),
            cancel_url: req.cancel_url.as_str(),
            amount: &req.amount,
            order_id: &req.order_id,
            discount_amount: req.discount_amount.as_deref(),
            discount_percent: req.discount_percent.as_deref(),
            client_ip: &req.client_ip,
            customer_name: &req.customer_name,
            customer_phone: &req.customer_phone,
            customer_email: &req.customer_email,
            customer_address: &req.customer_address,
            customer_city: &req.customer_city,
            customer_state: &req.customer_state,
            customer_postcode: &req.customer_postcode,
            customer_country: &req.customer_country,
            value1: req.value1.as_deref(),
            value2: req.value2.as_deref(),
            value3: req.value3.as_deref(),
            value4: req.value4.as_deref(),
        };

        let raw: Value = self.http.post_json(&url, headers, &body).await?;

        let redirect_url = raw
            .get("checkout_url")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                BdPaymentError::provider(
                    "shurjoPay response is missing checkout_url.",
                    "Confirm the request payload matches secret-pay required fields.",
                    None,
                    None,
                )
            })
            .and_then(|v| {
                Url::parse(v).map_err(|e| {
                    BdPaymentError::parse(
                        format!("Invalid shurjoPay checkout_url: {e}"),
                        "Provider returned malformed checkout URL.",
                    )
                })
            })?;

        let provider_reference = raw
            .get("sp_order_id")
            .or_else(|| raw.get("order_id"))
            .and_then(Value::as_str)
            .unwrap_or(&req.order_id)
            .to_owned();

        Ok(InitiatePaymentResponse {
            redirect_url,
            provider_reference,
            raw,
            request_id: req.correlation_id.clone(),
        })
    }

    async fn verify_payment(&self, req: &Self::VerifyRequest) -> Result<VerifyPaymentResponse> {
        if req.sp_order_id.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "sp_order_id is required for shurjoPay verification.",
                "Pass the provider reference returned during initiate_payment.",
            ));
        }

        let token = self.fetch_token().await?;
        let headers = Self::base_headers(&token, req.correlation_id.as_deref())?;

        let url = self.base_url.join("/api/verification").map_err(|e| {
            BdPaymentError::config(
                format!("Invalid shurjoPay verification URL: {e}"),
                "Check environment base URL configuration.",
            )
        })?;

        let raw: Value = self
            .http
            .post_json(&url, headers, &json!({"order_id": req.sp_order_id}))
            .await?;

        let status = map_status(&raw);

        Ok(VerifyPaymentResponse {
            status,
            provider_reference: req.sp_order_id.clone(),
            amount: None,
            currency: None,
            money: None,
            raw,
            request_id: req.correlation_id.clone(),
        })
    }

    async fn refund(&self, _req: &Self::RefundRequest) -> Result<RefundResponse> {
        Err(BdPaymentError::unsupported(
            "Refund API is not standardized for shurjoPay in this SDK yet.",
            "Use shurjoPay merchant panel or extend provider crate with your verified refund contract.",
        ))
    }
}

fn map_status(raw: &Value) -> PaymentStatus {
    let status = if raw.is_array() {
        raw.get(0)
            .and_then(|v| v.get("bank_status").or_else(|| v.get("sp_code")))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    } else {
        raw.get("bank_status")
            .or_else(|| raw.get("status"))
            .or_else(|| raw.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    }
    .to_ascii_lowercase();

    if status.contains("success") || status.contains("paid") || status.contains("complete") {
        PaymentStatus::Paid
    } else if status.contains("pending") {
        PaymentStatus::Pending
    } else if status.contains("cancel") {
        PaymentStatus::Cancelled
    } else if status.contains("fail") {
        PaymentStatus::Failed
    } else {
        PaymentStatus::Unknown(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::POST;
    use httpmock::MockServer;

    #[test]
    fn validate_initiate_request() {
        let req = InitiatePaymentRequest {
            amount: "".to_owned(),
            order_id: "".to_owned(),
            currency: "BDT".to_owned(),
            return_url: Url::parse("https://example.com/ok").expect("valid url"),
            cancel_url: Url::parse("https://example.com/cancel").expect("valid url"),
            client_ip: "127.0.0.1".to_owned(),
            customer_name: "".to_owned(),
            customer_phone: "".to_owned(),
            customer_email: "a@a.com".to_owned(),
            customer_address: "addr".to_owned(),
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
        };

        assert!(req.validate().is_err());
    }

    #[tokio::test]
    async fn initiate_payment_parses_checkout_url() {
        let server = MockServer::start();
        let _token_mock = server.mock(|when, then| {
            when.method(POST).path("/api/get_token");
            then.status(200)
                .json_body_obj(&serde_json::json!({"token": "tok_123"}));
        });
        let _init_mock = server.mock(|when, then| {
            when.method(POST).path("/api/secret-pay");
            then.status(200).json_body_obj(&serde_json::json!({
                "checkout_url": "https://checkout.example/123",
                "sp_order_id": "SP-001"
            }));
        });

        let client = ShurjopayClient::new(Config {
            username: "u".to_owned(),
            password: SecretString::new("p".to_owned().into()),
            prefix: "PX".to_owned(),
            environment: Environment::CustomBaseUrl(
                Url::parse(&server.base_url()).expect("mock server url"),
            ),
            http_settings: HttpSettings::default(),
        })
        .expect("client");

        let resp = client
            .initiate_payment(&InitiatePaymentRequest {
                amount: "100.00".to_owned(),
                order_id: "O-1".to_owned(),
                currency: "BDT".to_owned(),
                return_url: Url::parse("https://merchant.test/ok").expect("url"),
                cancel_url: Url::parse("https://merchant.test/cancel").expect("url"),
                client_ip: "127.0.0.1".to_owned(),
                customer_name: "A".to_owned(),
                customer_phone: "017".to_owned(),
                customer_email: "a@b.com".to_owned(),
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
            .await
            .expect("initiate");

        assert_eq!(resp.provider_reference, "SP-001");
        assert_eq!(resp.redirect_url.as_str(), "https://checkout.example/123");
    }
}
