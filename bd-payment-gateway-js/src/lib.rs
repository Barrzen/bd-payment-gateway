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
use url::Url;

#[derive(Deserialize)]
struct EnvInput {
    mode: String,
    custom_base_url: Option<String>,
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
    match raw.mode.to_ascii_lowercase().as_str() {
        "sandbox" => Ok(Environment::Sandbox),
        "production" | "live" => Ok(Environment::Production),
        "custom" => {
            let custom = raw.custom_base_url.ok_or_else(|| {
                Error::new(
                    Status::InvalidArg,
                    "custom_base_url is required when mode is custom".to_owned(),
                )
            })?;
            let url = Url::parse(&custom).map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Invalid custom_base_url for environment: {e}"),
                )
            })?;
            Ok(Environment::CustomBaseUrl(url))
        }
        _ => Err(Error::new(
            Status::InvalidArg,
            "environment.mode must be one of: sandbox, production, custom".to_owned(),
        )),
    }
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
        http_settings: bd_payment_gateway_core::HttpSettings::default(),
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
        http_settings: bd_payment_gateway_core::HttpSettings::default(),
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
        http_settings: bd_payment_gateway_core::HttpSettings::default(),
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
        http_settings: bd_payment_gateway_core::HttpSettings::default(),
    };

    let inner =
        bd_payment_gateway_sslcommerz::SslcommerzClient::new(config).map_err(to_napi_error)?;
    Ok(SslcommerzClient { inner })
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
