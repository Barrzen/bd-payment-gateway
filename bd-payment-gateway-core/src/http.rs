use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tokio::time::sleep;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{BdPaymentError, Result};

const REDACTED: &str = "[REDACTED]";

#[derive(Debug, Clone)]
pub struct HttpSettings {
    pub timeout: Duration,
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub user_agent: String,
}

impl Default for HttpSettings {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 2,
            initial_backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(2),
            user_agent: format!("bd-payment-gateway/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpLogRecord {
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub attempt: u32,
    pub duration_ms: u128,
    pub headers: BTreeMap<String, String>,
    pub body: Option<Value>,
    pub request_id: Option<String>,
}

pub trait HttpLogger: Send + Sync {
    fn on_request(&self, _record: &HttpLogRecord) {}
    fn on_response(&self, _record: &HttpLogRecord) {}
    fn on_retry(&self, _record: &HttpLogRecord, _reason: &str) {}
}

#[derive(Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    settings: HttpSettings,
    logger: Option<Arc<dyn HttpLogger>>,
}

impl HttpClient {
    pub fn new(settings: HttpSettings, logger: Option<Arc<dyn HttpLogger>>) -> Result<Self> {
        let inner = reqwest::Client::builder()
            .timeout(settings.timeout)
            .user_agent(settings.user_agent.clone())
            .build()
            .map_err(|e| {
                BdPaymentError::config(
                    format!("Failed to build HTTP client: {e}"),
                    "Verify TLS setup and ensure runtime supports rustls.",
                )
            })?;
        Ok(Self {
            inner,
            settings,
            logger,
        })
    }

    pub fn with_default_settings() -> Result<Self> {
        Self::new(HttpSettings::default(), None)
    }

    pub async fn get_json<R: DeserializeOwned>(
        &self,
        url: &url::Url,
        headers: HeaderMap,
    ) -> Result<R> {
        self.request_json::<(), R>(Method::GET, url, headers, None)
            .await
    }

