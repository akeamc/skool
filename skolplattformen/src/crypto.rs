use aes_gcm_siv::aead::{Aead, NewAead};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

use crate::ascii85;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encoding failed")]
    Encode(#[from] rmp_serde::encode::Error),

    #[error("decoding failed")]
    Decode(#[from] rmp_serde::decode::Error),

    #[error("aes error")]
    Aes,

    #[error("ascii85 error")]
    Ascii85(#[from] ascii85::Ascii85Error),

    #[error("ciphertext too short")]
    CiphertextTooShort,
}

impl From<aes_gcm_siv::aead::Error> for CryptoError {
    fn from(_: aes_gcm_siv::aead::Error) -> Self {
        Self::Aes
    }
}

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
type Key = [u8; KEY_LEN];

fn encrypt_bytes(val: &impl Serialize, key: &Key) -> Result<Vec<u8>, CryptoError> {
    let plaintext = rmp_serde::to_vec(val)?;
    let mut nonce = [0_u8; NONCE_LEN];
    rand::thread_rng().fill(&mut nonce);

    let cipher = Aes256GcmSiv::new(aes_gcm_siv::Key::from_slice(key));
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
    let cipher = Aes256GcmSiv::new(aes_gcm_siv::Key::from_slice(key));

    let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)?;
    rmp_serde::from_read_ref(&plaintext).map_err(|e| e.into())
}

pub fn encrypt(val: &impl Serialize, key: &Key) -> Result<String, CryptoError> {
    encrypt_bytes(val, key).map(|v| ascii85::encode(&v))
}

pub fn decrypt<T>(val: &str, key: &Key) -> Result<T, CryptoError>
where
    T: DeserializeOwned,
{
    let ciphertext = ascii85::decode(val)?;
    decrypt_bytes(&ciphertext, key)
}
