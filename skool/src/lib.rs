use aes_gcm_siv::{Aes256GcmSiv, Key};
use error::AppError;
use hex::FromHexError;
use sentry::types::Dsn;
use sqlx::Postgres;

pub mod credentials;
pub mod crypt;
pub mod error;
pub mod routes;
pub mod session;
pub mod share;
mod util;

pub type Result<T, E = AppError> = core::result::Result<T, E>;

#[derive(clap::Parser)]
pub struct Config {
    #[clap(env)]
    pub database_url: String,

    #[clap(long, env, default_value = "10")]
    pub max_database_connections: u32,

    #[clap(env)]
    pub redis_url: String,

    #[clap(env, parse(try_from_str = parse_hex_key))]
    pub aes_key: Key<Aes256GcmSiv>,

    #[clap(env)]
    pub sentry_dsn: Option<Dsn>,

    #[clap(env)]
    pub sentry_environment: Option<String>,
}

fn parse_hex_key(s: &str) -> Result<Key<Aes256GcmSiv>, FromHexError> {
    let mut data = [0; 32];
    hex::decode_to_slice(s, &mut data)?;
    Ok(*Key::<Aes256GcmSiv>::from_slice(&data))
}

pub struct ApiContext {
    pub postgres: sqlx::Pool<Postgres>,
    pub redis: deadpool_redis::Pool,
    pub config: Config,
}

impl ApiContext {
    pub fn aes_key(&self) -> &Key<Aes256GcmSiv> {
        &self.config.aes_key
    }
}
