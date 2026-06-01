use solidgate_connector::api::SolidGateApi;
use solidgate_connector::model::{ApiRequest, CancelSubscriptionCode, CancelSubscriptionRequest};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let api = SolidGateApi::new(
        "https://subscriptions.solidgate.com/api/v1",
        "https://api.solidgate.com/api/v1",
        std::env::var("SOLIDGATE_PUBLIC_KEY").unwrap_or_default(),
        std::env::var("SOLIDGATE_SECRET_KEY").unwrap_or_default(),
        Duration::from_secs(15),
    );

    let subscription_id = std::env::var("SOLIDGATE_SUBSCRIPTION_ID").unwrap_or_default();
    if subscription_id.is_empty() {
        println!("Set SOLIDGATE_PUBLIC_KEY/SECRET_KEY/SUBSCRIPTION_ID env vars to run a real cancel");
        return;
    }

    let result = api
        .cancel_subscription(&ApiRequest {
            data: CancelSubscriptionRequest {
                subscription_id,
                force: false,
                cancel_code: CancelSubscriptionCode::CustomerRequest,
            },
        })
        .await;

    println!("cancel_subscription -> {result:?}");
}
