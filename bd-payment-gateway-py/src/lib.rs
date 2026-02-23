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
use once_cell::sync::Lazy;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule};
use secrecy::SecretString;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::runtime::Runtime;
use url::Url;

static RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("tokio runtime should initialize for Python binding"));

pyo3::create_exception!(
    _bd_payment_gateway_py,
    PaymentGatewayError,
    pyo3::exceptions::PyException
);

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

#[pyclass]
struct InitiatePaymentResponse {
    #[pyo3(get)]
    redirect_url: String,
    #[pyo3(get)]
    provider_reference: String,
    #[pyo3(get)]
    raw: String,
    #[pyo3(get)]
    request_id: Option<String>,
}

#[pyclass]
struct VerifyPaymentResponse {
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    provider_reference: String,
    #[pyo3(get)]
    amount: Option<String>,
    #[pyo3(get)]
    currency: Option<String>,
    #[pyo3(get)]
    raw: String,
    #[pyo3(get)]
    request_id: Option<String>,
}

#[pyclass]
struct RefundResponse {
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    provider_reference: String,
    #[pyo3(get)]
    raw: String,
    #[pyo3(get)]
    request_id: Option<String>,
}

fn py_input_to_json(input: &Bound<'_, PyAny>, what: &str) -> PyResult<String> {
    if let Ok(raw) = input.extract::<String>() {
        return Ok(raw);
    }

    let json_mod = PyModule::import(input.py(), "json").map_err(|e| {
        PyValueError::new_err(format!("Failed to import json module for {what}: {e}"))
    })?;

    json_mod
        .call_method1("dumps", (input,))
        .and_then(|v| v.extract::<String>())
        .map_err(|e| {
            PyValueError::new_err(format!(
                "{what} must be a JSON string or a JSON-serializable mapping/object: {e}"
            ))
        })
}

fn parse_json_input<T: DeserializeOwned>(input: &Bound<'_, PyAny>, what: &str) -> PyResult<T> {
    let raw = py_input_to_json(input, what)?;
    serde_json::from_str(&raw).map_err(|e| PyValueError::new_err(format!("Invalid {what}: {e}")))
}

fn parse_environment(raw: EnvInput) -> PyResult<Environment> {
    parse_environment_raw(raw).map_err(PyValueError::new_err)
}

fn parse_environment_raw(raw: EnvInput) -> std::result::Result<Environment, String> {
    match raw.mode.to_ascii_lowercase().as_str() {
        "sandbox" => std::result::Result::Ok(Environment::Sandbox),
        "production" | "live" => std::result::Result::Ok(Environment::Production),
        "custom" => {
            let custom = raw
                .custom_base_url
                .ok_or_else(|| "custom_base_url is required when mode is custom".to_owned())?;
            let url = Url::parse(&custom).map_err(|e| format!("Invalid custom_base_url: {e}"))?;
            std::result::Result::Ok(Environment::CustomBaseUrl(url))
        }
        _ => std::result::Result::Err(
            "environment.mode must be one of: sandbox, production, custom".to_owned(),
        ),
    }
}

