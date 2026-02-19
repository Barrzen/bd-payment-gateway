use std::str::FromStr;

use async_trait::async_trait;
use bd_payment_gateway_core::{
    BdPaymentError, Currency, Environment, HttpClient, HttpSettings, InitiatePaymentResponse,
    PaymentProvider, PaymentStatus, RefundResponse, Result, VerifyPaymentResponse,
};
use reqwest::Method;
use rust_decimal::Decimal;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

const SANDBOX_BASE: &str = "https://sandbox.aamarpay.com";
const PRODUCTION_BASE: &str = "https://secure.aamarpay.com";

#[derive(Debug, Clone)]
pub struct Config {
    pub store_id: String,
    pub signature_key: SecretString,
    pub environment: Environment,
    pub http_settings: HttpSettings,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.store_id.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "aamarPay store_id is empty.",
                "Set Config.store_id from your aamarPay merchant account.",
            ));
        }

        if self.signature_key.expose_secret().trim().is_empty() {
            return Err(BdPaymentError::validation(
                "aamarPay signature_key is empty.",
                "Set Config.signature_key from your aamarPay merchant account.",
            ));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct AamarpayClient {
    config: Config,
    http: HttpClient,
    base_url: Url,
}

impl AamarpayClient {
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct InitiatePaymentRequest {
    pub tran_id: String,
    pub amount: String,
    pub currency: String,
    pub success_url: Url,
    pub fail_url: Url,
    pub cancel_url: Url,
    pub desc: Option<String>,
    pub cus_name: String,
    pub cus_email: String,
    pub cus_add1: String,
    pub cus_add2: Option<String>,
    pub cus_city: String,
    pub cus_state: Option<String>,
    pub cus_postcode: Option<String>,
    pub cus_country: String,
    pub cus_phone: String,
    pub opt_a: Option<String>,
    pub opt_b: Option<String>,
    pub opt_c: Option<String>,
    pub opt_d: Option<String>,
    pub signature_key: Option<SecretString>,
}

impl InitiatePaymentRequest {
    pub fn validate(&self) -> Result<()> {
        if self.tran_id.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "tran_id is required for aamarPay initiate.",
                "Use your unique transaction id for reconciliation.",
            ));
        }
        if Decimal::from_str(&self.amount).is_err() {
            return Err(BdPaymentError::validation(
                "amount must be numeric for aamarPay initiate.",
                "Use decimal string like '150.00'.",
            ));
        }
        if self.cus_name.trim().is_empty() || self.cus_phone.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "cus_name and cus_phone are required for aamarPay.",
                "Provide customer identity fields per aamarPay docs.",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentRequest {
    pub request_id: String,
}

#[derive(Debug, Serialize)]
struct JsonPostRequest<'a> {
    store_id: &'a str,
    signature_key: &'a str,
    tran_id: &'a str,
    amount: &'a str,
    currency: &'a str,
    success_url: &'a str,
    fail_url: &'a str,
    cancel_url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    desc: Option<&'a str>,
    cus_name: &'a str,
    cus_email: &'a str,
    cus_add1: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    cus_add2: Option<&'a str>,
    cus_city: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    cus_state: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cus_postcode: Option<&'a str>,
    cus_country: &'a str,
    cus_phone: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    opt_a: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opt_b: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opt_c: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opt_d: Option<&'a str>,
}

#[async_trait]
impl PaymentProvider for AamarpayClient {
    type InitiateRequest = InitiatePaymentRequest;
    type VerifyRequest = VerifyPaymentRequest;
    type RefundRequest = Value;

