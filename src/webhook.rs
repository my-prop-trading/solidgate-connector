/// HTTP headers SolidGate puts on every webhook delivery.
pub mod headers {
    pub const MERCHANT: &str = "merchant";
    pub const SIGNATURE: &str = "signature";
    pub const EVENT_ID: &str = "solidgate-event-id";
    pub const EVENT_CREATED_AT: &str = "solidgate-event-created-at";
    pub const EVENT_TYPE: &str = "solidgate-event-type";
}

/// Webhook `solidgate-event-type` values that academy-webhook expects to receive.
pub mod event_types {
    /// V2 of the subscription-updated event (recommended for new integrations).
    pub const SUBSCRIPTION_UPDATED_V2: &str = "subscription.updated.v2";
    /// V1 of the subscription-updated event (legacy).
    pub const SUBSCRIPTION_UPDATED_V1: &str = "subscription.updated";
}
