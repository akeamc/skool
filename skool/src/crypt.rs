use aes_gcm_siv::aead::Aead;
use aes_gcm_siv::{Key, KeyInit};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("encoding failed")]
    Encode(#[from] rmp_serde::encode::Error),

    #[error("decoding failed")]
    Decode(#[from] rmp_serde::decode::Error),

    #[error("aes error")]
    Aes,

    #[error("ciphertext too short")]
    CiphertextTooShort,
}

impl From<aes_gcm_siv::aead::Error> for Error {
    fn from(_: aes_gcm_siv::aead::Error) -> Self {
        Self::Aes
    }
}

const NONCE_LEN: usize = 12;

pub fn encrypt_bytes(val: &impl Serialize, key: &Key<Aes256GcmSiv>) -> Result<Vec<u8>, Error> {
    let plaintext = rmp_serde::to_vec(val)?;
    let mut nonce = [0_u8; NONCE_LEN];
    rand::thread_rng().fill(&mut nonce);

    let cipher = Aes256GcmSiv::new(key);
    let mut out = cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())?;
    out.extend_from_slice(&nonce);

    Ok(out)
}

pub fn decrypt_bytes<T: DeserializeOwned>(
    bytes: &[u8],
    key: &Key<Aes256GcmSiv>,
) -> Result<T, Error> {
    if bytes.len() < NONCE_LEN {
        // not handling this would cause panic
        return Err(Error::CiphertextTooShort);
    }

    let (ciphertext, nonce) = bytes.split_at(bytes.len() - NONCE_LEN);
    let cipher = Aes256GcmSiv::new(key);

    let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)?;
    rmp_serde::from_slice(&plaintext).map_err(|e| e.into())
}
