use serde::{Deserialize, Serialize};

// TODO: точная структура webhook payload от SolidGate приходит с документацией.
// Сейчас — минимальный stub чтобы webhook-сервис компилировался.
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookModel {
    #[serde(rename = "type")]
    pub event_type: String,
    pub action: String,
}
