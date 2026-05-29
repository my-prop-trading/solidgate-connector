use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Staff,
    Shop,
    Cp,
    Directlink,
    Sales,
}

impl Source {
    pub fn as_str(&self) -> &'static str {
        match self {
            Source::Staff => "staff",
            Source::Shop => "shop",
            Source::Cp => "cp",
            Source::Directlink => "directlink",
            Source::Sales => "sales",
        }
    }
}

pub struct ApiRequest<T: Serialize> {
    pub ip: String,
    pub data: T,
    pub source: Source,
    pub source_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneTimePaymentRequest {
    pub metadata: Option<HashMap<String, String>>,
    pub price: PriceModel,
    pub buyer: Option<BuyerModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceModel {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuyerModel {
    pub email: String,
    pub locale: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneTimePaymentResponse {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct DisableSubscriptionRequest {
    pub reason: String,
}
