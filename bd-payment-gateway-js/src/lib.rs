#![cfg_attr(
    not(any(
        feature = "shurjopay",
        feature = "portwallet",
        feature = "aamarpay",
        feature = "sslcommerz"
    )),
    allow(dead_code, unused_imports)
)]

use bd_payment_gateway_core::{BdPaymentError, Environment, PaymentProvider};
use napi::{Status, bindgen_prelude::*};
use napi_derive::napi;
use secrecy::SecretString;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use url::Url;

#[derive(Deserialize)]
struct EnvInput {
    mode: String,
    custom_base_url: Option<String>,
}

#[derive(Deserialize)]
struct HttpSettingsInput {
    timeout_ms: Option<u64>,
    max_retries: Option<u32>,
    initial_backoff_ms: Option<u64>,
    max_backoff_ms: Option<u64>,
    user_agent: Option<String>,
}

#[napi(object)]
pub struct JsInitiatePaymentResponse {
    pub redirect_url: String,
    pub provider_reference: String,
    pub raw: String,
    pub request_id: Option<String>,
}

#[napi(object)]
pub struct JsVerifyPaymentResponse {
    pub status: String,
    pub provider_reference: String,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub raw: String,
    pub request_id: Option<String>,
}

#[napi(object)]
pub struct JsRefundResponse {
    pub status: String,
    pub provider_reference: String,
    pub raw: String,
    pub request_id: Option<String>,
}

fn parse_environment(raw: EnvInput) -> napi::Result<Environment> {
    parse_environment_raw(raw).map_err(|message| Error::new(Status::InvalidArg, message))
}

fn parse_environment_raw(raw: EnvInput) -> std::result::Result<Environment, String> {
    match raw.mode.to_ascii_lowercase().as_str() {
        "sandbox" => std::result::Result::Ok(Environment::Sandbox),
        "production" | "live" => std::result::Result::Ok(Environment::Production),
        "custom" => {
            let custom = raw
                .custom_base_url
                .ok_or_else(|| "custom_base_url is required when mode is custom".to_owned())?;
            let url = Url::parse(&custom)
                .map_err(|e| format!("Invalid custom_base_url for environment: {e}"))?;
            std::result::Result::Ok(Environment::CustomBaseUrl(url))
        }
        _ => std::result::Result::Err(
            "environment.mode must be one of: sandbox, production, custom".to_owned(),
        ),
    }
}

fn parse_http_settings(
    raw: Option<HttpSettingsInput>,
) -> napi::Result<bd_payment_gateway_core::HttpSettings> {
    parse_http_settings_raw(raw).map_err(|message| Error::new(Status::InvalidArg, message))
}

fn parse_http_settings_raw(
    raw: Option<HttpSettingsInput>,
) -> std::result::Result<bd_payment_gateway_core::HttpSettings, String> {
    let mut settings = bd_payment_gateway_core::HttpSettings::default();
    if let Some(raw) = raw {
        if let Some(timeout_ms) = raw.timeout_ms {
            settings.timeout = Duration::from_millis(timeout_ms);
        }
        if let Some(max_retries) = raw.max_retries {
            settings.max_retries = max_retries;
        }
        if let Some(initial_backoff_ms) = raw.initial_backoff_ms {
            settings.initial_backoff = Duration::from_millis(initial_backoff_ms);
        }
        if let Some(max_backoff_ms) = raw.max_backoff_ms {
            settings.max_backoff = Duration::from_millis(max_backoff_ms);
        }
        if let Some(user_agent) = raw.user_agent {
            if user_agent.trim().is_empty() {
                return Err("http_settings.user_agent cannot be empty".to_owned());
            }
            settings.user_agent = user_agent;
        }
    }

    if settings.initial_backoff > settings.max_backoff {
        return Err(
            "http_settings.initial_backoff_ms cannot be greater than http_settings.max_backoff_ms"
                .to_owned(),
        );
    }

    Ok(settings)
}

fn to_napi_error(err: BdPaymentError) -> napi::Error {
    let details = json!({
        "message": err.to_string(),
        "code": err.code().as_str(),
        "hint": err.hint(),
    });
    Error::new(Status::GenericFailure, details.to_string())
}

fn map_initiate_response(
    resp: bd_payment_gateway_core::InitiatePaymentResponse,
) -> JsInitiatePaymentResponse {
    JsInitiatePaymentResponse {
        redirect_url: resp.redirect_url.to_string(),
        provider_reference: resp.provider_reference,
        raw: resp.raw.to_string(),
        request_id: resp.request_id,
    }
}

