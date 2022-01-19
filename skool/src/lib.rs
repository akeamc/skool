use skool_cookie::crypto::Key;

pub mod error;
pub mod extractor;
pub mod routes;

#[derive(Debug, Clone, Copy)]
pub struct WebhookConfig {
    pub key: Key,
}
