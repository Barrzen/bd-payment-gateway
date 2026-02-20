use std::str::FromStr;

use async_trait::async_trait;
use bd_payment_gateway_core::{
    BdPaymentError, Currency, Environment, HttpClient, HttpSettings, InitiatePaymentResponse,
    PaymentProvider, PaymentStatus, RefundResponse, RefundStatus, Result, VerifyPaymentResponse,
    http::add_default_headers,
};
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rust_decimal::Decimal;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use url::Url;

const SANDBOX_BASE: &str = "https://api-sandbox.portwallet.com";
const PRODUCTION_BASE: &str = "https://api.portwallet.com";

#[derive(Debug, Clone)]
pub struct Config {
    pub app_key: String,
    pub app_secret: SecretString,
    pub environment: Environment,
    pub http_settings: HttpSettings,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.app_key.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "PortWallet app_key is empty.",
                "Set Config.app_key from PortWallet merchant panel.",
            ));
        }

        if self.app_secret.expose_secret().trim().is_empty() {
            return Err(BdPaymentError::validation(
                "PortWallet app_secret is empty.",
                "Set Config.app_secret from PortWallet merchant panel.",
            ));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct PortwalletClient {
    config: Config,
    http: HttpClient,
    base_url: Url,
}

impl PortwalletClient {
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

    fn auth_headers(
        &self,
        correlation_id: Option<&str>,
        ts_override: Option<&str>,
    ) -> Result<HeaderMap> {
        let timestamp = ts_override
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| Utc::now().format("%Y%m%d%H%M%S").to_string());
        let signature = generate_signature(self.config.app_secret.expose_secret(), &timestamp);

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-app-key"),
            HeaderValue::from_str(&self.config.app_key).map_err(|e| {
                BdPaymentError::validation(
                    format!("Invalid PortWallet app key header: {e}"),
                    "Ensure app_key uses only valid HTTP header characters.",
                )
            })?,
        );
        headers.insert(
            HeaderName::from_static("x-app-signature"),
            HeaderValue::from_str(&signature).map_err(|e| {
                BdPaymentError::validation(
                    format!("Invalid PortWallet signature header: {e}"),
                    "Generated signature was not ASCII-safe; rotate key and retry.",
                )
            })?,
        );
        headers.insert(
            HeaderName::from_static("x-app-timestamp"),
            HeaderValue::from_str(&timestamp).map_err(|e| {
                BdPaymentError::validation(
                    format!("Invalid PortWallet timestamp header: {e}"),
                    "Timestamp format must be YYYYMMDDHHMMSS in UTC.",
                )
            })?,
        );

        add_default_headers(headers, correlation_id, None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerInfo {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub zip_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiatePaymentRequest {
    pub order: String,
    pub amount: String,
    pub currency: String,
    pub redirect_url: Url,
    pub ipn_url: Url,
    pub reference: Option<String>,
    pub customer: CustomerInfo,
    pub correlation_id: Option<String>,
}

impl InitiatePaymentRequest {
    pub fn validate(&self) -> Result<()> {
        if self.order.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "order is required for PortWallet invoice create.",
                "Use your unique order/invoice identifier.",
            ));
        }
        if Decimal::from_str(&self.amount).is_err() {
            return Err(BdPaymentError::validation(
                "amount must be a numeric decimal string for PortWallet.",
                "Use values like '100.00'.",
            ));
        }
        if self.customer.name.trim().is_empty() || self.customer.phone.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "customer.name and customer.phone are required for PortWallet.",
                "Provide customer identity fields as documented by PortWallet.",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentRequest {
    pub invoice_id: String,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub invoice_id: String,
    pub amount: String,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct InvoiceCreateRequest<'a> {
    order: &'a str,
    amount: &'a str,
    currency: &'a str,
    redirect_url: &'a str,
    ipn_url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<&'a str>,
    customer: &'a CustomerInfo,
}

#[async_trait]
impl PaymentProvider for PortwalletClient {
    type InitiateRequest = InitiatePaymentRequest;
    type VerifyRequest = VerifyPaymentRequest;
    type RefundRequest = RefundRequest;

    async fn initiate_payment(
        &self,
        req: &Self::InitiateRequest,
    ) -> Result<InitiatePaymentResponse> {
        req.validate()?;
        let headers = self.auth_headers(req.correlation_id.as_deref(), None)?;

        let url = self.base_url.join("/v2/invoice").map_err(|e| {
            BdPaymentError::config(
                format!("Invalid PortWallet invoice URL: {e}"),
                "Check environment base URL configuration.",
            )
        })?;

        let body = InvoiceCreateRequest {
            order: &req.order,
            amount: &req.amount,
            currency: &req.currency,
            redirect_url: req.redirect_url.as_str(),
            ipn_url: req.ipn_url.as_str(),
            reference: req.reference.as_deref(),
            customer: &req.customer,
        };

        let raw: Value = self.http.post_json(&url, headers, &body).await?;

        if is_result_failed(&raw) {
            return Err(BdPaymentError::provider(
                provider_message(&raw).unwrap_or("PortWallet rejected invoice create request."),
                "Verify auth headers (key/signature/timestamp) and request body fields.",
                provider_code(&raw),
                req.correlation_id.clone(),
            ));
        }

        let redirect = raw
            .pointer("/data/payment_url")
            .or_else(|| raw.pointer("/data/url"))
            .or_else(|| raw.get("payment_url"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                BdPaymentError::parse(
                    "PortWallet response missing payment_url.",
                    "Confirm API version v2 and invoice create payload format.",
                )
            })
            .and_then(|v| {
                Url::parse(v).map_err(|e| {
                    BdPaymentError::parse(
                        format!("Invalid PortWallet payment_url: {e}"),
                        "Provider returned malformed URL.",
                    )
                })
            })?;

        let provider_reference = raw
            .pointer("/data/invoice_id")
            .or_else(|| raw.get("invoice_id"))
            .and_then(Value::as_str)
            .unwrap_or(&req.order)
            .to_owned();

        Ok(InitiatePaymentResponse {
            redirect_url: redirect,
            provider_reference,
            raw,
            request_id: req.correlation_id.clone(),
        })
    }

    async fn verify_payment(&self, req: &Self::VerifyRequest) -> Result<VerifyPaymentResponse> {
        if req.invoice_id.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "invoice_id is required for PortWallet verify.",
                "Pass invoice_id returned from initiate_payment.",
            ));
        }

        let headers = self.auth_headers(req.correlation_id.as_deref(), None)?;
        let url = self
            .base_url
            .join(&format!("/v2/invoice/ipn/{}", req.invoice_id))
            .map_err(|e| {
                BdPaymentError::config(
                    format!("Invalid PortWallet verify URL: {e}"),
                    "Check environment base URL configuration.",
                )
            })?;

        let raw: Value = self.http.get_json(&url, headers).await?;

        if is_result_failed(&raw) {
            return Err(BdPaymentError::provider(
                provider_message(&raw).unwrap_or("PortWallet rejected invoice retrieval request."),
                "Ensure invoice_id exists and auth signature is valid.",
                provider_code(&raw),
                req.correlation_id.clone(),
            ));
        }

        let amount = raw
            .pointer("/data/amount")
            .and_then(Value::as_str)
            .and_then(|v| Decimal::from_str(v).ok());

        let currency = raw
            .pointer("/data/currency")
            .and_then(Value::as_str)
            .map(parse_currency);

        Ok(VerifyPaymentResponse {
            status: map_payment_status(&raw),
            provider_reference: req.invoice_id.clone(),
            amount,
            currency: currency.clone(),
            money: amount
                .zip(currency)
                .map(|(amount, currency)| bd_payment_gateway_core::Money { amount, currency }),
            raw,
            request_id: req.correlation_id.clone(),
        })
    }

    async fn refund(&self, req: &Self::RefundRequest) -> Result<RefundResponse> {
        if req.invoice_id.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "invoice_id is required for PortWallet refund.",
                "Pass invoice_id from invoice create/retrieve response.",
            ));
        }
        if Decimal::from_str(&req.amount).is_err() {
            return Err(BdPaymentError::validation(
                "amount must be numeric for PortWallet refund.",
                "Use decimal string like '50.00'.",
            ));
        }

        let headers = self.auth_headers(req.correlation_id.as_deref(), None)?;
        let url = self.base_url.join("/v2/invoice/refund").map_err(|e| {
            BdPaymentError::config(
                format!("Invalid PortWallet refund URL: {e}"),
                "Check environment base URL configuration.",
            )
        })?;

        let raw: Value = self
            .http
            .post_json(
                &url,
                headers,
                &serde_json::json!({
                    "invoice_id": req.invoice_id,
                    "amount": req.amount,
                    "reason": req.reason
                }),
            )
            .await?;

        if is_result_failed(&raw) {
            return Err(BdPaymentError::provider(
                provider_message(&raw).unwrap_or("PortWallet rejected refund request."),
                "Verify refund eligibility and invoice state on PortWallet.",
                provider_code(&raw),
                req.correlation_id.clone(),
            ));
        }

        let provider_reference = raw
            .pointer("/data/refund_id")
            .or_else(|| raw.pointer("/data/invoice_id"))
            .or_else(|| raw.get("refund_id"))
            .or_else(|| raw.get("invoice_id"))
            .and_then(Value::as_str)
            .unwrap_or(&req.invoice_id)
            .to_owned();

        Ok(RefundResponse {
            status: map_refund_status(&raw),
            provider_reference,
            raw,
            request_id: req.correlation_id.clone(),
        })
    }
}

