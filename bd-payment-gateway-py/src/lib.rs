use bd_payment_gateway_core::{BdPaymentError, Environment, PaymentProvider};
use once_cell::sync::Lazy;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use secrecy::SecretString;
use serde::Deserialize;
use serde_json::json;
use tokio::runtime::Runtime;
use url::Url;

static RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("tokio runtime should initialize for Python binding"));

pyo3::create_exception!(
    bd_payment_gateway_py,
    PaymentGatewayError,
    pyo3::exceptions::PyException
);

#[derive(Deserialize)]
struct EnvInput {
    mode: String,
    custom_base_url: Option<String>,
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

fn parse_environment(raw: EnvInput) -> PyResult<Environment> {
    match raw.mode.to_ascii_lowercase().as_str() {
        "sandbox" => Ok(Environment::Sandbox),
        "production" | "live" => Ok(Environment::Production),
        "custom" => {
            let custom = raw.custom_base_url.ok_or_else(|| {
                PyValueError::new_err("custom_base_url is required when mode is custom")
            })?;
            let url = Url::parse(&custom)
                .map_err(|e| PyValueError::new_err(format!("Invalid custom_base_url: {e}")))?;
            Ok(Environment::CustomBaseUrl(url))
        }
        _ => Err(PyValueError::new_err(
            "environment.mode must be one of: sandbox, production, custom",
        )),
    }
}

fn to_py_err(err: BdPaymentError) -> PyErr {
    let payload = json!({
        "message": err.to_string(),
        "code": err.code().as_str(),
        "hint": err.hint(),
    });
    PaymentGatewayError::new_err(payload.to_string())
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
    fn new(config_json: &str) -> PyResult<Self> {
        let cfg: ShurjopayConfigInput =
            serde_json::from_str(config_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let config = bd_payment_gateway_shurjopay::Config {
            username: cfg.username,
            password: SecretString::new(cfg.password.into()),
            prefix: cfg.prefix,
            environment: parse_environment(cfg.environment)?,
            http_settings: bd_payment_gateway_core::HttpSettings::default(),
        };
        let inner =
            bd_payment_gateway_shurjopay::ShurjopayClient::new(config).map_err(to_py_err)?;
        Ok(Self { inner })
    }

    fn initiate_payment(&self, request_json: &str) -> PyResult<InitiatePaymentResponse> {
        let request: bd_payment_gateway_shurjopay::InitiatePaymentRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;

        let resp = RUNTIME
            .block_on(self.inner.initiate_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_initiate_response(resp))
    }

    fn verify_payment(&self, request_json: &str) -> PyResult<VerifyPaymentResponse> {
        let request: bd_payment_gateway_shurjopay::VerifyPaymentRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;

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
    fn new(config_json: &str) -> PyResult<Self> {
        let cfg: PortwalletConfigInput =
            serde_json::from_str(config_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let config = bd_payment_gateway_portwallet::Config {
            app_key: cfg.app_key,
            app_secret: SecretString::new(cfg.app_secret.into()),
            environment: parse_environment(cfg.environment)?,
            http_settings: bd_payment_gateway_core::HttpSettings::default(),
        };
        let inner =
            bd_payment_gateway_portwallet::PortwalletClient::new(config).map_err(to_py_err)?;
        Ok(Self { inner })
    }

    fn initiate_payment(&self, request_json: &str) -> PyResult<InitiatePaymentResponse> {
        let request: bd_payment_gateway_portwallet::InitiatePaymentRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let resp = RUNTIME
            .block_on(self.inner.initiate_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_initiate_response(resp))
    }

    fn verify_payment(&self, request_json: &str) -> PyResult<VerifyPaymentResponse> {
        let request: bd_payment_gateway_portwallet::VerifyPaymentRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let resp = RUNTIME
            .block_on(self.inner.verify_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_verify_response(resp))
    }

    fn refund(&self, request_json: &str) -> PyResult<RefundResponse> {
        let request: bd_payment_gateway_portwallet::RefundRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
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
    fn new(config_json: &str) -> PyResult<Self> {
        let cfg: AamarpayConfigInput =
            serde_json::from_str(config_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let config = bd_payment_gateway_aamarpay::Config {
            store_id: cfg.store_id,
            signature_key: SecretString::new(cfg.signature_key.into()),
            environment: parse_environment(cfg.environment)?,
            http_settings: bd_payment_gateway_core::HttpSettings::default(),
        };
        let inner = bd_payment_gateway_aamarpay::AamarpayClient::new(config).map_err(to_py_err)?;
        Ok(Self { inner })
    }

    fn initiate_payment(&self, request_json: &str) -> PyResult<InitiatePaymentResponse> {
        let request: bd_payment_gateway_aamarpay::InitiatePaymentRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let resp = RUNTIME
            .block_on(self.inner.initiate_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_initiate_response(resp))
    }

    fn verify_payment(&self, request_json: &str) -> PyResult<VerifyPaymentResponse> {
        let request: bd_payment_gateway_aamarpay::VerifyPaymentRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
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
    fn new(config_json: &str) -> PyResult<Self> {
        let cfg: SslcommerzConfigInput =
            serde_json::from_str(config_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let config = bd_payment_gateway_sslcommerz::Config {
            store_id: cfg.store_id,
            store_passwd: SecretString::new(cfg.store_passwd.into()),
            environment: parse_environment(cfg.environment)?,
            http_settings: bd_payment_gateway_core::HttpSettings::default(),
        };
        let inner =
            bd_payment_gateway_sslcommerz::SslcommerzClient::new(config).map_err(to_py_err)?;
        Ok(Self { inner })
    }

    fn initiate_payment(&self, request_json: &str) -> PyResult<InitiatePaymentResponse> {
        let request: bd_payment_gateway_sslcommerz::InitiatePaymentRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let resp = RUNTIME
            .block_on(self.inner.initiate_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_initiate_response(resp))
    }

    fn verify_payment(&self, request_json: &str) -> PyResult<VerifyPaymentResponse> {
        let request: bd_payment_gateway_sslcommerz::VerifyPaymentRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let resp = RUNTIME
            .block_on(self.inner.verify_payment(&request))
            .map_err(to_py_err)?;
        Ok(map_verify_response(resp))
    }

    fn refund(&self, request_json: &str) -> PyResult<RefundResponse> {
        let request: bd_payment_gateway_sslcommerz::RefundRequest =
            serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let resp = RUNTIME
            .block_on(self.inner.refund(&request))
            .map_err(to_py_err)?;
        Ok(map_refund_response(resp))
    }
}

#[pymodule]
fn bd_payment_gateway_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
