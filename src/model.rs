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

/// SolidGate error envelope, e.g. `{"error":{"code":"1.01","messages":["Authentication failed"]}}`.
/// SolidGate sometimes returns this with a 2xx status, so we detect it explicitly.
#[derive(Debug, Deserialize)]
pub struct ErrorEnvelope {
    pub error: Option<SolidGateApiError>,
}

#[derive(Debug, Deserialize)]
pub struct SolidGateApiError {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub messages: Vec<String>,
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

/// Restore a previously cancelled subscription (SolidGate `POST /subscription/restore`).
/// SolidGate only restores a subscription whose status is `cancelled` and for which no other
/// active/processing subscription exists for the same customer+product.
#[derive(Debug, Serialize)]
pub struct RestoreSubscriptionRequest {
    pub subscription_id: String,
    /// Optional new expiration (`YYYY-MM-DD HH:MM:SS`). A past value is bumped to "now" by SolidGate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreSubscriptionResponse {
    #[serde(default)]
    pub status: Option<String>,
}

// ----------------------------------------------------------------------------
// Webhook endpoint management (SolidGate "Manage webhooks" API).
// Lives on the general API host (https://api.solidgate.com/api/v1), not on the
// subscriptions host used by cancel_subscription.
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WebhookEndpointStatus {
    Active,
    Inactive,
}

/// Create a webhook endpoint. `event_types` are SolidGate event identifiers
/// (e.g. [`crate::webhook::event_types::SUBSCRIPTION_UPDATED_V2`]).
#[derive(Debug, Serialize)]
pub struct CreateWebhookEndpointRequest {
    pub url: String,
    pub event_types: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<WebhookEndpointStatus>,
}

/// Patch a webhook endpoint. Every field is optional — only the present ones are updated.
#[derive(Debug, Default, Serialize)]
pub struct UpdateWebhookEndpointRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<WebhookEndpointStatus>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub url: String,
    #[serde(default)]
    pub event_types: Vec<String>,
    #[serde(default)]
    pub name: Option<String>,
    pub status: WebhookEndpointStatus,
    /// SolidGate format: `YYYY-MM-DD HH:MM:SS`.
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookEndpointPagination {
    pub offset: i64,
    pub limit: i64,
    pub total_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListWebhookEndpointsResponse {
    #[serde(default)]
    pub data: Vec<WebhookEndpoint>,
    #[serde(default)]
    pub pagination: Option<WebhookEndpointPagination>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serializes_lowercase() {
        let json = serde_json::to_string(&WebhookEndpointStatus::Inactive).unwrap();
        assert_eq!(json, "\"inactive\"");
    }

    #[test]
    fn update_request_skips_none_fields() {
        let req = UpdateWebhookEndpointRequest {
            status: Some(WebhookEndpointStatus::Inactive),
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"status":"inactive"}"#);
    }

    #[test]
    fn create_request_serializes_event_types() {
        let req = CreateWebhookEndpointRequest {
            url: "https://example.com/cb".to_string(),
            event_types: vec![crate::webhook::event_types::SUBSCRIPTION_UPDATED_V2.to_string()],
            name: None,
            status: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(
            json,
            r#"{"url":"https://example.com/cb","event_types":["subscription.updated.v2"]}"#
        );
    }

    #[test]
    fn error_envelope_parses() {
        let raw = r#"{"error":{"code":"1.01","messages":["Authentication failed"]}}"#;
        let env: ErrorEnvelope = serde_json::from_str(raw).unwrap();
        let err = env.error.expect("error present");
        assert_eq!(err.code, "1.01");
        assert_eq!(err.messages, vec!["Authentication failed".to_string()]);
    }

    #[test]
    fn success_body_has_no_error() {
        let raw = r#"{"id":"we_1","url":"https://e.com","status":"active"}"#;
        let env: ErrorEnvelope = serde_json::from_str(raw).unwrap();
        assert!(env.error.is_none());
    }

    #[test]
    fn endpoint_deserializes() {
        let raw = r#"{"id":"we_1","url":"https://e.com","event_types":["subscription.updated.v2"],"name":"academy","status":"active","created_at":"2026-06-01 10:00:00","updated_at":"2026-06-01 10:00:00"}"#;
        let ep: WebhookEndpoint = serde_json::from_str(raw).unwrap();
        assert_eq!(ep.id, "we_1");
        assert_eq!(ep.status, WebhookEndpointStatus::Active);
        assert_eq!(ep.event_types, vec!["subscription.updated.v2".to_string()]);
    }
}
