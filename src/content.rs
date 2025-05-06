use std::collections::HashMap;

use color_eyre::eyre::{eyre, Result};
use prost::Message;
use ring::aead;
use ring::aead::NonceSequence;
use ring::aead::{BoundKey, Nonce, NONCE_LEN};
use ring::error::Unspecified;
use ring::hkdf;
use xmtp_proto::xmtp::mls::message_contents::{ContentTypeId, EncodedContent};

/// Create a new message according to the xmtp content type
pub fn new_attachment(buffer: &[u8], mime_type: &str, filename: &str) -> Vec<u8> {
    let id = ContentTypeId {
        authority_id: "xmtp.org".into(),
        type_id: "attachment".into(),
        version_major: 1,
        version_minor: 0,
    };
    EncodedContent {
        r#type: Some(id),
        parameters: vec![
            ("mimeType".to_string(), mime_type.to_string()),
            ("filename".to_string(), filename.to_string()),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>(),
        fallback: Some("must say please.".to_string()),
        compression: None,
        content: buffer.to_vec(),
    }
    .encode_to_vec()
}

pub fn new_remote_attachment(mut attachment: Vec<u8>) -> Result<Vec<u8>> {
    let id = ContentTypeId {
        authority_id: "xmtp.org".into(),
        type_id: "remoteStaticContent".into(),
        version_major: 1,
        version_minor: 0,
    };
    let secret = new_secret();
    let mut salt = [0u8; 16]; // 128-bit salt
    rand::fill(&mut salt);
    let nonce = CounterNonceSequence(0).advance().expect("infallible; qed");
    encrypt(attachment.as_mut(), &secret, salt, 0)?;
    Ok(EncodedContent {
        r#type: Some(id),
        parameters: vec![
            kv_str("contentDigest", "todo"),
            kv_bytes("secret", &secret),
            kv_bytes("salt", &salt),
            kv_bytes("nonce", nonce.as_ref()),
            kv_str("scheme", "https://"),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>(),
        fallback: Some("must say please.".to_string()),
        compression: None,
        content: attachment,
    }
    .encode_to_vec())
}

fn kv_bytes(k: &str, v: &[u8]) -> (String, String) {
    (k.to_string(), "0x".to_owned() + &hex::encode(v))
}

fn kv_str(k: &str, v: &str) -> (String, String) {
    (k.to_owned(), v.to_owned())
}

/// Create a new encryption secret
pub fn new_secret() -> Vec<u8> {
    let mut key = vec![0u8; aead::AES_256_GCM.key_len()];
    rand::fill(key.as_mut_slice());
    key
}

/// Create a new salt
pub fn new_salt() -> [u8; 16] {
    let mut salt = [0u8; 16]; // 128-bit salt
    rand::fill(&mut salt);
    salt
}

/// Encrypts buffer in-place
pub fn encrypt(mut buffer: &mut Vec<u8>, secret: &[u8], salt: [u8; 16], nonce: u32) -> Result<()> {
    assert_eq!(secret.len(), aead::AES_256_GCM.key_len());
    // No info parameter
    let hkdf_no_info = &[];
    // Use HKDF to derive the key
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &salt);
    let prk = salt.extract(&secret);
    // Derive the key for AES-256-GCM
    let mut derived_key = [0u8; 32]; // AES-256 needs 32 bytes
    let okm_key = prk
        .expand(hkdf_no_info, hkdf::HKDF_SHA256)
        .map_err(|_| eyre!("HKDF expand failed"))?;
    okm_key
        .fill(&mut derived_key)
        .map_err(|_| eyre!("HKDF fill failed"))?;

    // The derived key can now be used with AES-256-GCM via the ring crate
    let key = aead::UnboundKey::new(&aead::AES_256_GCM, &derived_key)
        .map_err(|_| eyre!("Failed to create unbound key"))?;

    let mut key = aead::SealingKey::new(key, CounterNonceSequence(0));
    let aad = aead::Aad::from(b"~~ super secret cat pic ( 0 _ 0 ) ~~");
    let tag = key
        .seal_in_place_append_tag(aad, buffer)
        .map_err(|_| eyre!("encryption failed"))?;
    Ok(())
}

/// Can probably use timestamp or something more unique here
struct CounterNonceSequence(u32);

impl NonceSequence for CounterNonceSequence {
    // called once for each seal operation
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let mut nonce_bytes = vec![0; NONCE_LEN];

        let bytes = self.0.to_be_bytes();
        nonce_bytes[8..].copy_from_slice(&bytes);

        self.0 += 1; // advance the counter
        Nonce::try_assume_unique_for_key(&nonce_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_message() {
        let mut message = "this should be secret".to_string().into_bytes();
        let preserved = message.clone();
        let secret = new_secret();
        let salt = new_salt();
        encrypt(&mut message, &secret, salt, 0).unwrap();
        println!("message: 0x{}, encrypted: 0x{}", hex::encode(&preserved), hex::encode(&message));
        assert_ne!(message, preserved);
    }
}
