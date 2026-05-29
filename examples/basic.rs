use solidgate_connector::api::SolidGateApi;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let _api = SolidGateApi::new(
        "https://api-sandbox.solidgate.com",
        std::env::var("SOLIDGATE_MERCHANT_ID").unwrap_or_default(),
        std::env::var("SOLIDGATE_PUBLIC_KEY").unwrap_or_default(),
        std::env::var("SOLIDGATE_SECRET_KEY").unwrap_or_default(),
        Duration::from_secs(15),
    );

    println!("solidgate-connector: scaffold ready, awaiting docs");
}
