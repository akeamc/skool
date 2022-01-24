use skool_crypto::crypto::Key;
use structopt::StructOpt;

pub mod error;
pub mod routes;

#[derive(Debug, Clone, Copy, StructOpt)]
pub struct WebhookConfig {
    #[structopt(name = "webhook-key", env = "WEBHOOK_KEY", hide_env_values = true)]
    pub key: Key,
}
