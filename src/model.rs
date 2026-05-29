use serde::{Deserialize, Serialize};

/// SolidGate API authentication uses HMAC headers (Merchant + Signature); no per-request envelope.
/// We keep a simple wrapper for symmetry with billerix-connector's ApiRequest.
pub struct ApiRequest<T: Serialize> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CancelSubscriptionCode {
    /// Customer-initiated cancellation. Default used by academy-bridge.
    #[serde(rename = "8.14")]
    CustomerRequest,
    #[serde(rename = "8.11")]
    TokenRevokedByCustomer,
    #[serde(rename = "8.10")]
    TokenHasExpired,
    #[serde(rename = "8.09")]
    FailedRedemption,
    #[serde(rename = "8.06")]
    SupportRequest,
    #[serde(rename = "0.00")]
    BackendRequest,
}

#[derive(Debug, Serialize)]
pub struct CancelSubscriptionRequest {
    pub subscription_id: String,
    /// `false` → cancel at the end of the current billing period (subscription stays active until then).
    /// `true`  → immediate termination.
    pub force: bool,
    pub cancel_code: CancelSubscriptionCode,
}

#[derive(Debug, Deserialize)]
pub struct CancelSubscriptionResponse {
    #[serde(default)]
    pub status: Option<String>,
}
