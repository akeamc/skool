use std::str::FromStr;

use aes_gcm_siv::aead::{Aead, NewAead};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use base64::URL_SAFE_NO_PAD;
use hex::FromHexError;
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encoding failed")]
    Encode(#[from] rmp_serde::encode::Error),

    #[error("decoding failed")]
    Decode(#[from] rmp_serde::decode::Error),

    #[error("aes error")]
    Aes,

    #[error("base64 error")]
    Base64(#[from] base64::DecodeError),

    #[error("ciphertext too short")]
    CiphertextTooShort,
}

impl From<aes_gcm_siv::aead::Error> for CryptoError {
    fn from(_: aes_gcm_siv::aead::Error) -> Self {
        Self::Aes
    }
}

const NONCE_LEN: usize = 12;

#[derive(Clone, Copy, Debug)]
pub struct Key([u8; Self::LEN]);

impl Key {
    pub const LEN: usize = 32;
}

fn encrypt_bytes(val: &impl Serialize, key: &Key) -> Result<Vec<u8>, CryptoError> {
    let plaintext = rmp_serde::to_vec(val)?;
    let mut nonce = [0_u8; NONCE_LEN];
    rand::thread_rng().fill(&mut nonce);

    let cipher = Aes256GcmSiv::new(aes_gcm_siv::Key::from_slice(&key.0));
    let mut out = cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())?;
    out.extend_from_slice(&nonce);

    Ok(out)
}

fn decrypt_bytes<T>(bytes: &[u8], key: &Key) -> Result<T, CryptoError>
where
    T: DeserializeOwned,
{
    if bytes.len() < NONCE_LEN {
        // not handling this would cause panic
        return Err(CryptoError::CiphertextTooShort);
    }

    let (ciphertext, nonce) = bytes.split_at(bytes.len() - NONCE_LEN);
    let cipher = Aes256GcmSiv::new(aes_gcm_siv::Key::from_slice(&key.0));

    let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)?;
    rmp_serde::from_read_ref(&plaintext).map_err(|e| e.into())
}

pub fn encrypt(val: &impl Serialize, key: &Key) -> Result<String, CryptoError> {
    encrypt_bytes(val, key).map(|v| base64::encode_config(&v, URL_SAFE_NO_PAD))
}

pub fn decrypt<T>(val: &str, key: &Key) -> Result<T, CryptoError>
where
    T: DeserializeOwned,
{
    let ciphertext = base64::decode_config(val, URL_SAFE_NO_PAD)?;
    decrypt_bytes(&ciphertext, key)
}

impl FromStr for Key {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = [0_u8; Key::LEN];

        hex::decode_to_slice(s, &mut data)?;

        Ok(Key(data))
    }
}
