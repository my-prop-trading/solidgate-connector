pub mod api;
pub mod model;
pub mod subscription;
pub mod webhook;

use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha512;

type HmacSha512 = Hmac<Sha512>;

/// HMAC-SHA512 signature used by SolidGate for both API requests and webhook callbacks.
///
/// Algorithm: HMAC-SHA512(public_key + data + public_key, secret_key) → hex → base64.
/// Reference (PHP SDK): `base64_encode(hash_hmac('sha512', $pk.$data.$pk, $sk))`
/// `hash_hmac` returns a lowercase hex string by default — that string is then base64-encoded.
pub fn generate_signature(public_key: &str, secret_key: &str, data: &[u8]) -> String {
    let mut payload = Vec::with_capacity(public_key.len() * 2 + data.len());
    payload.extend_from_slice(public_key.as_bytes());
    payload.extend_from_slice(data);
    payload.extend_from_slice(public_key.as_bytes());

    let mut mac = HmacSha512::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(&payload);
    let hex_string = hex::encode(mac.finalize().into_bytes());

    general_purpose::STANDARD.encode(hex_string.as_bytes())
}

/// Constant-time comparison of two base64 signatures.
pub fn verify_signature(public_key: &str, secret_key: &str, data: &[u8], received: &str) -> bool {
    let expected = generate_signature(public_key, secret_key, data);
    // length first — avoids leaking via early-out on common-prefix
    if expected.len() != received.len() {
        return false;
    }
    expected
        .as_bytes()
        .iter()
        .zip(received.as_bytes().iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_is_deterministic() {
        let a = generate_signature("merchant", "secret", b"payload");
        let b = generate_signature("merchant", "secret", b"payload");
        assert_eq!(a, b);
    }

    #[test]
    fn signature_differs_for_different_inputs() {
        let a = generate_signature("merchant", "secret", b"payload1");
        let b = generate_signature("merchant", "secret", b"payload2");
        assert_ne!(a, b);
    }

    #[test]
    fn verify_accepts_matching_signature() {
        let sig = generate_signature("merchant", "secret", b"body");
        assert!(verify_signature("merchant", "secret", b"body", &sig));
    }

    #[test]
    fn verify_rejects_tampered_body() {
        let sig = generate_signature("merchant", "secret", b"body");
        assert!(!verify_signature("merchant", "secret", b"different", &sig));
    }
}
