use actix_web::{dev::ServiceRequest, web::Data, HttpRequest};
use crypto::Key;

use structopt::StructOpt;

pub mod crypto;

pub fn crypto_config(req: &impl UsableRequest) -> &CryptoConfig {
    req.app_data::<Data<CryptoConfig>>()
        .expect("CryptoConfig not found")
}

#[derive(Debug, Clone, StructOpt)]
pub struct CryptoConfig {
    #[structopt(name = "crypto-key", env = "CRYPTO_KEY", hide_env_values = true)]
    pub key: Key,
}

pub trait UsableRequest {
    fn app_data<T: 'static>(&self) -> Option<&T>;
}

impl UsableRequest for HttpRequest {
    fn app_data<T: 'static>(&self) -> Option<&T> {
        self.app_data()
    }
}

impl UsableRequest for ServiceRequest {
    fn app_data<T: 'static>(&self) -> Option<&T> {
        self.app_data()
    }
}
