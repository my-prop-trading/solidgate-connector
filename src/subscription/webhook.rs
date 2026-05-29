use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionData {
    pub id: String,
    pub status: SubscriptionStatus,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub expired_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub next_charge_at: Option<DateTime<Utc>>,
    #[serde(default)]
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
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub amount: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
}