fn map_verify_response(
    resp: bd_payment_gateway_core::VerifyPaymentResponse,
) -> JsVerifyPaymentResponse {
    JsVerifyPaymentResponse {
        status: match resp.status {
            bd_payment_gateway_core::PaymentStatus::Pending => "pending".to_owned(),
            bd_payment_gateway_core::PaymentStatus::Paid => "paid".to_owned(),
            bd_payment_gateway_core::PaymentStatus::Failed => "failed".to_owned(),
            bd_payment_gateway_core::PaymentStatus::Cancelled => "cancelled".to_owned(),
            bd_payment_gateway_core::PaymentStatus::Refunded => "refunded".to_owned(),
            bd_payment_gateway_core::PaymentStatus::Unknown(v) => v,
        },
        provider_reference: resp.provider_reference,
        amount: resp.amount.map(|a| a.to_string()),
        currency: resp.currency.map(|c| c.as_code().to_owned()),
        raw: resp.raw.to_string(),
        request_id: resp.request_id,
    }
}

fn map_refund_response(resp: bd_payment_gateway_core::RefundResponse) -> JsRefundResponse {
    JsRefundResponse {
        status: match resp.status {
            bd_payment_gateway_core::RefundStatus::Pending => "pending".to_owned(),
            bd_payment_gateway_core::RefundStatus::Completed => "completed".to_owned(),
            bd_payment_gateway_core::RefundStatus::Failed => "failed".to_owned(),
            bd_payment_gateway_core::RefundStatus::Unknown(v) => v,
        },
        provider_reference: resp.provider_reference,
        raw: resp.raw.to_string(),
        request_id: resp.request_id,
    }
}

#[cfg(feature = "shurjopay")]
#[derive(Deserialize)]
struct ShurjopayConfigInput {
    username: String,
    password: String,
    prefix: String,
    environment: EnvInput,
    http_settings: Option<HttpSettingsInput>,
}

#[cfg(feature = "shurjopay")]
#[napi]
pub struct ShurjopayClient {
    inner: bd_payment_gateway_shurjopay::ShurjopayClient,
}

#[cfg(feature = "shurjopay")]
#[napi]
pub fn create_shurjopay_client(config_json: String) -> napi::Result<ShurjopayClient> {
    let cfg: ShurjopayConfigInput = serde_json::from_str(&config_json).map_err(|e| {
        Error::new(
            Status::InvalidArg,
            format!("Invalid shurjoPay config JSON: {e}"),
        )
    })?;

    let config = bd_payment_gateway_shurjopay::Config {
        username: cfg.username,
        password: SecretString::new(cfg.password.into()),
        prefix: cfg.prefix,
        environment: parse_environment(cfg.environment)?,
        http_settings: parse_http_settings(cfg.http_settings)?,
    };

    let inner =
        bd_payment_gateway_shurjopay::ShurjopayClient::new(config).map_err(to_napi_error)?;
    Ok(ShurjopayClient { inner })
}

#[cfg(feature = "shurjopay")]
#[napi]
impl ShurjopayClient {
    #[napi]
    pub async fn initiate_payment(
        &self,
        request_json: String,
    ) -> napi::Result<JsInitiatePaymentResponse> {
        let request: bd_payment_gateway_shurjopay::InitiatePaymentRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid shurjoPay initiate request JSON: {e}"),
                )
            })?;

        self.inner
            .initiate_payment(&request)
            .await
            .map(map_initiate_response)
            .map_err(to_napi_error)
    }

    #[napi]
    pub async fn verify_payment(
        &self,
        request_json: String,
    ) -> napi::Result<JsVerifyPaymentResponse> {
        let request: bd_payment_gateway_shurjopay::VerifyPaymentRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid shurjoPay verify request JSON: {e}"),
                )
            })?;

        self.inner
            .verify_payment(&request)
            .await
            .map(map_verify_response)
            .map_err(to_napi_error)
    }
}

#[cfg(feature = "portwallet")]
#[derive(Deserialize)]
struct PortwalletConfigInput {
    app_key: String,
    app_secret: String,
    environment: EnvInput,
    http_settings: Option<HttpSettingsInput>,
}

#[cfg(feature = "portwallet")]
#[napi]
pub struct PortwalletClient {
    inner: bd_payment_gateway_portwallet::PortwalletClient,
}