fn generate_signature(secret: &str, timestamp: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(timestamp.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn parse_currency(raw: &str) -> Currency {
    match raw.to_ascii_uppercase().as_str() {
        "BDT" => Currency::Bdt,
        "USD" => Currency::Usd,
        "EUR" => Currency::Eur,
        other => Currency::Other(other.to_owned()),
    }
}

fn map_payment_status(raw: &Value) -> PaymentStatus {
    let status = raw
        .pointer("/data/status")
        .or_else(|| raw.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_ascii_lowercase();

    if status.contains("paid") || status.contains("accepted") || status.contains("success") {
        PaymentStatus::Paid
    } else if status.contains("pending") {
        PaymentStatus::Pending
    } else if status.contains("cancel") {
        PaymentStatus::Cancelled
    } else if status.contains("fail") || status.contains("decline") {
        PaymentStatus::Failed
    } else {
        PaymentStatus::Unknown(status)
    }
}

fn map_refund_status(raw: &Value) -> RefundStatus {
    let status = raw
        .pointer("/data/status")
        .or_else(|| raw.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_ascii_lowercase();

    if status.contains("success") || status.contains("complete") {
        RefundStatus::Completed
    } else if status.contains("pending") {
        RefundStatus::Pending
    } else if status.contains("fail") || status.contains("reject") {
        RefundStatus::Failed
    } else {
        RefundStatus::Unknown(status)
    }
}

fn is_result_failed(raw: &Value) -> bool {
    raw.get("result")
        .and_then(Value::as_str)
        .map(|v| v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("error"))
        .unwrap_or(false)
}

fn provider_message(raw: &Value) -> Option<&str> {
    raw.get("message")
        .or_else(|| raw.pointer("/error/message"))
        .and_then(Value::as_str)
}

fn provider_code(raw: &Value) -> Option<String> {
    raw.get("code")
        .or_else(|| raw.pointer("/error/code"))
        .and_then(|v| {
            if let Some(as_str) = v.as_str() {
                Some(as_str.to_owned())
            } else {
                v.as_i64().map(|as_i64| as_i64.to_string())
            }
        })
}

#[cfg(test)]
mod tests {
    use super::generate_signature;
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;

    #[test]
    fn signature_is_stable_for_given_secret_and_timestamp() {
        let secret = "my-secret";
        let ts = "20260219010203";
        let sig = generate_signature(secret, ts);
        assert_eq!(sig.len(), 64);
        assert_eq!(
            sig,
            "65e2221687dc97dfd292923dbdfa6796dfa05b965a73d5e3a98e393c51a92d13"
        );
    }

    #[test]
    fn different_timestamp_changes_signature() {
        let s1 = generate_signature("my-secret", "20260219010203");
        let s2 = generate_signature("my-secret", "20260219010204");
        assert_ne!(s1, s2);
    }

    #[tokio::test]
    async fn verify_payment_maps_paid_status() {
        let server = MockServer::start();
        let _verify_mock = server.mock(|when, then| {
            when.method(GET).path("/v2/invoice/ipn/INV-1");
            then.status(200).json_body_obj(&serde_json::json!({
                "result": "true",
                "data": {
                    "status": "PAID",
                    "amount": "100.00",
                    "currency": "BDT"
                }
            }));
        });

        let client = PortwalletClient::new(Config {
            app_key: "k".to_owned(),
            app_secret: SecretString::new("s".to_owned().into()),
            environment: Environment::CustomBaseUrl(
                Url::parse(&server.base_url()).expect("mock server url"),
            ),
            http_settings: HttpSettings::default(),
        })
        .expect("client");

        let result = client
            .verify_payment(&VerifyPaymentRequest {
                invoice_id: "INV-1".to_owned(),
                correlation_id: None,
            })
            .await
            .expect("verify");

        assert!(matches!(result.status, PaymentStatus::Paid));
        assert_eq!(
            result.amount.map(|v| v.to_string()).as_deref(),
            Some("100.00")
        );
    }
}