    async fn initiate_payment(
        &self,
        req: &Self::InitiateRequest,
    ) -> Result<InitiatePaymentResponse> {
        req.validate()?;

        let url = self.base_url.join("/jsonpost.php").map_err(|e| {
            BdPaymentError::config(
                format!("Invalid aamarPay JSON initiate URL: {e}"),
                "Check environment base URL configuration.",
            )
        })?;

        let body = JsonPostRequest {
            store_id: &self.config.store_id,
            signature_key: req
                .signature_key
                .as_ref()
                .unwrap_or(&self.config.signature_key)
                .expose_secret(),
            tran_id: &req.tran_id,
            amount: &req.amount,
            currency: &req.currency,
            success_url: req.success_url.as_str(),
            fail_url: req.fail_url.as_str(),
            cancel_url: req.cancel_url.as_str(),
            desc: req.desc.as_deref(),
            cus_name: &req.cus_name,
            cus_email: &req.cus_email,
            cus_add1: &req.cus_add1,
            cus_add2: req.cus_add2.as_deref(),
            cus_city: &req.cus_city,
            cus_state: req.cus_state.as_deref(),
            cus_postcode: req.cus_postcode.as_deref(),
            cus_country: &req.cus_country,
            cus_phone: &req.cus_phone,
            opt_a: req.opt_a.as_deref(),
            opt_b: req.opt_b.as_deref(),
            opt_c: req.opt_c.as_deref(),
            opt_d: req.opt_d.as_deref(),
        };

        let raw: Value = self
            .http
            .request_json(
                Method::POST,
                &url,
                reqwest::header::HeaderMap::new(),
                Some(&body),
            )
            .await?;

        if is_failure(&raw) {
            return Err(BdPaymentError::provider(
                raw.get("msg")
                    .or_else(|| raw.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("aamarPay rejected initiate request."),
                "Verify store_id/signature_key and required customer/payment fields.",
                raw.get("status_code")
                    .and_then(Value::as_i64)
                    .map(|v| v.to_string()),
                None,
            ));
        }

        let redirect_url = raw
            .get("payment_url")
            .or_else(|| raw.get("paymentUrl"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                BdPaymentError::parse(
                    "aamarPay response missing payment_url.",
                    "Confirm request payload and environment are correct.",
                )
            })
            .and_then(|v| {
                Url::parse(v).map_err(|e| {
                    BdPaymentError::parse(
                        format!("Invalid aamarPay payment_url: {e}"),
                        "Provider returned malformed URL.",
                    )
                })
            })?;

        let provider_reference = raw
            .get("request_id")
            .or_else(|| raw.get("tran_id"))
            .and_then(Value::as_str)
            .unwrap_or(&req.tran_id)
            .to_owned();

        Ok(InitiatePaymentResponse {
            redirect_url,
            provider_reference,
            raw,
            request_id: None,
        })
    }

    async fn verify_payment(&self, req: &Self::VerifyRequest) -> Result<VerifyPaymentResponse> {
        if req.request_id.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "request_id is required for aamarPay transaction check.",
                "Pass the request_id returned from initiate_payment.",
            ));
        }

        let mut url = self
            .base_url
            .join("/api/v1/trxcheck/request.php")
            .map_err(|e| {
                BdPaymentError::config(
                    format!("Invalid aamarPay verify URL: {e}"),
                    "Check environment base URL configuration.",
                )
            })?;
        url.query_pairs_mut()
            .append_pair("request_id", &req.request_id);

        let raw: Value = self
            .http
            .request_json::<(), Value>(Method::GET, &url, reqwest::header::HeaderMap::new(), None)
            .await?;

        let status = raw
            .get("pay_status")
            .or_else(|| raw.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_ascii_lowercase();

        let payment_status = if status.contains("successful") || status.contains("paid") {
            PaymentStatus::Paid
        } else if status.contains("pending") {
            PaymentStatus::Pending
        } else if status.contains("cancel") {
            PaymentStatus::Cancelled
        } else if status.contains("fail") {
            PaymentStatus::Failed
        } else {
            PaymentStatus::Unknown(status)
        };

        let amount = raw
            .get("amount")
            .and_then(Value::as_str)
            .and_then(|v| Decimal::from_str(v).ok());
        let currency = raw
            .get("currency")
            .and_then(Value::as_str)
            .map(parse_currency);

        Ok(VerifyPaymentResponse {
            status: payment_status,
            provider_reference: req.request_id.clone(),
            amount,
            currency: currency.clone(),
            money: amount
                .zip(currency)
                .map(|(amount, currency)| bd_payment_gateway_core::Money { amount, currency }),
            raw,
            request_id: None,
        })
    }

    async fn refund(&self, _req: &Self::RefundRequest) -> Result<RefundResponse> {
        Err(BdPaymentError::unsupported(
            "aamarPay refund endpoint is not published in this SDK scope.",
            "Use aamarPay merchant dashboard or add provider-specific refund API when officially documented.",
        ))
    }
}

