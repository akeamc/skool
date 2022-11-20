use std::sync::Arc;

use cookie_store::{Cookie, CookieStore};
use reqwest::StatusCode;
use reqwest_cookie_store::CookieStoreMutex;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, trace};

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

#[instrument(skip(password, client))]
async fn login_client(username: &str, password: &str, client: &reqwest::Client) -> Result<()> {
    trace!("GETting login page");

    let doc = get_doc(client, "https://fnsservicesso1.stockholm.se/sso-ng/saml-2.0/authenticate?customer=https://login001.stockholm.se&targetsystem=TimetableViewer").await?;

    let student_href = doc
        .find(select::predicate::Class("navBtn"))
        .find(|e| e.inner_html() == "Elever")
        .and_then(|e| e.attr("href"))
        .ok_or(Error::ScrapingFailed {
            details: "no student login button found".into(),
        })?;

    let student_login_url = format!(
        "https://login001.stockholm.se/siteminderagent/forms/{}",
        student_href
    );

    trace!(student_login_url = student_login_url.as_str());

    let student_doc = get_doc(client, &student_login_url).await?;

    let username_password_href = student_doc
        .find(select::predicate::Class("beta"))
        .next()
        .and_then(|e| e.attr("href"))
        .ok_or(Error::ScrapingFailed {
            details: "no username-password option found".into(),
        })?;

    let doc = get_doc(
        client,
        format!(
            "https://login001.stockholm.se/siteminderagent/forms/{}",
            username_password_href
        ),
    )
    .await?;

    let mut form_body = scrape_form(doc).ok_or(Error::ScrapingFailed {
        details: "no login form found".into(),
    })?;

    form_body.insert("user".to_owned(), username.to_owned());
    form_body.insert("password".to_owned(), password.to_owned());
    form_body.insert("submit".to_owned(), String::new());

    let html = client
        .post("https://login001.stockholm.se/siteminderagent/forms/login.fcc")
        .form(&form_body)
        .send()
        .await?
        .text()
        .await?;

    let form_body = scrape_form(html.as_str()).ok_or(Error::ScrapingFailed {
        details: "no sso request form found".into(),
    })?;

    let res = client
        .post("https://login001.stockholm.se/affwebservices/public/saml2sso")
        .form(&form_body)
        .send()
        .await?;

    if res.status() == StatusCode::BAD_REQUEST {
        debug!("bad credentials");
        return Err(Error::BadCredentials);
    }

    let form_body = scrape_form(res.text().await?.as_str()).ok_or(Error::ScrapingFailed {
        details: "no sso response form found".into(),
    })?;

    client
        .post("https://fnsservicesso1.stockholm.se/sso-ng/saml-2.0/response")
        .form(&form_body)
        .send()
        .await?;

    Ok(())
}

/// Start a session.
#[instrument(skip(password))]
pub async fn login(username: &str, password: &str) -> Result<Session> {
    let cookie_store = Arc::new(CookieStoreMutex::new(CookieStore::default()));

    let client = reqwest::Client::builder()
        .cookie_provider(cookie_store.clone())
        .user_agent(crate::client::USER_AGENT)
        .build()?;

    login_client(username, password, &client).await?;
    let scope = get_scope(&client).await?;

    drop(client);

    let lock = Arc::try_unwrap(cookie_store).expect("lock still has multiple owners");
    let cookie_store = lock.into_inner().expect("mutex cannot be locked");
    let cookies = cookie_store
        .iter_any()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    debug!("got {} cookies", cookies.len());

    Ok(Session { cookies, scope })
}
