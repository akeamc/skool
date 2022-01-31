use actix_web::{dev::ServiceRequest, web::Data, HttpRequest};
use crypto::AesKey;

use structopt::StructOpt;

pub mod crypto;

pub fn crypto_config(req: &impl UsableRequest) -> &WebtokenConfig {
    req.app_data::<Data<WebtokenConfig>>()
        .expect("webtoken config not found")
}

#[derive(Debug, Clone, StructOpt)]
pub struct WebtokenConfig {
    #[structopt(name = "webtoken-key", env = "WEBTOKEN_KEY", hide_env_values = true)]
    pub key: AesKey,
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
