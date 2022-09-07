use aes_gcm_siv::{Aes256GcmSiv, Key};
use hex::FromHexError;

pub mod crypt;
pub mod error;
pub mod routes;

#[derive(clap::Parser)]
pub struct Config {
    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env, default_value = "10")]
    pub max_database_connections: u32,

    #[clap(env, parse(try_from_str = parse_hex_key))]
    pub aes_key: Key<Aes256GcmSiv>,
}

fn parse_hex_key(s: &str) -> Result<Key<Aes256GcmSiv>, FromHexError> {
    let mut data = [0; 32];
    hex::decode_to_slice(s, &mut data)?;
    Ok(*Key::<Aes256GcmSiv>::from_slice(&data))
}