    pub async fn post_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &url::Url,
        headers: HeaderMap,
        body: &T,
    ) -> Result<R> {
        self.request_json(Method::POST, url, headers, Some(body))
            .await
    }

    pub async fn post_form<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &url::Url,
        headers: HeaderMap,
        form: &T,
    ) -> Result<R> {
        let mut headers = headers;
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let redacted_headers = redact_headers(&headers);
        let encoded_form = serde_urlencoded::to_string(form).map_err(|e| {
            BdPaymentError::validation(
                format!("Failed to encode form body: {e}"),
                "Check form fields and ensure serializable string values are provided.",
            )
        })?;

        let mut attempt = 0;
        loop {
            attempt += 1;
            let started = Instant::now();
            let req = self
                .inner
                .post(url.clone())
                .headers(headers.clone())
                .body(encoded_form.clone());

            self.log_request(&HttpLogRecord {
                method: "POST".to_owned(),
                url: url.to_string(),
                status: None,
                attempt,
                duration_ms: 0,
                headers: redacted_headers.clone(),
                body: None,
                request_id: None,
            });

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let request_id = extract_request_id(resp.headers());
                    let text = resp.text().await.map_err(|e| {
                        BdPaymentError::http(
                            format!("Unable to read HTTP response body: {e}"),
                            "Retry once. If persistent, capture provider status page and contact support.",
                            None,
                            request_id.clone(),
                            None,
                        )
                    })?;

                    let log = HttpLogRecord {
                        method: "POST".to_owned(),
                        url: url.to_string(),
                        status: Some(status.as_u16()),
                        attempt,
                        duration_ms: started.elapsed().as_millis(),
                        headers: redacted_headers.clone(),
                        body: None,
                        request_id: request_id.clone(),
                    };

                    if status.is_success() {
                        self.log_response(&log);
                        return parse_json::<R>(&text, request_id.clone());
                    }

                    if self.should_retry_status(status, attempt) {
                        self.log_retry(&log, "retryable HTTP status");
                        self.wait_backoff(attempt).await;
                        continue;
                    }

                    self.log_response(&log);
                    return Err(BdPaymentError::http(
                        format!("HTTP {} calling {}", status.as_u16(), url),
                        "Verify API credentials, endpoint environment (sandbox/live), and payload fields.",
                        Some(status.as_u16()),
                        request_id,
                        Some(truncate(&text, 1024)),
                    ));
                }
                Err(err) => {
                    let log = HttpLogRecord {
                        method: "POST".to_owned(),
                        url: url.to_string(),
                        status: None,
                        attempt,
                        duration_ms: started.elapsed().as_millis(),
                        headers: redacted_headers.clone(),
                        body: None,
                        request_id: None,
                    };
                    if self.should_retry_network(&err, attempt) {
                        self.log_retry(&log, "network error");
                        self.wait_backoff(attempt).await;
                        continue;
                    }
                    return Err(BdPaymentError::http(
                        format!("Network call to {} failed: {err}", url),
                        "Check DNS, connectivity, TLS trust roots, and provider uptime.",
                        None,
                        None,
                        None,
                    ));
                }
            }
        }
    }

    pub async fn request_json<T: Serialize, R: DeserializeOwned>(
        &self,
        method: Method,
        url: &url::Url,
        headers: HeaderMap,
        body: Option<&T>,
    ) -> Result<R> {
        let serialized_body = body.map(serde_json::to_value).transpose().map_err(|e| {
            BdPaymentError::validation(
                format!("Failed to serialize request body: {e}"),
                "Ensure all request fields are serializable and required fields are present.",
            )
        })?;

        let redacted_body = serialized_body.as_ref().map(redact_json);
        let redacted_headers = redact_headers(&headers);

        let mut attempt = 0;
        loop {
            attempt += 1;
            let started = Instant::now();

            let mut req = self
                .inner
                .request(method.clone(), url.clone())
                .headers(headers.clone());

            if let Some(body) = &serialized_body {
                req = req.json(body);
            }

            self.log_request(&HttpLogRecord {
                method: method.to_string(),
                url: url.to_string(),
                status: None,
                attempt,
                duration_ms: 0,
                headers: redacted_headers.clone(),
                body: redacted_body.clone(),
                request_id: None,
            });

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let request_id = extract_request_id(resp.headers());
                    let text = resp.text().await.map_err(|e| {
                        BdPaymentError::http(
                            format!("Unable to read HTTP response body: {e}"),
                            "Retry once. If persistent, capture provider status page and contact support.",
                            None,
                            request_id.clone(),
                            None,
                        )
                    })?;

                    let log = HttpLogRecord {
                        method: method.to_string(),
                        url: url.to_string(),
                        status: Some(status.as_u16()),
                        attempt,
                        duration_ms: started.elapsed().as_millis(),
                        headers: redacted_headers.clone(),
                        body: None,
                        request_id: request_id.clone(),
                    };

                    if status.is_success() {
                        self.log_response(&log);
                        return parse_json::<R>(&text, request_id.clone());
                    }

                    if self.should_retry_status(status, attempt) {
                        self.log_retry(&log, "retryable HTTP status");
                        self.wait_backoff(attempt).await;
                        continue;
                    }

                    self.log_response(&log);
                    return Err(BdPaymentError::http(
                        format!("HTTP {} calling {}", status.as_u16(), url),
                        "Verify API credentials, endpoint environment (sandbox/live), and payload fields.",
                        Some(status.as_u16()),
                        request_id,
                        Some(truncate(&text, 1024)),
                    ));
                }
                Err(err) => {
                    let log = HttpLogRecord {
                        method: method.to_string(),
                        url: url.to_string(),
                        status: None,
                        attempt,
                        duration_ms: started.elapsed().as_millis(),
                        headers: redacted_headers.clone(),
                        body: redacted_body.clone(),
                        request_id: None,
                    };
                    if self.should_retry_network(&err, attempt) {
                        self.log_retry(&log, "network error");
                        self.wait_backoff(attempt).await;
                        continue;
                    }
                    return Err(BdPaymentError::http(
                        format!("Network call to {} failed: {err}", url),
                        "Check DNS, connectivity, TLS trust roots, and provider uptime.",
                        None,
                        None,
                        None,
                    ));
                }
            }
        }
    }

    fn should_retry_status(&self, status: StatusCode, attempt: u32) -> bool {
        attempt <= self.settings.max_retries
            && (status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error())
    }

    fn should_retry_network(&self, err: &reqwest::Error, attempt: u32) -> bool {
        attempt <= self.settings.max_retries
            && (err.is_connect() || err.is_timeout() || err.is_request())
    }

    async fn wait_backoff(&self, attempt: u32) {
        let factor = 2_u32.saturating_pow(attempt.saturating_sub(1));
        let backoff = self
            .settings
            .initial_backoff
            .saturating_mul(factor)
            .min(self.settings.max_backoff);
        sleep(backoff).await;
    }

    fn log_request(&self, record: &HttpLogRecord) {
        debug!(
            method = %record.method,
            url = %record.url,
            attempt = record.attempt,
            "payment sdk request"
        );
        if let Some(logger) = &self.logger {
            logger.on_request(record);
        }
    }

    fn log_response(&self, record: &HttpLogRecord) {
        debug!(
            method = %record.method,
            url = %record.url,
            status = ?record.status,
            duration_ms = record.duration_ms,
            "payment sdk response"
        );
        if let Some(logger) = &self.logger {
            logger.on_response(record);
        }
    }

    fn log_retry(&self, record: &HttpLogRecord, reason: &str) {
        warn!(
            method = %record.method,
            url = %record.url,
            attempt = record.attempt,
            reason,
            "payment sdk retry"
        );
        if let Some(logger) = &self.logger {
            logger.on_retry(record, reason);
        }
    }
}