#[cfg(feature = "portwallet")]
#[napi]
pub fn create_portwallet_client(config_json: String) -> napi::Result<PortwalletClient> {
    let cfg: PortwalletConfigInput = serde_json::from_str(&config_json).map_err(|e| {
        Error::new(
            Status::InvalidArg,
            format!("Invalid PortWallet config JSON: {e}"),
        )
    })?;

    let config = bd_payment_gateway_portwallet::Config {
        app_key: cfg.app_key,
        app_secret: SecretString::new(cfg.app_secret.into()),
        environment: parse_environment(cfg.environment)?,
        http_settings: parse_http_settings(cfg.http_settings)?,
    };

    let inner =
        bd_payment_gateway_portwallet::PortwalletClient::new(config).map_err(to_napi_error)?;
    Ok(PortwalletClient { inner })
}

#[cfg(feature = "portwallet")]
#[napi]
impl PortwalletClient {
    #[napi]
    pub async fn initiate_payment(
        &self,
        request_json: String,
    ) -> napi::Result<JsInitiatePaymentResponse> {
        let request: bd_payment_gateway_portwallet::InitiatePaymentRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid PortWallet initiate request JSON: {e}"),
                )
            })?;

        self.inner
            .initiate_payment(&request)
            .await
            .map(map_initiate_response)
            .map_err(to_napi_error)
    }

    #[napi]
    pub async fn verify_payment(
        &self,
        request_json: String,
    ) -> napi::Result<JsVerifyPaymentResponse> {
        let request: bd_payment_gateway_portwallet::VerifyPaymentRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid PortWallet verify request JSON: {e}"),
                )
            })?;

        self.inner
            .verify_payment(&request)
            .await
            .map(map_verify_response)
            .map_err(to_napi_error)
    }

    #[napi]
    pub async fn refund(&self, request_json: String) -> napi::Result<JsRefundResponse> {
        let request: bd_payment_gateway_portwallet::RefundRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid PortWallet refund request JSON: {e}"),
                )
            })?;

        self.inner
            .refund(&request)
            .await
            .map(map_refund_response)
            .map_err(to_napi_error)
    }
}

#[cfg(feature = "aamarpay")]
#[derive(Deserialize)]
struct AamarpayConfigInput {
    store_id: String,
    signature_key: String,
    environment: EnvInput,
    http_settings: Option<HttpSettingsInput>,
}

#[cfg(feature = "aamarpay")]
#[napi]
pub struct AamarpayClient {
    inner: bd_payment_gateway_aamarpay::AamarpayClient,
}

#[cfg(feature = "aamarpay")]
#[napi]
pub fn create_aamarpay_client(config_json: String) -> napi::Result<AamarpayClient> {
    let cfg: AamarpayConfigInput = serde_json::from_str(&config_json).map_err(|e| {
        Error::new(
            Status::InvalidArg,
            format!("Invalid aamarPay config JSON: {e}"),
        )
    })?;

    let config = bd_payment_gateway_aamarpay::Config {
        store_id: cfg.store_id,
        signature_key: SecretString::new(cfg.signature_key.into()),
        environment: parse_environment(cfg.environment)?,
        http_settings: parse_http_settings(cfg.http_settings)?,
    };

    let inner = bd_payment_gateway_aamarpay::AamarpayClient::new(config).map_err(to_napi_error)?;
    Ok(AamarpayClient { inner })
}

#[cfg(feature = "aamarpay")]
#[napi]
impl AamarpayClient {
    #[napi]
    pub async fn initiate_payment(
        &self,
        request_json: String,
    ) -> napi::Result<JsInitiatePaymentResponse> {
        let request: bd_payment_gateway_aamarpay::InitiatePaymentRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid aamarPay initiate request JSON: {e}"),
                )
            })?;

        self.inner
            .initiate_payment(&request)
            .await
            .map(map_initiate_response)
            .map_err(to_napi_error)
    }

    #[napi]
    pub async fn verify_payment(
        &self,
        request_json: String,
    ) -> napi::Result<JsVerifyPaymentResponse> {
        let request: bd_payment_gateway_aamarpay::VerifyPaymentRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid aamarPay verify request JSON: {e}"),
                )
            })?;

        self.inner
            .verify_payment(&request)
            .await
            .map(map_verify_response)
            .map_err(to_napi_error)
    }
}

