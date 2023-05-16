use std::sync::Arc;

use cookie_store::CookieStore;
use reqwest::header::HeaderMap;

use crate::Session;

/// A wrapper around [`reqwest::Client`] that prevents unauthorized clients
/// from accidentaly being passed to Skolplattformen functions.
#[derive(Debug, Clone)]
pub struct Client(pub(crate) reqwest::Client);

impl Client {
    /// Intialize a client from a stored session.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying [`reqwest::Client`] initialization fails.
    #[allow(clippy::missing_panics_doc)] // function won't panic
    pub fn new(session: Session) -> reqwest::Result<Self> {
        // the only way from_cookies() can be Err is if the iterator yields an Err, which it doesn't do
        let cookie_store =
            CookieStore::from_cookies(session.cookies.into_iter().map(Ok::<_, ()>), true).unwrap();
        let cookie_store = Arc::new(reqwest_cookie_store::CookieStoreRwLock::new(cookie_store));

        let mut headers = HeaderMap::new();

        headers.insert("X-Scope", session.scope.into_inner());

        let client = reqwest::Client::builder()
            .cookie_provider(cookie_store)
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()?;

        Ok(Self(client))
    }
}

/// User agent used by the client 🥸
pub const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:95.0) Gecko/20100101 Firefox/95.0";
