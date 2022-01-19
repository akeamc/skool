use skool_cookie::crypto::Key;
use structopt::StructOpt;

pub mod error;
pub mod extractor;
pub mod routes;

#[derive(Debug, Clone, Copy, StructOpt)]
pub struct WebhookConfig {
    #[structopt(name = "webhook-key", env = "WEBHOOK_KEY", hide_env_values = true)]
    pub key: Key,
}
