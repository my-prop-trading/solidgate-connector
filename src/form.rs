//! Payment form: the data the SolidGate form SDK is initialized with.
//!
//! The payment intent is a JSON object encrypted with AES-256-CBC under the first 32 characters
//! of the secret key, the random IV put in front, and the result base64 encoded with `-_` for
//! `+/`. The signature is the API request signature over that encoded text.

use crate::generate_signature;
use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

const KEY_LEN: usize = 32;
const IV_LEN: usize = 16;

/// What the customer pays in the form. With `product_id` SolidGate takes the price from the
/// product, and `customer_account_id` is required.
#[derive(Debug, Serialize)]
pub struct PaymentIntent {
    pub order_id: String,
    pub order_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    /// Smallest currency unit; left out with a product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    pub customer_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_account_id: Option<String>,
    pub ip_address: String,
    pub platform: crate::card::Platform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_url: Option<String>,
}

/// The `merchantData` of the form SDK.
#[derive(Debug, Clone, Serialize)]
pub struct PaymentFormData {
    pub merchant: String,
    pub payment_intent: String,
    pub signature: String,
}

/// Encrypts `intent` and signs it; fails on a secret key shorter than the AES key.
pub fn payment_form_data(
    public_key: &str,
    secret_key: &str,
    intent: &PaymentIntent,
) -> Result<PaymentFormData, String> {
    let mut iv = [0u8; IV_LEN];
    getrandom::getrandom(&mut iv).map_err(|err| format!("no random IV: {err}"))?;
    let json = serde_json::to_vec(intent).map_err(|err| format!("{err:?}"))?;
    let payment_intent = encrypt(secret_key, &iv, &json)?;
    Ok(PaymentFormData {
        merchant: public_key.to_string(),
        signature: generate_signature(public_key, secret_key, payment_intent.as_bytes()),
        payment_intent,
    })
}

fn encrypt(secret_key: &str, iv: &[u8; IV_LEN], data: &[u8]) -> Result<String, String> {
    let key = secret_key
        .as_bytes()
        .get(..KEY_LEN)
        .ok_or("secret key is shorter than the AES key")?;
    let encrypted = Aes256CbcEnc::new(key.into(), iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data);
    let mut raw = iv.to_vec();
    raw.extend_from_slice(&encrypted);
    Ok(general_purpose::STANDARD
        .encode(raw)
        .replace('+', "-")
        .replace('/', "_"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Made with `openssl enc -aes-256-cbc -K <hex of KEY[..32]> -iv 000102…0f` over `PLAIN`,
    /// the IV put in front, base64 with `-_`.
    const KEY: &str = "api_sk_0123456789abcdef0123456789abcdef";
    const PLAIN: &[u8] = br#"{"order_id":"academy-1"}"#;
    const EXPECTED: &str = include_str!("../tests/form_vector.txt");

    #[test]
    fn encryption_matches_openssl() {
        let iv: [u8; IV_LEN] = core::array::from_fn(|i| i as u8);
        assert_eq!(encrypt(KEY, &iv, PLAIN).as_deref(), Ok(EXPECTED.trim()));
    }

    #[test]
    fn short_secret_key_is_refused() {
        let iv = [0u8; IV_LEN];
        assert!(encrypt("short", &iv, PLAIN).is_err());
    }

    #[test]
    fn form_data_is_signed_over_the_intent() {
        let intent = PaymentIntent {
            order_id: "academy-1".to_string(),
            order_description: "AI assistant".to_string(),
            product_id: Some("p1".to_string()),
            amount: None,
            currency: None,
            customer_email: "a@b.c".to_string(),
            customer_account_id: Some("c1".to_string()),
            ip_address: "203.0.113.0".to_string(),
            platform: crate::card::Platform::Web,
            success_url: None,
            fail_url: None,
        };
        let data = payment_form_data("api_pk_1", KEY, &intent).expect("the key is long enough");
        assert_eq!(data.merchant, "api_pk_1");
        assert_eq!(
            data.signature,
            generate_signature("api_pk_1", KEY, data.payment_intent.as_bytes())
        );
    }
}
