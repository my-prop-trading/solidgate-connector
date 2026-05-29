use serde::{Deserialize, Serialize};

// TODO: реальная структура subscription-webhook'а SolidGate уточняется по доке.
// Сейчас держим placeholder.
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionWebhookModel {
    #[serde(rename = "type")]
    pub event_type: String,
    pub action: SubscriptionActionWrapper,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionAction {
    // TODO: реальные event names SolidGate.
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SubscriptionActionWrapper {
    Known(SubscriptionAction),
    Unknown(String),
}
