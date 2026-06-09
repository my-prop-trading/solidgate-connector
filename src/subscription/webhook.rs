use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// SolidGate sends timestamps as naive UTC strings `"YYYY-MM-DD HH:MM:SS"` (no `T`, no zone),
/// which chrono's default RFC3339 `DateTime<Utc>` deserializer rejects ("premature end of input").
/// Parse them as naive UTC. Empty/absent → `None`.
fn deserialize_solidgate_dt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if !s.is_empty() => NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .map(|naive| Some(naive.and_utc()))
            .map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

/// Payload of `subscription.updated.v2` webhook from SolidGate.
///
/// SolidGate delivers the full subscription snapshot on every change; the high-level reason
/// for the delivery lives in `callback_type`. Status field on `subscription` carries the
/// current resulting state (pending/active/cancelled/redemption/paused/expired).
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionWebhookModel {
    /// Reason for this delivery (active / renew / cancel / pause / resume / redemption / create).
    pub callback_type: CallbackTypeWrapper,
    pub subscription: SubscriptionData,
    pub customer: SubscriptionCustomer,
    pub product: SubscriptionProduct,
    /// Map of invoice_id → invoice. We don't need it for academy bookkeeping but keep the field
    /// so deserialization never fails on its presence.
    #[serde(default)]
    pub invoices: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallbackType {
    /// Subscription created with `pending` status, awaiting first payment. Academy ignores it
    /// and waits for `active` instead.
    #[serde(rename = "create")]
    Create,
    /// First successful charge — subscription becomes `active`. Academy treats this as Create.
    #[serde(rename = "active")]
    Active,
    /// Recurring charge succeeded; new billing period started.
    #[serde(rename = "renew")]
    Renew,
    /// Subscription is paused (per pause schedule).
    #[serde(rename = "pause")]
    Pause,
    /// Pause period ended; subscription resumed.
    #[serde(rename = "resume")]
    Resume,
    /// Subscription cancelled (any reason).
    #[serde(rename = "cancel")]
    Cancel,
    /// Recurring charge failed; subscription is in retry/redemption window.
    #[serde(rename = "redemption")]
    Redemption,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CallbackTypeWrapper {
    Known(CallbackType),
    Unknown(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Pending,
    Active,
    Cancelled,
    Redemption,
    Paused,
    Expired,
    /// Any status SolidGate adds in the future — keeps deserialization from failing; the webhook
    /// service treats it as "skip" rather than misclassifying it.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionData {
    pub id: String,
    pub status: SubscriptionStatus,
    #[serde(default, deserialize_with = "deserialize_solidgate_dt")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "deserialize_solidgate_dt")]
    pub expired_at: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "deserialize_solidgate_dt")]
    pub next_charge_at: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "deserialize_solidgate_dt")]
    pub cancelled_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub trial: bool,
    #[serde(default)]
    pub cancel_code: Option<String>,
    #[serde(default)]
    pub cancel_message: Option<String>,
    #[serde(default)]
    pub payment_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionCustomer {
    pub customer_account_id: String,
    #[serde(default)]
    pub customer_email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionProduct {
    /// SolidGate names this `product_id` in the webhook payload.
    #[serde(rename = "product_id")]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub amount: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real `subscription.updated.v2` body that previously failed to parse:
    /// naive-UTC dates (`"YYYY-MM-DD HH:MM:SS"`), `product_id` (not `id`), `callback_type: order_update`.
    const REAL_PAYLOAD: &str = r#"{"subscription":{"id":"21967be8-62dd-4ad7-905e-a4c258499e88","next_charge_at":"2026-09-07 11:22:15","status":"active","updated_at":"2026-06-09 11:22:15","started_at":"2026-06-09 11:22:15","expired_at":"2026-09-07 11:22:15","trial":false,"payment_type":"card"},"product":{"product_id":"b079959f-3c80-4008-a322-38d0222712ec","currency":"USD","name":"3 MONTHS","amount":2400,"trial":false},"customer":{"customer_email":"test@test.com","customer_account_id":"0e855ea1-16fa-4b3b-877c-c2c1f92dde00"},"invoices":{"7dc15d43-b05c-4cbe-b0df-2a8a18c75b03":{"id":"7dc15d43-b05c-4cbe-b0df-2a8a18c75b03","amount":2400,"status":"success"}},"callback_type":"order_update"}"#;

    #[test]
    fn parses_real_subscription_updated_v2_payload() {
        let model: SubscriptionWebhookModel =
            serde_json::from_str(REAL_PAYLOAD).expect("real payload must deserialize");

        assert_eq!(model.subscription.id, "21967be8-62dd-4ad7-905e-a4c258499e88");
        assert_eq!(model.subscription.status, SubscriptionStatus::Active);
        assert_eq!(
            model.subscription.started_at.unwrap().to_rfc3339(),
            "2026-06-09T11:22:15+00:00"
        );
        assert_eq!(
            model.subscription.expired_at.unwrap().to_rfc3339(),
            "2026-09-07T11:22:15+00:00"
        );
        // `product_id` maps onto `product.id`.
        assert_eq!(model.product.id, "b079959f-3c80-4008-a322-38d0222712ec");
        assert_eq!(model.customer.customer_account_id, "0e855ea1-16fa-4b3b-877c-c2c1f92dde00");
        // Unknown callback_type does not break parsing.
        assert!(matches!(model.callback_type, CallbackTypeWrapper::Unknown(_)));
    }

    #[test]
    fn unknown_status_falls_back_to_unknown_variant() {
        let json = r#"{"id":"s1","status":"some_future_status","trial":false}"#;
        let sub: SubscriptionData = serde_json::from_str(json).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Unknown);
    }
}
