//! Card payments API (`pay.solidgate.com`): charges by a saved card token and order status.
//!
//! An order is identified by the merchant's `order_id`. A charge answers with the order in its
//! current status; a `3ds_verify` order waits for the customer at `verify_url`, and the final
//! status arrives later through the order status call or the `card_gate.order.updated` webhook.

use crate::model::SolidGateApiError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum RecurringPaymentType {
    /// A purchase the customer starts, paid with the card of an earlier payment.
    #[serde(rename = "1-click")]
    OneClick,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum Platform {
    #[serde(rename = "WEB")]
    Web,
    #[serde(rename = "MOB")]
    Mobile,
    #[serde(rename = "APP")]
    App,
}

/// `POST /recurring`: charges the card behind `recurring_token`.
#[derive(Debug, Serialize)]
pub struct RecurringRequest {
    pub order_id: String,
    /// Smallest currency unit.
    pub amount: i64,
    pub currency: String,
    pub order_description: String,
    pub payment_type: RecurringPaymentType,
    pub recurring_token: String,
    pub customer_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_account_id: Option<String>,
    pub ip_address: String,
    pub platform: Platform,
    pub product_id: String,
    /// Where the customer returns after 3DS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderStatusRequest {
    pub order_id: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CardOrderStatus {
    Processing,
    #[serde(rename = "3ds_verify")]
    ThreeDsVerify,
    AuthOk,
    AuthFailed,
    SettleOk,
    PartialSettled,
    VoidOk,
    Refunded,
    /// A status added after this crate; callers treat it as not final.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct CardOrder {
    pub order_id: String,
    pub status: CardOrderStatus,
    #[serde(default)]
    pub amount: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub refunded_amount: Option<i64>,
    #[serde(default)]
    pub product_id: Option<String>,
    #[serde(default)]
    pub subscription_id: Option<String>,
    #[serde(default)]
    pub payment_type: Option<String>,
    #[serde(default)]
    pub payment_method: Option<String>,
    #[serde(default)]
    pub customer_email: Option<String>,
    #[serde(default)]
    pub customer_account_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardToken {
    pub token: String,
    /// `apple-pay` or `google-pay` for a wallet payment.
    #[serde(default)]
    pub original_payment_method: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    #[serde(default)]
    pub brand: Option<String>,
    /// Masked, e.g. `444111XXXXXX9435`.
    #[serde(default)]
    pub number: Option<String>,
    #[serde(default)]
    pub card_token: Option<CardToken>,
}

#[derive(Debug, Deserialize)]
pub struct CardTransaction {
    pub id: String,
    #[serde(default)]
    pub operation: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub card_token: Option<CardToken>,
    #[serde(default)]
    pub card: Option<Card>,
    #[serde(default)]
    pub error: Option<SolidGateApiError>,
}

/// An order with its last transaction, as `/recurring`, `/status` and the
/// `card_gate.order.updated` webhook carry it.
#[derive(Debug, Deserialize)]
pub struct CardOrderResponse {
    pub order: CardOrder,
    #[serde(default)]
    pub transaction: Option<CardTransaction>,
    #[serde(default)]
    pub transactions: HashMap<String, CardTransaction>,
    /// Set while the order waits for 3DS.
    #[serde(default)]
    pub verify_url: Option<String>,
}

impl CardOrderResponse {
    /// The card token of the order: from its last transaction, else from any of them.
    pub fn card_token(&self) -> Option<&CardToken> {
        self.transaction
            .as_ref()
            .and_then(CardTransaction::card_token)
            .or_else(|| {
                self.transactions
                    .values()
                    .find_map(CardTransaction::card_token)
            })
    }
}

impl CardTransaction {
    fn card_token(&self) -> Option<&CardToken> {
        self.card_token
            .as_ref()
            .or_else(|| self.card.as_ref().and_then(|c| c.card_token.as_ref()))
    }
}

/// `card_gate.chargeback.received`: a chargeback opened against an order.
#[derive(Debug, Deserialize)]
pub struct ChargebackWebhookModel {
    pub order: ChargebackOrder,
    pub chargeback: Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct ChargebackOrder {
    pub order_id: String,
    #[serde(default)]
    pub amount: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Chargeback {
    /// E.g. `1st_chb`.
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub reason_code: Option<String>,
    #[serde(default)]
    pub reason_description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trimmed `/recurring` answer from the SolidGate API reference.
    const RECURRING_RESPONSE: &str = r#"{
        "order": {
            "order_id": "923bb4e6-4a5f-41ec-81fb-28eb8a152e55",
            "product_id": "a51ba9fd-3be1-4ef9-b00f-bb85157597f5",
            "amount": 100, "currency": "USD", "refunded_amount": 0,
            "status": "settle_ok", "payment_type": "1-click", "payment_method": "card",
            "customer_email": "example@example.com"
        },
        "transaction": {
            "id": "5019d00bb70f82cd42f6bc654cbdfcbd63a9b5b1dbd6a",
            "operation": "recurring-auth", "status": "processing",
            "card": {
                "brand": "VISA", "number": "444111XXXXXX9435",
                "card_token": {"token": "baf2ff", "original_payment_method": "apple-pay"}
            }
        },
        "transactions": {}
    }"#;

    #[test]
    fn parses_recurring_response() {
        let response: CardOrderResponse =
            serde_json::from_str(RECURRING_RESPONSE).expect("reference answer must parse");
        assert_eq!(response.order.status, CardOrderStatus::SettleOk);
        assert_eq!(response.order.amount, Some(100));
        let token = response.card_token().expect("the card carries the token");
        assert_eq!(token.token, "baf2ff");
        assert_eq!(token.original_payment_method.as_deref(), Some("apple-pay"));
    }

    #[test]
    fn unknown_status_parses() {
        let order: CardOrder =
            serde_json::from_str(r#"{"order_id": "o1", "status": "some_new_status"}"#)
                .expect("an unknown status falls back to Unknown");
        assert_eq!(order.status, CardOrderStatus::Unknown);
        let order: CardOrder =
            serde_json::from_str(r#"{"order_id": "o1", "status": "3ds_verify"}"#)
                .expect("3ds_verify is a known status");
        assert_eq!(order.status, CardOrderStatus::ThreeDsVerify);
    }

    #[test]
    fn recurring_request_wire_names() {
        let request = RecurringRequest {
            order_id: "o1".to_string(),
            amount: 1999,
            currency: "USD".to_string(),
            order_description: "AI assistant".to_string(),
            payment_type: RecurringPaymentType::OneClick,
            recurring_token: "t".to_string(),
            customer_email: "a@b.c".to_string(),
            customer_account_id: None,
            ip_address: "203.0.113.0".to_string(),
            platform: Platform::Web,
            product_id: "p1".to_string(),
            success_url: None,
            fail_url: None,
        };
        let json = serde_json::to_value(&request).expect("the request holds only plain values");
        assert_eq!(json["payment_type"], "1-click");
        assert_eq!(json["platform"], "WEB");
        assert!(json.get("success_url").is_none());
    }

    #[test]
    fn parses_chargeback_webhook() {
        let body = r#"{
            "order": {"id": "pay_1", "order_id": "academy-1", "amount": 100, "currency": "USD", "status": "settle_ok"},
            "chargeback": {"id": 147712, "type": "1st_chb", "status": "in_progress", "reason_code": "10.4",
                           "reason_description": "Fraud - Card-Absent Environment"},
            "chargeback_flow": []
        }"#;
        let webhook: ChargebackWebhookModel =
            serde_json::from_str(body).expect("reference chargeback must parse");
        assert_eq!(webhook.order.order_id, "academy-1");
        assert_eq!(webhook.chargeback.kind.as_deref(), Some("1st_chb"));
        assert_eq!(webhook.chargeback.reason_code.as_deref(), Some("10.4"));
    }
}
