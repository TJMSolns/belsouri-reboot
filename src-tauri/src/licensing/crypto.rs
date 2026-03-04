use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signature, VerifyingKey};
use ed25519_dalek::Verifier;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid license key encoding: {0}")]
    Encoding(String),
    #[error("License key too short — must be at least 64 bytes")]
    TooShort,
    #[error("Signature verification failed")]
    InvalidSignature,
    #[error("Invalid public key")]
    InvalidPublicKey,
}

pub type Result<T> = std::result::Result<T, CryptoError>;

/// The embedded Ed25519 public key (32 bytes, raw).
/// For Track A development this is the DEV keypair public key.
/// In production this is replaced with the License Server's real public key.
///
/// Generated via: `generate_dev_keypair()` test helper below.
/// To regenerate: run `cargo test generate_dev_keypair -- --nocapture --ignored`
pub const EMBEDDED_PUBLIC_KEY: [u8; 32] = [
    // DEV KEY — Track A only. Replace before any production build.
    // Generated: 2026-03-04 via `cargo test generate_dev_keypair -- --nocapture --ignored`
    0xda, 0x6b, 0xe1, 0xde, 0x6f, 0xc0, 0xa0, 0xc7,
    0x33, 0x51, 0x94, 0x80, 0x58, 0xe3, 0xc4, 0x2f,
    0xba, 0x24, 0xc6, 0x93, 0x28, 0x81, 0xad, 0xdf,
    0x5f, 0xf5, 0x1f, 0x90, 0xba, 0x87, 0x6c, 0x2e,
];

/// Decodes a base64url-encoded license key into (payload_bytes, signature_bytes).
/// The license key format is: base64url(payload_json_bytes || signature_bytes)
/// where signature_bytes is always the last 64 bytes.
pub fn decode_license_key(key_str: &str) -> Result<(Vec<u8>, [u8; 64])> {
    let bytes = URL_SAFE_NO_PAD
        .decode(key_str.trim())
        .map_err(|e| CryptoError::Encoding(e.to_string()))?;

    if bytes.len() < 65 {
        return Err(CryptoError::TooShort);
    }

    let split_at = bytes.len() - 64;
    let payload_bytes = bytes[..split_at].to_vec();
    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(&bytes[split_at..]);

    Ok((payload_bytes, sig_bytes))
}

/// Verifies an Ed25519 signature over payload_bytes using the provided public key bytes.
pub fn verify_signature(
    payload_bytes: &[u8],
    signature_bytes: &[u8; 64],
    public_key_bytes: &[u8; 32],
) -> Result<()> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|_| CryptoError::InvalidPublicKey)?;
    let signature = Signature::from_bytes(signature_bytes);
    verifying_key
        .verify(payload_bytes, &signature)
        .map_err(|_| CryptoError::InvalidSignature)
}

/// Convenience: decode and verify a license key in one step.
/// Returns the raw payload bytes on success.
pub fn decode_and_verify(key_str: &str, public_key_bytes: &[u8; 32]) -> Result<Vec<u8>> {
    let (payload_bytes, sig_bytes) = decode_license_key(key_str)?;
    verify_signature(&payload_bytes, &sig_bytes, public_key_bytes)?;
    Ok(payload_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{SigningKey, Signer};
    use rand::rngs::OsRng;

    fn generate_test_keypair() -> (SigningKey, [u8; 32]) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
        (signing_key, public_key_bytes)
    }

    fn sign_payload(signing_key: &SigningKey, payload: &[u8]) -> String {
        let signature = signing_key.sign(payload);
        let mut combined = payload.to_vec();
        combined.extend_from_slice(&signature.to_bytes());
        URL_SAFE_NO_PAD.encode(&combined)
    }

    #[test]
    fn test_decode_valid_license_key() {
        let (signing_key, _) = generate_test_keypair();
        let payload = b"test payload";
        let key_str = sign_payload(&signing_key, payload);
        let (decoded_payload, _sig) = decode_license_key(&key_str).unwrap();
        assert_eq!(decoded_payload, payload);
    }

    #[test]
    fn test_decode_too_short() {
        // 63 bytes of base64url encodes to <65 bytes decoded
        let short = URL_SAFE_NO_PAD.encode(&[0u8; 63]);
        let err = decode_license_key(&short).unwrap_err();
        assert!(matches!(err, CryptoError::TooShort));
    }

    #[test]
    fn test_decode_invalid_base64() {
        let err = decode_license_key("not!valid!base64!!!").unwrap_err();
        assert!(matches!(err, CryptoError::Encoding(_)));
    }

    #[test]
    fn test_verify_valid_signature() {
        let (signing_key, pub_key_bytes) = generate_test_keypair();
        let payload = br#"{"schema_version":2,"license_type":"eval"}"#;
        let key_str = sign_payload(&signing_key, payload);
        let (decoded_payload, sig_bytes) = decode_license_key(&key_str).unwrap();
        assert!(verify_signature(&decoded_payload, &sig_bytes, &pub_key_bytes).is_ok());
    }

    #[test]
    fn test_verify_tampered_payload() {
        let (signing_key, pub_key_bytes) = generate_test_keypair();
        let payload = br#"{"schema_version":2,"license_type":"eval"}"#;
        let key_str = sign_payload(&signing_key, payload);
        let (mut decoded_payload, sig_bytes) = decode_license_key(&key_str).unwrap();
        decoded_payload[0] ^= 0xFF; // flip bits in first byte
        let err = verify_signature(&decoded_payload, &sig_bytes, &pub_key_bytes).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidSignature));
    }

    #[test]
    fn test_verify_wrong_public_key() {
        let (signing_key, _) = generate_test_keypair();
        let (_, wrong_pub_key) = generate_test_keypair();
        let payload = b"some payload";
        let key_str = sign_payload(&signing_key, payload);
        let (decoded_payload, sig_bytes) = decode_license_key(&key_str).unwrap();
        let err = verify_signature(&decoded_payload, &sig_bytes, &wrong_pub_key).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidSignature));
    }

    #[test]
    fn test_decode_and_verify_convenience() {
        let (signing_key, pub_key_bytes) = generate_test_keypair();
        let payload = b"convenience test payload";
        let key_str = sign_payload(&signing_key, payload);
        let result = decode_and_verify(&key_str, &pub_key_bytes).unwrap();
        assert_eq!(result, payload);
    }

    /// Run this once to generate the dev keypair for EMBEDDED_PUBLIC_KEY and EVAL_TOKEN.
    /// cargo test generate_dev_keypair -- --nocapture --ignored
    #[test]
    #[ignore]
    fn generate_dev_keypair() {
        let (signing_key, pub_key_bytes) = generate_test_keypair();

        println!("\n=== DEV KEYPAIR (Track A only) ===");
        print!("EMBEDDED_PUBLIC_KEY bytes: [");
        for (i, b) in pub_key_bytes.iter().enumerate() {
            if i % 8 == 0 { print!("\n    "); }
            print!("0x{:02x}, ", b);
        }
        println!("\n]");

        // Sign a dev eval payload
        let payload = serde_json::json!({
            "schema_version": 2,
            "license_type": "eval",
            "practice_id": null,
            "issued_at": "2026-01-01T00:00:00Z",
            "max_duration_days": 30,
            "modules": [{"name": "scheduling", "grace_period_days": 90}]
        });
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let key_str = sign_payload(&signing_key, &payload_bytes);
        println!("EVAL_TOKEN: {}", key_str);
        println!("=================================\n");
    }
}
