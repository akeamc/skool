use std::{collections::HashMap, sync::Arc};

use cookie_store::{Cookie, CookieStore};
use reqwest::{StatusCode, Url};
use reqwest_cookie_store::CookieStoreRwLock;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::{
    schedule::{get_scope, Scope},
    util::{get_doc, scrape_form},
    Error, Result,
};

/// Skolplattformen session info.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    /// Cookies used in this session.
    pub cookies: Vec<Cookie<'static>>,

    /// Skola24 `X-Scope` header.
    pub scope: Scope,
}

#[instrument(skip(client))]
async fn student_href(client: &reqwest::Client) -> Result<String> {
    let doc = get_doc(client, "https://fnsservicesso1.stockholm.se/sso-ng/saml-2.0/authenticate?customer=https://login001.stockholm.se&targetsystem=TimetableViewer").await?;

    let href = doc
        .find(select::predicate::Class("navBtn"))
        .find(|e| e.inner_html() == "Elever")
        .and_then(|e| e.attr("href"))
        .ok_or(Error::ScrapingFailed {
            details: "no student login button found".into(),
        })?;

    Ok(href.to_owned())
}

#[instrument(skip(client))]
async fn basic_login_url(client: &reqwest::Client) -> Result<Url> {
    let student_href = student_href(client).await?;

    let student_doc = get_doc(
        client,
        format!("https://login001.stockholm.se/siteminderagent/forms/{student_href}"),
    )
    .await?;

    let href = student_doc
        .find(select::predicate::Class("beta"))
        .next()
        .and_then(|e| e.attr("href"))
        .ok_or(Error::ScrapingFailed {
            details: "no username-password option found".into(),
        })?;

    let url = format!("https://login001.stockholm.se/siteminderagent/forms/{href}")
        .parse()
        .map_err(|_| Error::ScrapingFailed {
            details: "invalid basic login url".to_owned(),
        })?;

    Ok(url)
}

#[instrument(skip(client))]
async fn basic_login_form(client: &reqwest::Client) -> Result<HashMap<String, String>> {
    let url = basic_login_url(client).await?;
    let doc = get_doc(client, url).await?;

    scrape_form(doc).ok_or(Error::ScrapingFailed {
        details: "no login form found".into(),
    })
}

#[instrument(skip(client))]
async fn send_login_form(
    client: &reqwest::Client,
    username: &str,
    password: &SecretString,
) -> Result<HashMap<String, String>> {
    let mut form = basic_login_form(client).await?;

    form.insert("user".to_owned(), username.to_owned());
    form.insert("password".to_owned(), password.expose_secret().to_string());
    form.insert("submit".to_owned(), String::new());

    let html = client
        .post("https://login001.stockholm.se/siteminderagent/forms/login.fcc")
        .form(&form)
        .send()
        .await?
        .text()
        .await?;

    let form_body: HashMap<String, String> =
        scrape_form(html.as_str()).ok_or(Error::ScrapingFailed {
            details: "no sso request form found".into(),
        })?;

    Ok(form_body)
}

#[instrument(skip(form, client))]
async fn submit_sso_form(form: &HashMap<String, String>, client: &reqwest::Client) -> Result<()> {
    let res = client
        .post("https://login001.stockholm.se/affwebservices/public/saml2sso")
        .form(form)
        .send()
        .await?;

    if res.status() == StatusCode::BAD_REQUEST {
        debug!("bad credentials");
        return Err(Error::BadCredentials);
    }

    let form = scrape_form(res.text().await?.as_str()).ok_or(Error::ScrapingFailed {
        details: "no sso response form found".into(),
    })?;

    client
        .post("https://fnsservicesso1.stockholm.se/sso-ng/saml-2.0/response")
        .form(&form)
        .send()
        .await?;

    Ok(())
}

#[instrument(skip(client))]
async fn login_client(
    username: &str,
    password: &SecretString,
    client: &reqwest::Client,
) -> Result<()> {
    let form = send_login_form(client, username, password).await?;

    submit_sso_form(&form, client).await?;

    Ok(())
}

/// Start a session.
#[instrument]
pub async fn login(username: &str, password: &SecretString) -> Result<Session> {
    let cookie_store = Arc::new(CookieStoreRwLock::new(CookieStore::default()));

    let client = reqwest::Client::builder()
        .cookie_provider(cookie_store.clone())
        .user_agent(crate::client::USER_AGENT)
        .build()?;

    login_client(username, password, &client).await?;

    let scope = get_scope(&client).await?;

    drop(client);

    let lock = Arc::try_unwrap(cookie_store).expect("lock still has multiple owners");
    let cookie_store = lock.into_inner().expect("rwlock cannot be locked");
    let cookies = cookie_store
        .iter_any()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    debug!("got {} cookies", cookies.len());

    Ok(Session { cookies, scope })
}
