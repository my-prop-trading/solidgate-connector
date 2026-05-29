use crate::model::{ApiRequest, ApiResponse, DisableSubscriptionRequest, OneTimePaymentRequest, OneTimePaymentResponse};
use std::time::Duration;

pub struct SolidGateApi {
    base_url: String,
    merchant_id: String,
    public_key: String,
    secret_key: String,
    timeout: Duration,
}

impl SolidGateApi {
    pub fn new(
        base_url: impl Into<String>,
        merchant_id: impl Into<String>,
        public_key: impl Into<String>,
        secret_key: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            merchant_id: merchant_id.into(),
            public_key: public_key.into(),
            secret_key: secret_key.into(),
            timeout,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn merchant_id(&self) -> &str {
        &self.merchant_id
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub fn secret_key(&self) -> &str {
        &self.secret_key
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub async fn one_time_payment(
        &self,
        _req: &ApiRequest<OneTimePaymentRequest>,
    ) -> Result<ApiResponse<OneTimePaymentResponse>, String> {
        Err(format!(
            "solidgate-connector: one_time_payment not implemented (base_url={})",
            self.base_url
        ))
    }

    pub async fn disable_subscription(
        &self,
        _subscription_id: &str,
        _req: &ApiRequest<DisableSubscriptionRequest>,
    ) -> Result<(), String> {
        Err(format!(
            "solidgate-connector: disable_subscription not implemented (base_url={})",
            self.base_url
        ))
    }
}
