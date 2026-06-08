use crate::generate_signature;
use crate::model::{
    ApiRequest, ApiResponse, CancelSubscriptionRequest, CancelSubscriptionResponse,
    CreateWebhookEndpointRequest, ListWebhookEndpointsResponse, UpdateWebhookEndpointRequest,
    WebhookEndpoint,
};
use flurl::{hyper::Method, FlUrl};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::time::Duration;

/// SolidGate production subscriptions host (cancel / subscription endpoints).
pub const SUBSCRIPTIONS_API_URL: &str = "https://subscriptions.solidgate.com/api/v1";
/// SolidGate production general API host (webhook-endpoint management).
pub const GENERAL_API_URL: &str = "https://api.solidgate.com/api/v1";

pub struct SolidGateApi {
    /// Subscriptions host: `https://subscriptions.solidgate.com/api/v1`.
    base_url: String,
    /// General API host: `https://api.solidgate.com/api/v1` (webhook management lives here).
    general_api_url: String,
    public_key: String,
    secret_key: String,
    timeout: Duration,
}

impl SolidGateApi {
    /// Build a client against SolidGate's production hosts ([`SUBSCRIPTIONS_API_URL`] +
    /// [`GENERAL_API_URL`]). These hosts are fixed — sandbox vs live is selected by the
    /// public/secret key pair, not the URL. Use [`Self::with_urls`] only to override (tests).
    pub fn new(
        public_key: impl Into<String>,
        secret_key: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self::with_urls(
            SUBSCRIPTIONS_API_URL,
            GENERAL_API_URL,
            public_key,
            secret_key,
            timeout,
        )
    }

    /// * `base_url` — subscriptions host, e.g. `https://subscriptions.solidgate.com/api/v1`.
    /// * `general_api_url` — general API host, e.g. `https://api.solidgate.com/api/v1`
    ///   (webhook-endpoint management is served from here, not from the subscriptions host).
    pub fn with_urls(
        base_url: impl Into<String>,
        general_api_url: impl Into<String>,
        public_key: impl Into<String>,
        secret_key: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            general_api_url: general_api_url.into(),
            public_key: public_key.into(),
            secret_key: secret_key.into(),
            timeout,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn general_api_url(&self) -> &str {
        &self.general_api_url
    }

    /// Cancel a subscription. Pass `force=false` to cancel at the end of the billing period
    /// (subscription stays active until `expired_at`), `true` for immediate termination.
    pub async fn cancel_subscription(
        &self,
        req: &ApiRequest<CancelSubscriptionRequest>,
    ) -> Result<ApiResponse<CancelSubscriptionResponse>, String> {
        let body = serde_json::to_string(&req.data).map_err(|e| format!("{e:?}"))?;
        self.send_signed(
            &self.base_url,
            "subscription/cancel",
            Method::POST,
            &[],
            Some(body),
        )
        .await
    }

    /// Register a new webhook endpoint on SolidGate's side.
    pub async fn create_webhook_endpoint(
        &self,
        req: &CreateWebhookEndpointRequest,
    ) -> Result<WebhookEndpoint, String> {
        let body = serde_json::to_string(req).map_err(|e| format!("{e:?}"))?;
        self.send_signed(
            &self.general_api_url,
            "webhooks/endpoints",
            Method::POST,
            &[],
            Some(body),
        )
        .await
    }

    /// List webhook endpoints. All args are optional filters.
    pub async fn list_webhook_endpoints(
        &self,
        filter_id: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<ListWebhookEndpointsResponse, String> {
        let mut query: Vec<(&'static str, String)> = Vec::new();
        if let Some(id) = filter_id {
            query.push(("filter[id]", id.to_string()));
        }
        if let Some(limit) = limit {
            query.push(("pagination[limit]", limit.to_string()));
        }
        if let Some(offset) = offset {
            query.push(("pagination[offset]", offset.to_string()));
        }
        self.send_signed(
            &self.general_api_url,
            "webhooks/endpoints",
            Method::GET,
            &query,
            None,
        )
        .await
    }

    /// Update a webhook endpoint (PATCH — only the populated fields are changed).
    pub async fn update_webhook_endpoint(
        &self,
        id: &str,
        req: &UpdateWebhookEndpointRequest,
    ) -> Result<WebhookEndpoint, String> {
        let body = serde_json::to_string(req).map_err(|e| format!("{e:?}"))?;
        let path = format!("webhooks/endpoints/{id}");
        self.send_signed(&self.general_api_url, &path, Method::PATCH, &[], Some(body))
            .await
    }

    /// Delete a webhook endpoint.
    pub async fn delete_webhook_endpoint(&self, id: &str) -> Result<(), String> {
        let path = format!("webhooks/endpoints/{id}");
        self.send_signed_raw(&self.general_api_url, &path, Method::DELETE, &[], None)
            .await?;
        Ok(())
    }

    /// Send a signed request and deserialize the JSON response into `T`.
    async fn send_signed<T: DeserializeOwned + Debug>(
        &self,
        base_url: &str,
        path: &str,
        method: Method,
        query: &[(&'static str, String)],
        body: Option<String>,
    ) -> Result<T, String> {
        let response = self
            .send_signed_raw(base_url, path, method.clone(), query, body)
            .await?;
        serde_json::from_str(&response).map_err(|err| {
            format!("Failed to deserialize: {err:?}. Url: {method:?} {base_url}/{path}. Body: {response}")
        })
    }

    /// Send a signed request and return the raw response body.
    ///
    /// The HMAC signature is computed over the request body bytes (empty for GET/DELETE),
    /// matching SolidGate's signing scheme. Query params are not part of the signature.
    async fn send_signed_raw(
        &self,
        base_url: &str,
        path: &str,
        method: Method,
        query: &[(&'static str, String)],
        body: Option<String>,
    ) -> Result<String, String> {
        let body = body.unwrap_or_default();
        let signature = generate_signature(&self.public_key, &self.secret_key, body.as_bytes());

        let url = format!("{}/{}", base_url.trim_end_matches('/'), path);
        let mut flurl = FlUrl::new(&url)
            .set_timeout(self.timeout)
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/json")
            .with_header("Merchant", &self.public_key)
            .with_header("Signature", signature);

        for (name, value) in query {
            flurl = flurl.append_query_param(*name, Some(value.as_str()));
        }

        let body_bytes: Option<Vec<u8>> = if body.is_empty() {
            None
        } else {
            Some(body.clone().into_bytes())
        };

        let result = match method {
            Method::POST => flurl.post(body_bytes).await,
            Method::PUT => flurl.put(body_bytes).await,
            Method::PATCH => flurl.patch(body_bytes).await,
            Method::GET => flurl.get().await,
            Method::DELETE => flurl.delete().await,
            other => return Err(format!("unsupported method {other:?}")),
        };

        let resp = result.map_err(|err| format!("FlUrl failed: Url: {url}. Request: {body}. {err:?}"))?;

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
