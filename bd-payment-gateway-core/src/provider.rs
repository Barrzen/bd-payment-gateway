use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{BdPaymentError, Currency, Money, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Cancelled,
    Refunded,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefundStatus {
    Pending,
    Completed,
    Failed,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiatePaymentResponse {
    pub redirect_url: Url,
    pub provider_reference: String,
    pub raw: serde_json::Value,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentResponse {
    pub status: PaymentStatus,
    pub provider_reference: String,
    pub amount: Option<Decimal>,
    pub currency: Option<Currency>,
    pub money: Option<Money>,
    pub raw: serde_json::Value,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub status: RefundStatus,
    pub provider_reference: String,
    pub raw: serde_json::Value,
    pub request_id: Option<String>,
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    type InitiateRequest: Send + Sync;
    type VerifyRequest: Send + Sync;
    type RefundRequest: Send + Sync;

    async fn initiate_payment(
        &self,
        req: &Self::InitiateRequest,
    ) -> Result<InitiatePaymentResponse>;

    async fn verify_payment(&self, req: &Self::VerifyRequest) -> Result<VerifyPaymentResponse>;

    async fn refund(&self, _req: &Self::RefundRequest) -> Result<RefundResponse> {
        Err(BdPaymentError::unsupported(
            "This provider does not support refunds through this SDK API.",
            "Use provider dashboard/manual refund flow, or call provider-specific refund API if available.",
        ))
    }
}
