use std::sync::Arc;

use cookie_store::CookieStore;
use lazy_static::lazy_static;
use reqwest::{Client, StatusCode};
use reqwest_cookie_store::CookieStoreMutex;
use scrape::scrape_form;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use skool_cookie::Cookie;
use thiserror::Error;

use crate::{util::get_html, USER_AGENT};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("bad credentials")]
    BadCredentials,

    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Debug, Serialize, Deserialize, Cookie)]
#[cookie_name("session")]
pub struct Session {
    pub cookie_store: CookieStore,
}

impl Session {
    pub fn into_client(self) -> Result<Client, reqwest::Error> {
        let cookie_store = Arc::new(reqwest_cookie_store::CookieStoreMutex::new(
            self.cookie_store,
        ));

        Client::builder()
            .cookie_provider(cookie_store)
            .user_agent(USER_AGENT)
            .build()
    }
}

async fn fill_jar_with_session_data(
    username: &str,
    password: &str,
    cookie_store: Arc<CookieStoreMutex>,
) -> Result<(), AuthError> {
    lazy_static! {
        static ref A_NAVBTN: Selector = Selector::parse("a.navBtn").unwrap();
        static ref A_BETA: Selector = Selector::parse("a.beta").unwrap();
    }

    let client = Client::builder()
        .cookie_provider(cookie_store.clone())
        .user_agent(USER_AGENT)
        .build()?;

    let doc = get_html(&client, "https://fnsservicesso1.stockholm.se/sso-ng/saml-2.0/authenticate?customer=https://login001.stockholm.se").await?;

    let student_href = doc
        .select(&A_NAVBTN)
        .find(|e| e.inner_html() == "Elever")
        .map(|e| e.value().attr("href"))
        .flatten()
        .unwrap();

    let student_doc = get_html(
        &client,
        format!(
            "https://login001.stockholm.se/siteminderagent/forms/{}",
            student_href
        ),
    )
    .await?;

    let username_password_href = student_doc
        .select(&A_BETA)
        .next()
        .map(|e| e.value().attr("href"))
        .flatten()
        .unwrap();

    let doc = get_html(
        &client,
        format!(
            "https://login001.stockholm.se/siteminderagent/forms/{}",
            username_password_href
        ),
    )
    .await?;

    let mut form_body = scrape_form(&doc).unwrap();

    form_body.insert("user".to_owned(), username.to_owned());
    form_body.insert("password".to_owned(), password.to_owned());
    form_body.insert("submit".to_owned(), "".to_owned());

    let res = client
        .post("https://login001.stockholm.se/siteminderagent/forms/login.fcc")
        .form(&form_body)
        .send()
        .await?;

    let form_body = scrape_form(&Html::parse_document(&res.text().await?)).unwrap();

    let res = client
        .post("https://login001.stockholm.se/affwebservices/public/saml2sso")
        .form(&form_body)
        .send()
        .await?;

    if res.status() == StatusCode::BAD_REQUEST {
        return Err(AuthError::BadCredentials);
    }

    let doc = Html::parse_document(&res.text().await?);

    let form_body = scrape_form(&doc).unwrap();

    client
        .post("https://fnsservicesso1.stockholm.se/sso-ng/saml-2.0/response")
        .form(&form_body)
        .send()
        .await?;

    Ok(())
}

pub async fn start_session(username: &str, password: &str) -> Result<Session, AuthError> {
    let cookie_store = CookieStore::default();
    let cookie_store = Arc::new(reqwest_cookie_store::CookieStoreMutex::new(cookie_store));

    fill_jar_with_session_data(username, password, cookie_store.clone()).await?;

    let lock = Arc::try_unwrap(cookie_store).expect("lock still has multiple owners");
    let cookie_store = lock.into_inner().expect("mutex cannot be locked");

    Ok(Session { cookie_store })
}