#[cfg(feature = "sslcommerz")]
#[derive(Deserialize)]
struct SslcommerzConfigInput {
    store_id: String,
    store_passwd: String,
    environment: EnvInput,
    http_settings: Option<HttpSettingsInput>,
}

#[cfg(feature = "sslcommerz")]
#[napi]
pub struct SslcommerzClient {
    inner: bd_payment_gateway_sslcommerz::SslcommerzClient,
}

#[cfg(feature = "sslcommerz")]
#[napi]
pub fn create_sslcommerz_client(config_json: String) -> napi::Result<SslcommerzClient> {
    let cfg: SslcommerzConfigInput = serde_json::from_str(&config_json).map_err(|e| {
        Error::new(
            Status::InvalidArg,
            format!("Invalid SSLCOMMERZ config JSON: {e}"),
        )
    })?;

    let config = bd_payment_gateway_sslcommerz::Config {
        store_id: cfg.store_id,
        store_passwd: SecretString::new(cfg.store_passwd.into()),
        environment: parse_environment(cfg.environment)?,
        http_settings: parse_http_settings(cfg.http_settings)?,
    };

    let inner =
        bd_payment_gateway_sslcommerz::SslcommerzClient::new(config).map_err(to_napi_error)?;
    Ok(SslcommerzClient { inner })
}

#[cfg(test)]
mod tests {
    use super::{EnvInput, HttpSettingsInput, parse_environment_raw, parse_http_settings_raw};

    #[test]
    fn parse_environment_supports_custom_mode() {
        let env = parse_environment_raw(EnvInput {
            mode: "custom".to_owned(),
            custom_base_url: Some("https://merchant.test".to_owned()),
        })
        .expect("custom environment should parse");

        assert!(matches!(
            env,
            bd_payment_gateway_core::Environment::CustomBaseUrl(_)
        ));
    }

    #[test]
    fn parse_http_settings_overrides_defaults() {
        let settings = parse_http_settings_raw(Some(HttpSettingsInput {
            timeout_ms: Some(45_000),
            max_retries: Some(5),
            initial_backoff_ms: Some(300),
            max_backoff_ms: Some(2_500),
            user_agent: Some("bd-payment-gateway-js-test".to_owned()),
        }))
        .expect("settings should parse");

        assert_eq!(settings.timeout.as_millis(), 45_000);
        assert_eq!(settings.max_retries, 5);
        assert_eq!(settings.initial_backoff.as_millis(), 300);
        assert_eq!(settings.max_backoff.as_millis(), 2_500);
        assert_eq!(settings.user_agent, "bd-payment-gateway-js-test");
    }

    #[test]
    fn parse_http_settings_rejects_invalid_backoff_bounds() {
        let err = parse_http_settings_raw(Some(HttpSettingsInput {
            timeout_ms: None,
            max_retries: None,
            initial_backoff_ms: Some(2_000),
            max_backoff_ms: Some(500),
            user_agent: None,
        }))
        .expect_err("invalid backoff bounds should fail");

        assert!(err.contains("initial_backoff_ms"));
    }
}

#[cfg(feature = "sslcommerz")]
#[napi]
impl SslcommerzClient {
    #[napi]
    pub async fn initiate_payment(
        &self,
        request_json: String,
    ) -> napi::Result<JsInitiatePaymentResponse> {
        let request: bd_payment_gateway_sslcommerz::InitiatePaymentRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid SSLCOMMERZ initiate request JSON: {e}"),
                )
            })?;

        self.inner
            .initiate_payment(&request)
            .await
            .map(map_initiate_response)
            .map_err(to_napi_error)
    }

    #[napi]
    pub async fn verify_payment(
        &self,
        request_json: String,
    ) -> napi::Result<JsVerifyPaymentResponse> {
        let request: bd_payment_gateway_sslcommerz::VerifyPaymentRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid SSLCOMMERZ verify request JSON: {e}"),
                )
            })?;

        self.inner
            .verify_payment(&request)
            .await
            .map(map_verify_response)
            .map_err(to_napi_error)
    }

    #[napi]
    pub async fn refund(&self, request_json: String) -> napi::Result<JsRefundResponse> {
        let request: bd_payment_gateway_sslcommerz::RefundRequest =
            serde_json::from_str(&request_json).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid SSLCOMMERZ refund request JSON: {e}"),
                )
            })?;

        self.inner
            .refund(&request)
            .await
            .map(map_refund_response)
            .map_err(to_napi_error)
    }
}