fn parse_http_settings(
    raw: Option<HttpSettingsInput>,
) -> PyResult<bd_payment_gateway_core::HttpSettings> {
    parse_http_settings_raw(raw).map_err(PyValueError::new_err)
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

fn to_py_err(err: BdPaymentError) -> PyErr {
    let (code, message, hint, provider_payload) = match &err {
        BdPaymentError::ConfigError {
            code,
            message,
            hint,
        }
        | BdPaymentError::ValidationError {
            code,
            message,
            hint,
        }
        | BdPaymentError::Unsupported {
            code,
            message,
            hint,
        }
        | BdPaymentError::ParseError {
            code,
            message,
            hint,
        } => (
            code.as_str().to_owned(),
            message.clone(),
            hint.clone(),
            None::<Value>,
        ),
        BdPaymentError::HttpError {
            code,
            message,
            hint,
            status,
            request_id,
            body,
        } => (
            code.as_str().to_owned(),
            message.clone(),
            hint.clone(),
            Some(json!({
                "status": status,
                "request_id": request_id,
                "body": body,
            })),
        ),
        BdPaymentError::ProviderError {
            code,
            message,
            hint,
            provider_code,
            request_id,
        } => (
            code.as_str().to_owned(),
            message.clone(),
            hint.clone(),
            Some(json!({
                "provider_code": provider_code,
                "request_id": request_id,
            })),
        ),
    };

    let fallback_payload = json!({
        "message": message,
        "code": code,
        "hint": hint,
        "provider_payload": provider_payload,
    });

    Python::try_attach(|py| {
        let py_err = PaymentGatewayError::new_err(message.clone());
        let err_value = py_err.value(py);

        let _ = err_value.setattr("code", code.clone());
        let _ = err_value.setattr("message", message.clone());
        let _ = err_value.setattr("hint", hint.clone());

        if let Some(payload) = provider_payload {
            let json_mod = PyModule::import(py, "json");
            let payload_value = json_mod
                .and_then(|mod_json| mod_json.call_method1("loads", (payload.to_string(),)))
                .map(|obj| obj.unbind());

            match payload_value {
                Ok(parsed_payload) => {
                    let _ = err_value.setattr("provider_payload", parsed_payload);
                }
                Err(_) => {
                    let _ = err_value.setattr("provider_payload", payload.to_string());
                }
            }
        } else {
            let _ = err_value.setattr("provider_payload", py.None());
        }

        if err_value.getattr("code").is_err() || err_value.getattr("hint").is_err() {
            PaymentGatewayError::new_err(fallback_payload.to_string())
        } else {
            py_err
        }
    })
    .unwrap_or_else(|| PaymentGatewayError::new_err(fallback_payload.to_string()))
}

fn map_initiate_response(
    resp: bd_payment_gateway_core::InitiatePaymentResponse,
) -> InitiatePaymentResponse {
    InitiatePaymentResponse {
        redirect_url: resp.redirect_url.to_string(),
        provider_reference: resp.provider_reference,
        raw: resp.raw.to_string(),
        request_id: resp.request_id,
    }
}

fn map_verify_response(
    resp: bd_payment_gateway_core::VerifyPaymentResponse,
) -> VerifyPaymentResponse {
    VerifyPaymentResponse {
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

fn map_refund_response(resp: bd_payment_gateway_core::RefundResponse) -> RefundResponse {
    RefundResponse {
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
#[pyclass]
struct ShurjopayClient {
    inner: bd_payment_gateway_shurjopay::ShurjopayClient,
}

#[cfg(feature = "shurjopay")]
#[pymethods]
impl ShurjopayClient {
    #[new]
    fn new(config: &Bound<'_, PyAny>) -> PyResult<Self> {
        let cfg: ShurjopayConfigInput = parse_json_input(config, "shurjoPay config")?;
        let config = bd_payment_gateway_shurjopay::Config {
            username: cfg.username,
            password: SecretString::new(cfg.password.into()),
            prefix: cfg.prefix,
            environment: parse_environment(cfg.environment)?,
            http_settings: parse_http_settings(cfg.http_settings)?,
        };
        let inner =
            bd_payment_gateway_shurjopay::ShurjopayClient::new(config).map_err(to_py_err)?;
        Ok(Self { inner })
    }

    fn initiate_payment(&self, request: &Bound<'_, PyAny>) -> PyResult<InitiatePaymentResponse> {
        let request: bd_payment_gateway_shurjopay::InitiatePaymentRequest =
            parse_json_input(request, "shurjoPay initiate request")?;

        let resp = RUNTIME
            .block_on(self.inner.initiate_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_initiate_response(resp))
    }

    fn verify_payment(&self, request: &Bound<'_, PyAny>) -> PyResult<VerifyPaymentResponse> {
        let request: bd_payment_gateway_shurjopay::VerifyPaymentRequest =
            parse_json_input(request, "shurjoPay verify request")?;

        let resp = RUNTIME
            .block_on(self.inner.verify_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_verify_response(resp))
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
#[pyclass]
struct PortwalletClient {
    inner: bd_payment_gateway_portwallet::PortwalletClient,
}

#[cfg(feature = "portwallet")]
#[pymethods]
impl PortwalletClient {
    #[new]
    fn new(config: &Bound<'_, PyAny>) -> PyResult<Self> {
        let cfg: PortwalletConfigInput = parse_json_input(config, "PortWallet config")?;
        let config = bd_payment_gateway_portwallet::Config {
            app_key: cfg.app_key,
            app_secret: SecretString::new(cfg.app_secret.into()),
            environment: parse_environment(cfg.environment)?,
            http_settings: parse_http_settings(cfg.http_settings)?,
        };
        let inner =
            bd_payment_gateway_portwallet::PortwalletClient::new(config).map_err(to_py_err)?;
        Ok(Self { inner })
    }

    fn initiate_payment(&self, request: &Bound<'_, PyAny>) -> PyResult<InitiatePaymentResponse> {
        let request: bd_payment_gateway_portwallet::InitiatePaymentRequest =
            parse_json_input(request, "PortWallet initiate request")?;
        let resp = RUNTIME
            .block_on(self.inner.initiate_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_initiate_response(resp))
    }

    fn verify_payment(&self, request: &Bound<'_, PyAny>) -> PyResult<VerifyPaymentResponse> {
        let request: bd_payment_gateway_portwallet::VerifyPaymentRequest =
            parse_json_input(request, "PortWallet verify request")?;
        let resp = RUNTIME
            .block_on(self.inner.verify_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_verify_response(resp))
    }

    fn refund(&self, request: &Bound<'_, PyAny>) -> PyResult<RefundResponse> {
        let request: bd_payment_gateway_portwallet::RefundRequest =
            parse_json_input(request, "PortWallet refund request")?;
        let resp = RUNTIME
            .block_on(self.inner.refund(&request))
            .map_err(to_py_err)?;
        Ok(map_refund_response(resp))
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
#[pyclass]
struct AamarpayClient {
    inner: bd_payment_gateway_aamarpay::AamarpayClient,
}

#[cfg(feature = "aamarpay")]
#[pymethods]
impl AamarpayClient {
    #[new]
    fn new(config: &Bound<'_, PyAny>) -> PyResult<Self> {
        let cfg: AamarpayConfigInput = parse_json_input(config, "aamarPay config")?;
        let config = bd_payment_gateway_aamarpay::Config {
            store_id: cfg.store_id,
            signature_key: SecretString::new(cfg.signature_key.into()),
            environment: parse_environment(cfg.environment)?,
            http_settings: parse_http_settings(cfg.http_settings)?,
        };
        let inner = bd_payment_gateway_aamarpay::AamarpayClient::new(config).map_err(to_py_err)?;
        Ok(Self { inner })
    }

    fn initiate_payment(&self, request: &Bound<'_, PyAny>) -> PyResult<InitiatePaymentResponse> {
        let request: bd_payment_gateway_aamarpay::InitiatePaymentRequest =
            parse_json_input(request, "aamarPay initiate request")?;
        let resp = RUNTIME
            .block_on(self.inner.initiate_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_initiate_response(resp))
    }

    fn verify_payment(&self, request: &Bound<'_, PyAny>) -> PyResult<VerifyPaymentResponse> {
        let request: bd_payment_gateway_aamarpay::VerifyPaymentRequest =
            parse_json_input(request, "aamarPay verify request")?;
        let resp = RUNTIME
            .block_on(self.inner.verify_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_verify_response(resp))
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
#[pyclass]
struct SslcommerzClient {
    inner: bd_payment_gateway_sslcommerz::SslcommerzClient,
}

#[cfg(feature = "sslcommerz")]
#[pymethods]
impl SslcommerzClient {
    #[new]
    fn new(config: &Bound<'_, PyAny>) -> PyResult<Self> {
        let cfg: SslcommerzConfigInput = parse_json_input(config, "SSLCOMMERZ config")?;
        let config = bd_payment_gateway_sslcommerz::Config {
            store_id: cfg.store_id,
            store_passwd: SecretString::new(cfg.store_passwd.into()),
            environment: parse_environment(cfg.environment)?,
            http_settings: parse_http_settings(cfg.http_settings)?,
        };
        let inner =
            bd_payment_gateway_sslcommerz::SslcommerzClient::new(config).map_err(to_py_err)?;
        Ok(Self { inner })
    }

    fn initiate_payment(&self, request: &Bound<'_, PyAny>) -> PyResult<InitiatePaymentResponse> {
        let request: bd_payment_gateway_sslcommerz::InitiatePaymentRequest =
            parse_json_input(request, "SSLCOMMERZ initiate request")?;
        let resp = RUNTIME
            .block_on(self.inner.initiate_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_initiate_response(resp))
    }

    fn verify_payment(&self, request: &Bound<'_, PyAny>) -> PyResult<VerifyPaymentResponse> {
        let request: bd_payment_gateway_sslcommerz::VerifyPaymentRequest =
            parse_json_input(request, "SSLCOMMERZ verify request")?;
        let resp = RUNTIME
            .block_on(self.inner.verify_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_verify_response(resp))
    }

    fn refund(&self, request: &Bound<'_, PyAny>) -> PyResult<RefundResponse> {
        let request: bd_payment_gateway_sslcommerz::RefundRequest =
            parse_json_input(request, "SSLCOMMERZ refund request")?;
        let resp = RUNTIME
            .block_on(self.inner.refund(&request))
            .map_err(to_py_err)?;
        Ok(map_refund_response(resp))
    }
}

#[pymodule]
fn _bd_payment_gateway_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add(
        "PaymentGatewayError",
        m.py().get_type::<PaymentGatewayError>(),
    )?;
    m.add_class::<InitiatePaymentResponse>()?;
    m.add_class::<VerifyPaymentResponse>()?;
    m.add_class::<RefundResponse>()?;

    #[cfg(feature = "shurjopay")]
    m.add_class::<ShurjopayClient>()?;
    #[cfg(feature = "portwallet")]
    m.add_class::<PortwalletClient>()?;
    #[cfg(feature = "aamarpay")]
    m.add_class::<AamarpayClient>()?;
    #[cfg(feature = "sslcommerz")]
    m.add_class::<SslcommerzClient>()?;

    Ok(())
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
            timeout_ms: Some(40_000),
            max_retries: Some(4),
            initial_backoff_ms: Some(250),
            max_backoff_ms: Some(2_000),
            user_agent: Some("bd-payment-gateway-py-test".to_owned()),
        }))
        .expect("settings should parse");

        assert_eq!(settings.timeout.as_millis(), 40_000);
        assert_eq!(settings.max_retries, 4);
        assert_eq!(settings.initial_backoff.as_millis(), 250);
        assert_eq!(settings.max_backoff.as_millis(), 2_000);
        assert_eq!(settings.user_agent, "bd-payment-gateway-py-test");
    }

    #[test]
    fn parse_http_settings_rejects_invalid_backoff_bounds() {
        let err = parse_http_settings_raw(Some(HttpSettingsInput {
            timeout_ms: None,
            max_retries: None,
            initial_backoff_ms: Some(1_000),
            max_backoff_ms: Some(100),
            user_agent: None,
        }))
        .expect_err("invalid backoff bounds should fail");

        assert!(err.contains("initial_backoff_ms"));
    }
}