fn parse_currency(raw: &str) -> Currency {
    match raw.to_ascii_uppercase().as_str() {
        "BDT" => Currency::Bdt,
        "USD" => Currency::Usd,
        "EUR" => Currency::Eur,
        other => Currency::Other(other.to_owned()),
    }
}

fn is_failure(raw: &Value) -> bool {
    raw.get("result")
        .and_then(Value::as_bool)
        .map(|v| !v)
        .or_else(|| {
            raw.get("result")
                .and_then(Value::as_str)
                .map(|v| v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("error"))
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::POST;
    use httpmock::MockServer;

    #[test]
    fn request_validation_requires_numeric_amount() {
        let req = InitiatePaymentRequest {
            tran_id: "TXN-1".to_owned(),
            amount: "not-a-number".to_owned(),
            currency: "BDT".to_owned(),
            success_url: Url::parse("https://example.com/success").expect("valid url"),
            fail_url: Url::parse("https://example.com/fail").expect("valid url"),
            cancel_url: Url::parse("https://example.com/cancel").expect("valid url"),
            desc: None,
            cus_name: "Nobin".to_owned(),
            cus_email: "n@example.com".to_owned(),
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
        };

        assert!(req.validate().is_err());
    }

    #[tokio::test]
    async fn initiate_payment_parses_payment_url() {
        let server = MockServer::start();
        let _init_mock = server.mock(|when, then| {
            when.method(POST).path("/jsonpost.php");
            then.status(200).json_body_obj(&serde_json::json!({
                "result": true,
                "payment_url": "https://sandbox.aamarpay.com/pay/abc",
                "request_id": "REQ-1"
            }));
        });

        let client = AamarpayClient::new(Config {
            store_id: "store".to_owned(),
            signature_key: SecretString::new("secret".to_owned().into()),
            environment: Environment::CustomBaseUrl(
                Url::parse(&server.base_url()).expect("mock server url"),
            ),
            http_settings: HttpSettings::default(),
        })
        .expect("client");

        let result = client
            .initiate_payment(&InitiatePaymentRequest {
                tran_id: "T-1".to_owned(),
                amount: "120.00".to_owned(),
                currency: "BDT".to_owned(),
                success_url: Url::parse("https://merchant.test/s").expect("url"),
                fail_url: Url::parse("https://merchant.test/f").expect("url"),
                cancel_url: Url::parse("https://merchant.test/c").expect("url"),
                desc: None,
                cus_name: "Demo".to_owned(),
                cus_email: "demo@example.com".to_owned(),
                cus_add1: "Dhaka".to_owned(),
                cus_add2: None,
                cus_city: "Dhaka".to_owned(),
                cus_state: None,
                cus_postcode: None,
                cus_country: "Bangladesh".to_owned(),
                cus_phone: "017".to_owned(),
                opt_a: None,
                opt_b: None,
                opt_c: None,
                opt_d: None,
                signature_key: None,
            })
            .await
            .expect("initiate");

        assert_eq!(result.provider_reference, "REQ-1");
        assert_eq!(
            result.redirect_url.as_str(),
            "https://sandbox.aamarpay.com/pay/abc"
        );
    }
}