fn parse_json<R: DeserializeOwned>(text: &str, request_id: Option<String>) -> Result<R> {
    serde_json::from_str::<R>(text).map_err(|e| {
        BdPaymentError::parse(
            format!("Failed to parse JSON response: {e}"),
            format!(
                "Provider returned invalid JSON. request_id={:?}, body_snippet={}",
                request_id,
                truncate(text, 250)
            ),
        )
    })
}

fn extract_request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .or_else(|| headers.get("x-correlation-id"))
        .or_else(|| headers.get("request-id"))
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
}

pub fn generate_correlation_id() -> String {
    Uuid::now_v7().to_string()
}

pub fn generate_idempotency_key() -> String {
    Uuid::now_v7().to_string()
}

pub fn add_default_headers(
    mut headers: HeaderMap,
    correlation_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<HeaderMap> {
    if let Some(correlation_id) = correlation_id {
        headers.insert(
            HeaderName::from_static("x-correlation-id"),
            HeaderValue::from_str(correlation_id).map_err(|e| {
                BdPaymentError::validation(
                    format!("Invalid correlation ID header value: {e}"),
                    "Use an ASCII-safe UUID-like value.",
                )
            })?,
        );
    }

    if let Some(idempotency_key) = idempotency_key {
        headers.insert(
            HeaderName::from_static("idempotency-key"),
            HeaderValue::from_str(idempotency_key).map_err(|e| {
                BdPaymentError::validation(
                    format!("Invalid idempotency key header value: {e}"),
                    "Use an ASCII-safe UUID-like value.",
                )
            })?,
        );
    }

    Ok(headers)
}

fn redact_headers(headers: &HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| {
            let key = k.as_str().to_owned();
            let value = if is_sensitive_key(k.as_str()) {
                REDACTED.to_owned()
            } else {
                v.to_str().unwrap_or("<binary>").to_owned()
            };
            (key, value)
        })
        .collect()
}

pub fn redact_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| {
                    if is_sensitive_key(k) {
                        (k.clone(), Value::String(REDACTED.to_owned()))
                    } else {
                        (k.clone(), redact_json(v))
                    }
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(redact_json).collect()),
        _ => value.clone(),
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "token",
        "secret",
        "password",
        "authorization",
        "key",
        "store_id",
        "signature",
    ]
    .iter()
    .any(|needle| key.contains(needle))
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_owned();
    }
    let truncated: String = s.chars().take(max_chars).collect();
    format!("{truncated}...")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::redact_json;

    #[test]
    fn redacts_sensitive_json_fields() {
        let value = json!({
            "api_key": "123",
            "nested": {"token": "abc", "visible": "ok"}
        });

        let redacted = redact_json(&value);
        assert_eq!(redacted["api_key"], "[REDACTED]");
        assert_eq!(redacted["nested"]["token"], "[REDACTED]");
        assert_eq!(redacted["nested"]["visible"], "ok");
    }
}
