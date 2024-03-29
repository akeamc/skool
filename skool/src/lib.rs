use aes_gcm_siv::{Aes256GcmSiv, Key};
use error::AppError;
use hex::FromHexError;
use sqlx::PgPool;

pub mod class;
pub mod credentials;
pub mod crypt;
pub mod error;
pub mod routes;
pub mod session;
pub mod share;
pub mod skolplattformen;
mod util;

pub type Result<T, E = AppError> = core::result::Result<T, E>;

#[derive(clap::Parser, Clone)]
pub struct Config {
    #[clap(env)]
    pub database_url: String,

    #[clap(long, env, default_value = "10")]
    pub max_database_connections: u32,

    #[clap(env)]
    pub redis_url: String,

    #[clap(env, parse(try_from_str = parse_hex_key))]
    pub aes_key: Key<Aes256GcmSiv>,

    #[clap(long, env, default_value = "http://localhost:4317")]
    pub otlp_endpoint: String,
}

fn parse_hex_key(s: &str) -> Result<Key<Aes256GcmSiv>, FromHexError> {
    let mut data = [0; 32];
    hex::decode_to_slice(s, &mut data)?;
    Ok(*Key::<Aes256GcmSiv>::from_slice(&data))
}

#[derive(Clone)]
pub struct AppState {
    pub postgres: PgPool,
    pub redis: deadpool_redis::Pool,
    pub config: Config,
}

impl AppState {
    pub fn aes_key(&self) -> &Key<Aes256GcmSiv> {
        &self.config.aes_key
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum System {
    Skolplattformen = 0,
}

impl System {
    const fn as_u8(&self) -> u8 {
        *self as _
    }
}
