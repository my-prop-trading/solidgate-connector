use crate::generate_signature;
use crate::model::{
    ApiRequest, ApiResponse, CancelSubscriptionRequest, CancelSubscriptionResponse,
};
use flurl::{hyper::Method, FlUrl};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::time::Duration;

pub struct SolidGateApi {
    base_url: String,
    public_key: String,
    secret_key: String,
    timeout: Duration,
}

impl SolidGateApi {
    /// Default subscription-API base URL: <https://subscriptions.solidgate.com/api/v1/>.
    pub fn new(
        base_url: impl Into<String>,
        public_key: impl Into<String>,
        secret_key: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            public_key: public_key.into(),
            secret_key: secret_key.into(),
            timeout,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Cancel a subscription. Pass `force=false` to cancel at the end of the billing period
    /// (subscription stays active until `expired_at`), `true` for immediate termination.
    pub async fn cancel_subscription(
        &self,
        req: &ApiRequest<CancelSubscriptionRequest>,
    ) -> Result<ApiResponse<CancelSubscriptionResponse>, String> {
        self.send_flurl_deserialized("subscription/cancel", &Method::POST, req)
            .await
    }

    async fn send_flurl_deserialized<R: Serialize + Debug, T: DeserializeOwned + Debug>(
        &self,
        endpoint: &str,
        method: &Method,
        request: &ApiRequest<R>,
    ) -> Result<ApiResponse<T>, String> {
        let response = self.send_flurl(endpoint, method, request).await?;
        let parsed: Result<ApiResponse<T>, _> = serde_json::from_str(&response);

        parsed.map_err(|err| {
            format!(
                "Failed to deserialize: {err:?}. Url: {method:?} {endpoint}. \
                 Request: {:?}. Body: {response}",
                request.data
            )
        })
    }

    async fn send_flurl<R: Serialize + Debug>(
        &self,
        endpoint: &str,
        method: &Method,
        request: &ApiRequest<R>,
    ) -> Result<String, String> {
        let body = serde_json::to_string(&request.data).map_err(|e| format!("{e:?}"))?;
        let signature = generate_signature(&self.public_key, &self.secret_key, body.as_bytes());

        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint);
        let flurl = FlUrl::new(&url)
            .set_timeout(self.timeout)
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/json")
            .with_header("Merchant", &self.public_key)
            .with_header("Signature", signature);

        let body_bytes: Option<Vec<u8>> = Some(body.clone().into_bytes());

        let result = if method == Method::POST {
            flurl.post(body_bytes).await
        } else if method == Method::PUT {
            flurl.put(body_bytes).await
        } else if method == Method::GET {
            flurl.get().await
        } else if method == Method::DELETE {
            flurl.delete().await
        } else {
            return Err(format!("unsupported method {method:?}"));
        };

        let resp = result.map_err(|err| {
            format!("FlUrl failed: Url: {url}. Request: {body}. {err:?}")
        })?;

        let status_code = resp.get_status_code();
        let body_bytes = resp
            .receive_body()
            .await
            .map_err(|err| format!("FlUrl failed to receive_body: {err:?}"))?;
        let body_str = String::from_utf8(body_bytes)
            .map_err(|err| format!("Non-utf8 response body: {err:?}"))?;

        if status_code > 299 {
            return Err(format!(
                "Response code: {status_code}. Url: {method:?} {url}. \
                 Request: {body}. Response: {body_str}"
            ));
        }

        Ok(body_str)
    }
}
