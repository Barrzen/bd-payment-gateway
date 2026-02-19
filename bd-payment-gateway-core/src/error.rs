use thiserror::Error;

pub type Result<T> = std::result::Result<T, BdPaymentError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    ConfigInvalid,
    ValidationFailed,
    HttpFailure,
    ProviderRejected,
    UnsupportedOperation,
    ParseFailed,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConfigInvalid => "CONFIG_INVALID",
            Self::ValidationFailed => "VALIDATION_FAILED",
            Self::HttpFailure => "HTTP_FAILURE",
            Self::ProviderRejected => "PROVIDER_REJECTED",
            Self::UnsupportedOperation => "UNSUPPORTED_OPERATION",
            Self::ParseFailed => "PARSE_FAILED",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Error)]
pub enum BdPaymentError {
    #[error("Configuration error: {message}. Hint: {hint}")]
    ConfigError {
        code: ErrorCode,
        message: String,
        hint: String,
    },
    #[error("Validation error: {message}. Hint: {hint}")]
    ValidationError {
        code: ErrorCode,
        message: String,
        hint: String,
    },
    #[error("HTTP error: {message}. Hint: {hint}")]
    HttpError {
        code: ErrorCode,
        message: String,
        hint: String,
        status: Option<u16>,
        request_id: Option<String>,
        body: Option<String>,
    },
    #[error("Provider rejected request: {message}. Hint: {hint}")]
    ProviderError {
        code: ErrorCode,
        message: String,
        hint: String,
        provider_code: Option<String>,
        request_id: Option<String>,
    },
    #[error("Operation unsupported: {message}. Hint: {hint}")]
    Unsupported {
        code: ErrorCode,
        message: String,
        hint: String,
    },
    #[error("Parse error: {message}. Hint: {hint}")]
    ParseError {
        code: ErrorCode,
        message: String,
        hint: String,
    },
}

impl BdPaymentError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::ConfigError { code, .. }
            | Self::ValidationError { code, .. }
            | Self::HttpError { code, .. }
            | Self::ProviderError { code, .. }
            | Self::Unsupported { code, .. }
            | Self::ParseError { code, .. } => *code,
        }
    }

    pub fn hint(&self) -> &str {
        match self {
            Self::ConfigError { hint, .. }
            | Self::ValidationError { hint, .. }
            | Self::HttpError { hint, .. }
            | Self::ProviderError { hint, .. }
            | Self::Unsupported { hint, .. }
            | Self::ParseError { hint, .. } => hint,
        }
    }

    pub fn config(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::ConfigError {
            code: ErrorCode::ConfigInvalid,
            message: message.into(),
            hint: hint.into(),
        }
    }

    pub fn validation(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::ValidationError {
            code: ErrorCode::ValidationFailed,
            message: message.into(),
            hint: hint.into(),
        }
    }

    pub fn http(
        message: impl Into<String>,
        hint: impl Into<String>,
        status: Option<u16>,
        request_id: Option<String>,
        body: Option<String>,
    ) -> Self {
        Self::HttpError {
            code: ErrorCode::HttpFailure,
            message: message.into(),
            hint: hint.into(),
            status,
            request_id,
            body,
        }
    }

    pub fn provider(
        message: impl Into<String>,
        hint: impl Into<String>,
        provider_code: Option<String>,
        request_id: Option<String>,
    ) -> Self {
        Self::ProviderError {
            code: ErrorCode::ProviderRejected,
            message: message.into(),
            hint: hint.into(),
            provider_code,
            request_id,
        }
    }

    pub fn unsupported(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::Unsupported {
            code: ErrorCode::UnsupportedOperation,
            message: message.into(),
            hint: hint.into(),
        }
    }

    pub fn parse(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::ParseError {
            code: ErrorCode::ParseFailed,
            message: message.into(),
            hint: hint.into(),
        }
    }
}
