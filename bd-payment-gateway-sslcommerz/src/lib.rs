use std::str::FromStr;

use async_trait::async_trait;
use bd_payment_gateway_core::{
    BdPaymentError, Currency, Environment, HttpClient, HttpSettings, InitiatePaymentResponse,
    PaymentProvider, PaymentStatus, RefundResponse, RefundStatus, Result, VerifyPaymentResponse,
};
use reqwest::header::HeaderMap;
use rust_decimal::Decimal;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

const SANDBOX_BASE: &str = "https://sandbox.sslcommerz.com";
const PRODUCTION_BASE: &str = "https://securepay.sslcommerz.com";

#[derive(Debug, Clone)]
pub struct Config {
    pub store_id: String,
    pub store_passwd: SecretString,
    pub environment: Environment,
    pub http_settings: HttpSettings,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.store_id.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "SSLCOMMERZ store_id is empty.",
                "Set Config.store_id from SSLCOMMERZ merchant panel.",
            ));
        }

        if self.store_passwd.expose_secret().trim().is_empty() {
            return Err(BdPaymentError::validation(
                "SSLCOMMERZ store_passwd is empty.",
                "Set Config.store_passwd from SSLCOMMERZ merchant panel.",
            ));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct SslcommerzClient {
    config: Config,
    http: HttpClient,
    base_url: Url,
}

impl SslcommerzClient {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiatePaymentRequest {
    pub total_amount: String,
    pub currency: String,
    pub tran_id: String,
    pub success_url: Url,
    pub fail_url: Url,
    pub cancel_url: Url,
    pub ipn_url: Option<Url>,
    pub shipping_method: Option<String>,
    pub product_name: String,
    pub product_category: String,
    pub product_profile: String,
    pub cus_name: String,
    pub cus_email: String,
    pub cus_add1: String,
    pub cus_city: String,
    pub cus_country: String,
    pub cus_phone: String,
    pub value_a: Option<String>,
    pub value_b: Option<String>,
    pub value_c: Option<String>,
    pub value_d: Option<String>,
}

impl InitiatePaymentRequest {
    pub fn validate(&self) -> Result<()> {
        if self.tran_id.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "tran_id is required for SSLCOMMERZ.",
                "Use your unique transaction id for reconciliation.",
            ));
        }

        if Decimal::from_str(&self.total_amount).is_err() {
            return Err(BdPaymentError::validation(
                "total_amount must be numeric for SSLCOMMERZ.",
                "Use decimal string like '100.00'.",
            ));
        }

        if self.cus_name.trim().is_empty() || self.cus_phone.trim().is_empty() {
            return Err(BdPaymentError::validation(
                "cus_name and cus_phone are required for SSLCOMMERZ.",
                "Provide customer identity fields per SSLCOMMERZ docs.",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerifyReference {
    ValId(String),
    SessionKey(String),
    TranId(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentRequest {
    pub reference: VerifyReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefundRequest {
    Initiate {
        bank_tran_id: String,
        refund_amount: String,
        refund_remarks: String,
    },
    Query {
        refund_ref_id: String,
    },
}

#[derive(Debug, Serialize)]
struct InitiateForm<'a> {
    store_id: &'a str,
    store_passwd: &'a str,
    total_amount: &'a str,
    currency: &'a str,
    tran_id: &'a str,
    success_url: &'a str,
    fail_url: &'a str,
    cancel_url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    ipn_url: Option<&'a str>,
    shipping_method: &'a str,
    product_name: &'a str,
    product_category: &'a str,
    product_profile: &'a str,
    cus_name: &'a str,
    cus_email: &'a str,
    cus_add1: &'a str,
    cus_city: &'a str,
    cus_country: &'a str,
    cus_phone: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_a: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_b: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_c: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_d: Option<&'a str>,
}

#[async_trait]
impl PaymentProvider for SslcommerzClient {
    type InitiateRequest = InitiatePaymentRequest;
    type VerifyRequest = VerifyPaymentRequest;
    type RefundRequest = RefundRequest;

    async fn initiate_payment(
        &self,
        req: &Self::InitiateRequest,
    ) -> Result<InitiatePaymentResponse> {
        req.validate()?;

        let url = self.base_url.join("/gwprocess/v4/api.php").map_err(|e| {
            BdPaymentError::config(
                format!("Invalid SSLCOMMERZ initiate URL: {e}"),
                "Check environment base URL configuration.",
            )
        })?;

        let body = InitiateForm {
            store_id: &self.config.store_id,
            store_passwd: self.config.store_passwd.expose_secret(),
            total_amount: &req.total_amount,
            currency: &req.currency,
            tran_id: &req.tran_id,
            success_url: req.success_url.as_str(),
            fail_url: req.fail_url.as_str(),
            cancel_url: req.cancel_url.as_str(),
            ipn_url: req.ipn_url.as_ref().map(Url::as_str),
            shipping_method: req.shipping_method.as_deref().unwrap_or("NO"),
            product_name: &req.product_name,
            product_category: &req.product_category,
            product_profile: &req.product_profile,
            cus_name: &req.cus_name,
            cus_email: &req.cus_email,
            cus_add1: &req.cus_add1,
            cus_city: &req.cus_city,
            cus_country: &req.cus_country,
            cus_phone: &req.cus_phone,
            value_a: req.value_a.as_deref(),
            value_b: req.value_b.as_deref(),
            value_c: req.value_c.as_deref(),
            value_d: req.value_d.as_deref(),
        };

        let raw: Value = self.http.post_form(&url, HeaderMap::new(), &body).await?;

        if is_failure(&raw) {
            return Err(BdPaymentError::provider(
                raw.get("failedreason")
                    .or_else(|| raw.get("status"))
                    .and_then(Value::as_str)
                    .unwrap_or("SSLCOMMERZ initiate failed."),
                "Verify store credentials, return URLs, and transaction fields.",
                raw.get("error")
                    .or_else(|| raw.get("status"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                None,
            ));
        }

        let redirect_url = raw
            .get("GatewayPageURL")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                BdPaymentError::parse(
                    "SSLCOMMERZ response missing GatewayPageURL.",
                    "Check required request fields and merchant activation state.",
                )
            })
            .and_then(|v| {
                Url::parse(v).map_err(|e| {
                    BdPaymentError::parse(
                        format!("Invalid SSLCOMMERZ GatewayPageURL: {e}"),
                        "Provider returned malformed URL.",
                    )
                })
            })?;

        let provider_reference = raw
            .get("sessionkey")
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
        let mut url = match &req.reference {
            VerifyReference::ValId(_) => {
                self.base_url.join("/validator/api/validationserverAPI.php")
            }
            VerifyReference::SessionKey(_) | VerifyReference::TranId(_) => self
                .base_url
                .join("/validator/api/merchantTransIDvalidationAPI.php"),
        }
        .map_err(|e| {
            BdPaymentError::config(
                format!("Invalid SSLCOMMERZ verify URL: {e}"),
                "Check environment base URL configuration.",
            )
        })?;

        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("store_id", &self.config.store_id);
            qp.append_pair("store_passwd", self.config.store_passwd.expose_secret());
            qp.append_pair("v", "1");
            qp.append_pair("format", "json");
            match &req.reference {
                VerifyReference::ValId(val_id) => {
                    qp.append_pair("val_id", val_id);
                }
                VerifyReference::SessionKey(sessionkey) => {
                    qp.append_pair("sessionkey", sessionkey);
                }
                VerifyReference::TranId(tran_id) => {
                    qp.append_pair("tran_id", tran_id);
                }
            }
        }

        let raw: Value = self.http.get_json(&url, HeaderMap::new()).await?;

        let status = extract_status(&raw).unwrap_or_else(|| "unknown".to_owned());
        let payment_status = map_payment_status(&status);

        let amount = raw
            .get("amount")
            .or_else(|| raw.get("store_amount"))
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| Decimal::from_str(s).ok())
                    .or_else(|| v.as_f64().and_then(Decimal::from_f64_retain))
            });
        let currency = raw
            .get("currency")
            .and_then(Value::as_str)
            .map(parse_currency);

        let provider_reference = match &req.reference {
            VerifyReference::ValId(v) => v.clone(),
            VerifyReference::SessionKey(v) => v.clone(),
            VerifyReference::TranId(v) => v.clone(),
        };

        Ok(VerifyPaymentResponse {
            status: payment_status,
            provider_reference,
            amount,
            currency: currency.clone(),
            money: amount
                .zip(currency)
                .map(|(amount, currency)| bd_payment_gateway_core::Money { amount, currency }),
            raw,
            request_id: None,
        })
    }

    async fn refund(&self, req: &Self::RefundRequest) -> Result<RefundResponse> {
        let mut url = self
            .base_url
            .join("/validator/api/merchantTransIDvalidationAPI.php")
            .map_err(|e| {
                BdPaymentError::config(
                    format!("Invalid SSLCOMMERZ refund URL: {e}"),
                    "Check environment base URL configuration.",
                )
            })?;

        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("store_id", &self.config.store_id);
            qp.append_pair("store_passwd", self.config.store_passwd.expose_secret());
            qp.append_pair("v", "1");
            qp.append_pair("format", "json");
            match req {
                RefundRequest::Initiate {
                    bank_tran_id,
                    refund_amount,
                    refund_remarks,
                } => {
                    if Decimal::from_str(refund_amount).is_err() {
                        return Err(BdPaymentError::validation(
                            "refund_amount must be numeric for SSLCOMMERZ.",
                            "Use decimal string like '25.00'.",
                        ));
                    }
                    qp.append_pair("bank_tran_id", bank_tran_id);
                    qp.append_pair("refund_amount", refund_amount);
                    qp.append_pair("refund_remarks", refund_remarks);
                }
                RefundRequest::Query { refund_ref_id } => {
                    qp.append_pair("refund_ref_id", refund_ref_id);
                }
            }
        }

        let raw: Value = self.http.get_json(&url, HeaderMap::new()).await?;

        let status = extract_status(&raw).unwrap_or_else(|| "unknown".to_owned());
        let refund_status = if status.contains("success") || status.contains("done") {
            RefundStatus::Completed
        } else if status.contains("pending") {
            RefundStatus::Pending
        } else if status.contains("fail") || status.contains("invalid") {
            RefundStatus::Failed
        } else {
            RefundStatus::Unknown(status)
        };

        let provider_reference = raw
            .get("refund_ref_id")
            .or_else(|| raw.get("bank_tran_id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| match req {
                RefundRequest::Initiate { bank_tran_id, .. } => bank_tran_id.clone(),
                RefundRequest::Query { refund_ref_id } => refund_ref_id.clone(),
            });

        Ok(RefundResponse {
            status: refund_status,
            provider_reference,
            raw,
            request_id: None,
        })
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

fn map_payment_status(status: &str) -> PaymentStatus {
    if status.contains("valid") || status.contains("success") || status.contains("paid") {
        PaymentStatus::Paid
    } else if status.contains("pending") {
        PaymentStatus::Pending
    } else if status.contains("cancel") {
        PaymentStatus::Cancelled
    } else if status.contains("fail") || status.contains("invalid") {
        PaymentStatus::Failed
    } else {
        PaymentStatus::Unknown(status.to_owned())
    }
}

fn extract_status(raw: &Value) -> Option<String> {
    raw.get("status")
        .or_else(|| raw.get("APIConnect"))
        .or_else(|| raw.get("APIConnectStatus"))
        .and_then(Value::as_str)
        .map(|s| s.to_ascii_lowercase())
}

fn is_failure(raw: &Value) -> bool {
    raw.get("status")
        .and_then(Value::as_str)
        .map(|s| {
            s.eq_ignore_ascii_case("FAILED")
                || s.eq_ignore_ascii_case("INVALID_REQUEST")
                || s.eq_ignore_ascii_case("FAILED_VALIDATION")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::POST;
    use httpmock::MockServer;

    #[test]
    fn maps_valid_status_to_paid() {
        assert!(matches!(map_payment_status("valid"), PaymentStatus::Paid));
    }

    #[tokio::test]
    async fn initiate_payment_parses_gateway_url() {
        let server = MockServer::start();
        let _init_mock = server.mock(|when, then| {
            when.method(POST).path("/gwprocess/v4/api.php");
            then.status(200).json_body_obj(&serde_json::json!({
                "status": "SUCCESS",
                "GatewayPageURL": "https://sandbox.sslcommerz.com/gw/abc",
                "sessionkey": "SSN-1"
            }));
        });

        let client = SslcommerzClient::new(Config {
            store_id: "store".to_owned(),
            store_passwd: SecretString::new("pass".to_owned().into()),
            environment: Environment::CustomBaseUrl(
                Url::parse(&server.base_url()).expect("mock server url"),
            ),
            http_settings: HttpSettings::default(),
        })
        .expect("client");

        let result = client
            .initiate_payment(&InitiatePaymentRequest {
                total_amount: "99.00".to_owned(),
                currency: "BDT".to_owned(),
                tran_id: "TXN-1".to_owned(),
                success_url: Url::parse("https://merchant.test/s").expect("url"),
                fail_url: Url::parse("https://merchant.test/f").expect("url"),
                cancel_url: Url::parse("https://merchant.test/c").expect("url"),
                ipn_url: None,
                shipping_method: Some("NO".to_owned()),
                product_name: "Book".to_owned(),
                product_category: "General".to_owned(),
                product_profile: "general".to_owned(),
                cus_name: "Demo".to_owned(),
                cus_email: "demo@example.com".to_owned(),
                cus_add1: "Dhaka".to_owned(),
                cus_city: "Dhaka".to_owned(),
                cus_country: "Bangladesh".to_owned(),
                cus_phone: "017".to_owned(),
                value_a: None,
                value_b: None,
                value_c: None,
                value_d: None,
            })
            .await
            .expect("initiate");

        assert_eq!(result.provider_reference, "SSN-1");
        assert_eq!(
            result.redirect_url.as_str(),
            "https://sandbox.sslcommerz.com/gw/abc"
        );
    }
}
