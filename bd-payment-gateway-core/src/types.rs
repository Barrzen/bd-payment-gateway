use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Currency {
    Bdt,
    Usd,
    Eur,
    Other(String),
}

impl Currency {
    pub fn as_code(&self) -> &str {
        match self {
            Self::Bdt => "BDT",
            Self::Usd => "USD",
            Self::Eur => "EUR",
            Self::Other(v) => v.as_str(),
        }
    }
}

impl Serialize for Currency {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_code())
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let upper = raw.to_ascii_uppercase();
        Ok(match upper.as_str() {
            "BDT" => Self::Bdt,
            "USD" => Self::Usd,
            "EUR" => Self::Eur,
            _ => Self::Other(upper),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Customer {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postcode: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct OrderId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TransactionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct RedirectUrl(pub Url);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub provider: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum Environment {
    Sandbox,
    Production,
    CustomBaseUrl(Url),
}

impl Environment {
    pub fn resolve(&self, sandbox_base: &str, production_base: &str) -> crate::Result<Url> {
        match self {
            Self::Sandbox => Url::parse(sandbox_base).map_err(|e| {
                crate::BdPaymentError::config(
                    format!("Invalid sandbox base URL: {e}"),
                    "Use the provider default sandbox URL or provide a valid CustomBaseUrl.",
                )
            }),
            Self::Production => Url::parse(production_base).map_err(|e| {
                crate::BdPaymentError::config(
                    format!("Invalid production base URL: {e}"),
                    "Use the provider default production URL or provide a valid CustomBaseUrl.",
                )
            }),
            Self::CustomBaseUrl(url) => Ok(url.clone()),
        }
    }
}
